/*!

ALPHA easy configurable web server.

*/

#![deny(missing_docs)]

extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate toml;
#[macro_use] extern crate serde_derive;

use futures::future::FutureResult;

use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

use std::thread;

/// Main config
#[derive(Debug, Deserialize)]
struct Config {
    server: Option<Vec<ServerConfig>>,
}


/// Server config struct
#[derive(Debug, Deserialize)]
struct ServerConfig {
    ip: Option<String>,
}


/// Empty struct for the Tug service
struct Tug;

/// Tug service implementation
impl Service for Tug {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        futures::future::ok(match (req.method(), req.path()) {
            (&Get, "/") => {
                let test: &'static [u8] = b"test get";
                Response::new()
                    .with_header(ContentLength(test.len() as u64))
                    .with_body(test)
            },
            (&Post, "/") => {
                let test: &'static [u8] = b"test post";
                Response::new()
                    .with_header(ContentLength(test.len() as u64))
                    .with_body(test)
            },
            _ => {
                Response::new()
                    .with_status(StatusCode::NotFound)
            }
        })
    }

}


fn main() {
    pretty_env_logger::init().unwrap();
    let toml_str = r#"
        [[server]]
        ip = "127.0.0.1:7357"
        [[server]]
        ip = "127.0.0.1:1337"
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();

    for server_config in config.server.unwrap() {
        let ip = server_config.ip.unwrap();
        let addr = ip.parse().unwrap();

        thread::spawn(move || {
            let server = Http::new().bind(&addr, || Ok(Tug)).unwrap();
            info!("Serving at http://{}", server.local_addr().unwrap());
            server.run().unwrap();
        });
    }

    thread::park();
}
