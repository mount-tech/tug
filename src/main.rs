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

use futures::future::FutureResult;

use hyper::StatusCode;
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

use std::thread;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

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
}


/// Empty struct for the Tug service
struct Tug {
    root: String,
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

            Response::new()
                .with_header(ContentLength(buf.len() as u64))
                .with_body(buf)
        } else {
            Response::new().with_status(StatusCode::NotFound)
        })
    }
}


fn main() {
    pretty_env_logger::init().unwrap();
    let toml_str = r#"
        [[server]]
        host = "127.0.0.1:7357"

        [[server]]
        host = "127.0.0.1:1337"
        root = "./src"
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();

    for server_config in config.server.unwrap() {
        let host = server_config.host.unwrap();
        let addr = host.parse().unwrap();
        let root = server_config.root.unwrap_or(".".to_string());

        thread::spawn(move || {
            let server = Http::new()
                .bind(&addr, move || {
                    let tug = Tug { root: root.clone() };
                    Ok(tug)
                })
                .unwrap();
            info!("Serving at http://{}", server.local_addr().unwrap());
            server.run().unwrap();
        });
    }

    thread::park();
}
