#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;

use futures::future::FutureResult;

use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

struct Tug;

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
    let addr = "0.0.0.0:7357".parse().unwrap();

    let server = Http::new().bind(&addr, || Ok(Tug)).unwrap();
    println!("Serving at http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}
