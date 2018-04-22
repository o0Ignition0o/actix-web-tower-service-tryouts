extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate openssl;
extern crate tower;

use futures::prelude::{Async, Poll};
use std::sync::Arc;
use tower::Service;
use actix_web::*;
use futures::future::{result, Future};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web::{server, App, HttpRequest, dev::Handler};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

trait ServiceHandlerBridge<S>
where
    S: 'static,
{
    fn invoke(
        &mut self,
        request: HttpRequest<S>,
    ) -> Box<Future<Item = HttpResponse, Error = Error>>;
}

impl<S> Service for ServiceHandlerBridge<S>
where
    ServiceHandlerBridge<S>: 'static,
{
    type Request = HttpRequest<S>;
    type Response = HttpResponse;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn call(&mut self, request: HttpRequest<S>) -> Self::Future {
        self.invoke(request)
    }
}

impl<S> Handler<S> for ServiceHandlerBridge<S>
where
    S: 'static,
{
    type Result = Box<Future<Item = HttpResponse, Error = Error>>;

    fn handle(&mut self, request: HttpRequest<S>) -> Self::Result {
        self.invoke(request)
    }
}

struct HelloWorld(Arc<AtomicUsize>);

impl<S> ServiceHandlerBridge<S> for HelloWorld
where
    S: 'static,
{
    fn invoke(
        &mut self,
        request: HttpRequest<S>,
    ) -> Box<Future<Item = HttpResponse, Error = Error>> {
        let name: String = request.query().get("name").unwrap_or("world").into();
        let times_invoked = self.0.fetch_add(1, Relaxed);
        let hello_response = format!(
            "Hello {}, welcome to the actix tower-service test ! I've been invoked {} times so far :)",
            name,
            times_invoked
        );
        result(Ok(HttpResponse::from(hello_response))).responder()
    }
}

fn main() {
    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
    let sys = actix::System::new("ws-example");

    let inc = Arc::new(AtomicUsize::new(1));

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

mod test {
    #[test]
    fn test_hello_world() {
        use actix_web::test::TestServer;
        use actix_web::{http, HttpMessage};
        use HelloWorld;
        use std::sync::Arc;
        use std::sync::atomic::AtomicUsize;

        let mut srv = TestServer::new(|app| app.handler(HelloWorld(Arc::new(AtomicUsize::new(1))))); // <- Start new test server
        let request = srv.get().finish().unwrap(); // <- create client request
        let first_response = srv.execute(request.send()).unwrap(); // <- send request to the server

        assert!(first_response.status().is_success()); // <- check response
        let first_response_text =
            String::from_utf8(srv.execute(first_response.body()).unwrap().to_vec()).unwrap();

        assert!(first_response_text.contains("world"),
            format!(
                "A request without name parameter should contain Hello World in the response. Got : \n {:?}", 
                first_response_text
            )
        ); // <- check name parameter

        assert!(first_response_text.contains("invoked 1 times"),
            format!(
                "The first request should claim the server has been invoked only one time. Got : \n {:?}", 
                first_response_text
            )
        ); // <- check number of times invoked

        let name = "Jeremy";
        let second_request = srv.client(http::Method::GET, &format!("/?name={}", name))
            .finish()
            .unwrap(); // <- create a client request with a name parameter
        let second_response = srv.execute(second_request.send()).unwrap(); // <- send the second request to the server

        assert!(second_response.status().is_success()); // <- check response
        let second_response_text =
            String::from_utf8(srv.execute(second_response.body()).unwrap().to_vec()).unwrap();

        assert!(second_response_text.contains(name),
            format!(
                "A request with {} as name parameter should contain {} in the response. Got : \n {:?}", 
                name,
                name,
                second_response_text
            )
        ); // <- check name parameter

        assert!(second_response_text.contains("invoked 2 times"),
            format!(
                "The first request should claim the server has been invoked only one time. Got : \n {:?}", 
                second_response_text
            )
        ); // <- check number of times invoked
    }
}
