extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate openssl;
extern crate tower;

use std::sync::Arc;
use tower::ReadyService;
use actix_web::*;
use futures::future::{result, Future};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web::{server, App, HttpRequest, dev::Handler};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct HelloWorld(Arc<AtomicUsize>);

impl<S> Handler<S> for HelloWorld {
    type Result = Box<Future<Item = String, Error = Error>>;
    fn handle(&mut self, request: HttpRequest<S>) -> Self::Result {
        let times_invoked = self.0.fetch_add(1, Relaxed);
        let hello_response = format!(
            "Hello {}, welcome to the actix tower-service test ! I've been invoked {} times so far :)",
            request.query().get("name").unwrap_or("world"),
            times_invoked
        );
        result(Ok(hello_response)).responder()
    }
}

impl ReadyService for HelloWorld {
    type Request = HttpRequest;
    type Response = String;
    type Error = Error;
    type Future = Box<Future<Item = String, Error = Error>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let times_invoked = self.0.fetch_add(1, Relaxed);
        let hello_response = format!(
            "Hello {}, welcome to the actix tower-service test ! I've been invoked {} times so far :)",
            request.query().get("name").unwrap_or("world"),
            times_invoked
        );
        result(Ok(hello_response)).responder()
    }
}

fn main() {
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
    let sys = actix::System::new("ws-example");

    let inc = Arc::new(AtomicUsize::new(0));

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    server::new(move || {
        let cloned = inc.clone();
        App::new().resource("/", move |r| r.h(HelloWorld(cloned)))
    }).bind("127.0.0.1:8443")
        .unwrap()
        .start_ssl(builder)
        .unwrap();

    println!("Started http server: 127.0.0.1:8443");
    let _ = sys.run();
}
