/*!

  ALPHA easy configurable web server.

*/

#![deny(missing_docs)]

extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate acme_client;
extern crate chrono;
extern crate clap;
extern crate fern;
extern crate libflate;
extern crate pulldown_cmark;

use futures::future::FutureResult;

use hyper::header::{ContentEncoding, ContentLength, Date, Encoding, Headers};
use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;

use libflate::gzip::Encoder;

use pulldown_cmark::{html, Parser};

use clap::{App, Arg};

use acme_client::Directory;

use std::env::args;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::thread;
use std::time::SystemTime;

const DEFAULT_CONFIG: &'static str = "tug.toml";

/// Main config
#[derive(Debug, Deserialize)]
struct Config {
    log: Option<String>,
    server: Option<Vec<ServerConfig>>,
}

/// Server config struct
#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: Option<String>,
    root: Option<String>,
    gzip: Option<bool>,
    markdown: Option<MarkdownConfig>,
}

/// Markdown config struct
#[derive(Debug, Deserialize, Clone)]
struct MarkdownConfig {
    js: Option<String>,
    css: Option<String>,
}

/// Empty struct for the Tug service
struct Tug {
    root: String,
    gzip: bool,
    markdown: Option<MarkdownConfig>,
}

/// Tug service implementation
impl Service for Tug {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let root_path = self.root.clone();
        let path_string = format!("{}{}", root_path, req.path());
        let file_path = Path::new(&path_string);

        futures::future::ok(if file_path.exists() {
            let mut file = File::open(file_path).unwrap();
            let mut buf = Vec::new();

            if self.markdown.is_some() && file_path.extension() == Some(OsStr::new("md")) {
                let markdown_conf = self.markdown.clone().unwrap();
                let mut string_buf = String::new();
                let _ = file.read_to_string(&mut string_buf);

                let parser = Parser::new(string_buf.as_str());

                let mut html_buf = String::new();
                if markdown_conf.css.is_some() {
                    html_buf.push_str(
                        format!(
                            "<link rel=\"stylesheet\" type=\"text/css\" href=\"{}\">",
                            markdown_conf.css.unwrap()
                        ).as_str(),
                    );
                }
                html::push_html(&mut html_buf, parser);
                if markdown_conf.js.is_some() {
                    html_buf.push_str(
                        format!("<script src=\"{}\"></script>", markdown_conf.js.unwrap()).as_str(),
                    );
                }

                buf = html_buf.into_bytes();
            } else {
                let _ = file.read_to_end(&mut buf);
            }

            let mut headers = Headers::new();
            headers.set(Date(SystemTime::now().into()));

            // gzip encoding
            if self.gzip {
                let mut encoder = Encoder::new(Vec::new()).unwrap();
                io::copy(&mut &buf[..], &mut encoder).unwrap();
                buf = encoder.finish().into_result().unwrap();
                headers.set(ContentEncoding(vec![Encoding::Gzip, Encoding::Chunked]));
            }

            headers.set(ContentLength(buf.len() as u64));

            Response::new().with_headers(headers).with_body(buf)
        } else {
            Response::new().with_status(StatusCode::NotFound)
        })
    }
}

/// Config file handling
fn handle_config() -> Option<Config> {
    let (file_path, default) = match args().nth(1) {
        Some(fp) => (fp, false),
        None => (DEFAULT_CONFIG.to_string(), true),
    };

    let toml_str = match File::open(file_path) {
        Ok(mut f) => {
            let mut toml_str = String::new();
            let _ = f.read_to_string(&mut toml_str);
            toml_str
        }
        Err(e) => {
            if !default {
                error!("Config: {}", e);
                return None;
            }
            info!("Using default");
            "[[server]]\nhost = \"127.0.0.1:8080\"".to_string()
        }
    };

    Some(toml::from_str(toml_str.as_str()).unwrap())
}

/// Setup logging to a file
fn setup_logging(path: String) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file(path)?)
        // Apply globally
        .apply()?;

    Ok(())
}

/// Setup SSL
fn setup_ssl(host: String, root: String) -> Result<(), acme_client::error::Error> {
    let directory = Directory::lets_encrypt()?;
    let account = directory.account_registration().register()?;

    // Create a identifier authorization for example.com
    let authorization = account.authorization(host.as_str())?;

    // Validate ownership of example.com with http challenge
    let http_challenge = authorization
        .get_http_challenge()
        .ok_or("HTTP challenge not found")?;
    http_challenge.save_key_authorization(root.as_str())?;
    http_challenge.validate()?;

    let cert = account
        .certificate_signer(&[host.as_str()])
        .sign_certificate()?;
    cert.save_signed_certificate("certificate.pem")?;
    cert.save_private_key("certificate.key")?;

    Ok(())
}

/// Start the server blocks
fn start_servers(server_configs: Vec<ServerConfig>) -> Result<(), ()> {
    for server_config in server_configs {
        let host = server_config.host.unwrap();
        let addr = host.parse().unwrap();
        let root = server_config.root.unwrap_or(".".to_string());
        let gzip = server_config.gzip.unwrap_or(true);
        let markdown = server_config.markdown;

        let _ = setup_ssl(host, root.clone());

        thread::spawn(move || {
            let server = Http::new()
                .bind(&addr, move || {
                    let tug = Tug {
                        root: root.clone(),
                        gzip: gzip,
                        markdown: markdown.clone(),
                    };
                    Ok(tug)
                })
                .unwrap();
            info!("Serving at http://{}", server.local_addr().unwrap());
            server.run().unwrap();
        });
    }

    Ok(())
}

fn main() {
    let _ = App::new("tug")
        .version("0.1.0")
        .author("S. Hockham <shockham@protonmail.com>")
        .about("Easy to configure web server")
        .arg(
            Arg::with_name("CONFIG")
                .help("Sets the config file to use")
                .required(false)
                .index(1),
        )
        .get_matches();

    let config = match handle_config() {
        Some(c) => c,
        None => return,
    };

    let log_path = config.log.unwrap_or("output.log".to_string());
    setup_logging(log_path).unwrap();

    start_servers(config.server.unwrap()).unwrap();

    thread::park();
}
