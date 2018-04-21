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

impl HelloWorld {
    fn greet(&mut self, name: String) -> Box<Future<Item = String, Error = Error>> {
        let times_invoked = self.0.fetch_add(1, Relaxed);
        let hello_response = format!(
            "Hello {}, welcome to the actix tower-service test ! I've been invoked {} times so far :)",
            name,
            times_invoked
        );
        result(Ok(hello_response)).responder()
    }
}

impl<S> Handler<S> for HelloWorld {
    type Result = Box<Future<Item = String, Error = Error>>;

    fn handle(&mut self, request: HttpRequest<S>) -> Self::Result {
        self.greet(request.query().get("name").unwrap_or("world").into())
    }
}

impl ReadyService for HelloWorld {
    type Request = HttpRequest;
    type Response = String;
    type Error = Error;
    type Future = Box<Future<Item = String, Error = Error>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        self.greet(request.query().get("name").unwrap_or("world").into())
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
