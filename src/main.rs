/*!

ALPHA easy configurable web server.

*/

#![deny(missing_docs)]

extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate libflate;

use futures::future::FutureResult;

use hyper::StatusCode;
use hyper::header::{ContentLength, ContentEncoding, Encoding, Date};
use hyper::server::{Http, Service, Request, Response};

use libflate::gzip::Encoder;

use std::thread;
use std::path::Path;
use std::fs::File;
use std::io::{self, Read};
use std::time::SystemTime;
use std::env::args;

/// Main config
#[derive(Debug, Deserialize)]
struct Config {
    server: Option<Vec<ServerConfig>>,
}


/// Server config struct
#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: Option<String>,
    root: Option<String>,
    gzip: Option<bool>,
}


/// Empty struct for the Tug service
struct Tug {
    root: String,
    gzip: bool,
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
            let _ = file.read_to_end(&mut buf);

            // gzip encoding
            if self.gzip {
                let mut encoder = Encoder::new(Vec::new()).unwrap();
                io::copy(&mut &buf[..], &mut encoder).unwrap();
                buf = encoder.finish().into_result().unwrap();
            }

            Response::new()
                .with_header(ContentLength(buf.len() as u64))
                .with_header(ContentEncoding(vec![Encoding::Gzip, Encoding::Chunked]))
                .with_header(Date(SystemTime::now().into()))
                .with_body(buf)
        } else {
            Response::new().with_status(StatusCode::NotFound)
        })
    }
}


fn main() {
    pretty_env_logger::init().unwrap();
    let file_path = args().nth(1).unwrap_or("tug.toml".to_string());
    let mut config_file = File::open(file_path).unwrap();
    let mut toml_str = String::new();
    let _ = config_file.read_to_string(&mut toml_str);

    let config: Config = toml::from_str(toml_str.as_str()).unwrap();

    for server_config in config.server.unwrap() {
        let host = server_config.host.unwrap();
        let addr = host.parse().unwrap();
        let root = server_config.root.unwrap_or(".".to_string());
        let gzip = server_config.gzip.unwrap_or(true);

        thread::spawn(move || {
            let server = Http::new()
                .bind(&addr, move || {
                    let tug = Tug {
                        root: root.clone(),
                        gzip: gzip,
                    };
                    Ok(tug)
                })
                .unwrap();
            info!("Serving at http://{}", server.local_addr().unwrap());
            server.run().unwrap();
        });
    }

    thread::park();
}
