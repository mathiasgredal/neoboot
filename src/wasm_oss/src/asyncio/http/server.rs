use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1::Builder;
use hyper::service::service_fn;
use log::{error, info};
use std::net::Ipv4Addr;

use crate::asyncio::http::stream::AnyHttpStream;
use crate::asyncio::net::TcpListener;
use crate::executor::Executor;

pub async fn run_server(executor: &Executor<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = Ipv4Addr::UNSPECIFIED;
    let port = 8080;

    info!("Starting to serve on http://{}:{}", addr, port);

    let incoming = TcpListener::bind(addr.to_string().as_str(), port)?;

    let service = service_fn(echo);

    loop {
        let tcp_stream = AnyHttpStream::Http(incoming.accept().await?);

        executor.spawn(async move {
            if let Err(err) = Builder::new().serve_connection(tcp_stream, service).await {
                error!("failed to serve connection: {err:#}");
            }
        });
    }
}

// Custom echo service, handling two different routes and a
// catch-all 404 responder.
async fn echo(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response = Response::new(Full::default());
    match (req.method(), req.uri().path()) {
        // Help route.
        (&Method::GET, "/") => {
            *response.body_mut() = Full::from("Try POST /echo\n");
        }
        // Echo service route.
        (&Method::POST, "/echo") => {
            *response.body_mut() = Full::from(req.into_body().collect().await?.to_bytes());
        }
        // Catch-all 404.
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}
