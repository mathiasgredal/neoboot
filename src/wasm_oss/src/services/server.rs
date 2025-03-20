use crate::asyncio::net::TcpListener;
use crate::asyncio::{http::stream::AnyHttpStream, net::TcpStream};
use crate::commands::CommandDispatcher;
use crate::errors::lwip_error::LwipError;
use crate::executor::Executor;
use base64::prelude::*;
use bytes::Bytes;
use futures::future::{select, Either};
use futures::FutureExt;
use futures_lite::StreamExt;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::header::HeaderValue;
use hyper::{body::Incoming, server::conn::http1::Builder, service::service_fn};
use log::error;
use prost::Message;
use proto_rs::schema::ClientRequest;
use std::cell::RefCell;
use std::future::Future;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::rc::Rc;

/// HTTP server service that handles incoming connections and routes requests
pub struct ServerService<'a> {
    listener: Option<TcpListener>,
    dispatcher: Rc<RefCell<CommandDispatcher<'a>>>,
}

impl<'a> ServerService<'a> {
    /// Creates a new ServerService instance
    pub fn new(dispatcher: Rc<RefCell<CommandDispatcher<'a>>>) -> Self {
        Self {
            listener: None,
            dispatcher,
        }
    }

    /// Handles an incoming HTTP connection
    async fn handle_connection(
        dispatcher: Rc<RefCell<CommandDispatcher<'a>>>,
        executor: Executor<'a>,
        tcp_stream: AnyHttpStream<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = service_fn(move |req: Request<Incoming>| {
            let dispatcher = dispatcher.clone();
            let executor = executor.clone();
            return async move {
                let response = Self::handle_request(dispatcher.clone(), req).await;
                dispatcher
                    .borrow()
                    .finalize_shutdown_if_requested(&executor);
                response
            };
        });

        let mut http = Builder::new();
        http.keep_alive(false);
        http.max_buf_size(8192);
        if let Err(err) = http.serve_connection(tcp_stream, service).await {
            error!("Failed to serve connection: {err:#}");
        }

        Ok(())
    }

    /// Processes an HTTP request and returns an appropriate response
    async fn handle_request(
        dispatcher: Rc<RefCell<CommandDispatcher<'a>>>,
        mut req: Request<Incoming>,
    ) -> Result<Response<Full<Bytes>>, hyper::Error> {
        let mut response = Response::new(Full::default());

        match (req.method(), req.uri().path()) {
            // Root route - help information
            (&Method::GET, "/") => {
                // TODO: Add a more detailed help message, including the version of the server, client configuration, root public key, etc.
                *response.body_mut() = Full::from("Welcome to NeoBoot Local HTTP Server\n\nAvailable endpoints:\n- GET /: This help message\n- POST /api/v1/rpc: RPC service endpoint for client requests\n\nServer is running on port 8080\n");
            }

            // RPC service route
            (&Method::POST, "/api/v1/rpc") => {
                let client_request = match req.headers().get("X-Client-Request") {
                    Some(header) => match header.to_str() {
                        Ok(str) => str,
                        Err(_) => {
                            *response.status_mut() = StatusCode::BAD_REQUEST;
                            *response.body_mut() = Full::from("Invalid header encoding");
                            return Ok(response);
                        }
                    },
                    None => {
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        *response.body_mut() = Full::from("Missing X-Client-Request header");
                        return Ok(response);
                    }
                };

                let client_request =
                    match base64::engine::general_purpose::STANDARD.decode(client_request) {
                        Ok(decoded) => decoded,
                        Err(err) => {
                            *response.status_mut() = StatusCode::BAD_REQUEST;
                            *response.body_mut() =
                                Full::from(format!("Invalid base64 encoding: {}", err));
                            return Ok(response);
                        }
                    };

                let client_request: ClientRequest =
                    match ClientRequest::decode(client_request.as_slice()) {
                        Ok(client_request) => client_request,
                        Err(err) => {
                            *response.status_mut() = StatusCode::BAD_REQUEST;
                            *response.body_mut() =
                                Full::from(format!("Invalid ClientRequest encoding: {}", err));
                            return Ok(response);
                        }
                    };

                let stream = Some(req.body_mut().into_data_stream().boxed());
                let client_response =
                    match dispatcher.borrow().dispatch(&client_request, stream).await {
                        Ok(result) => result,
                        Err(err) => {
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() =
                                Full::from(format!("Error dispatching request: {}", err));
                            return Ok(response);
                        }
                    };

                // Encode the client response as a base64 string, as place it in the header
                let encoded_response = base64::engine::general_purpose::STANDARD
                    .encode(client_response.encode_to_vec());
                let header_map = response.headers_mut();
                header_map.insert(
                    "X-Client-Response",
                    HeaderValue::from_str(&encoded_response).unwrap(),
                );

                *response.status_mut() = StatusCode::OK;
            }

            // Catch-all 404 for unhandled routes
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            }
        };

        Ok(response)
    }
}

impl<'a> super::Service<'a> for ServerService<'a> {
    fn name(&self) -> &'static str {
        "server"
    }

    fn run(mut self: Box<Self>, executor: Executor<'a>) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        const DEFAULT_PORT: u16 = 8080;
        let addr = Ipv4Addr::UNSPECIFIED;

        self.listener = Some(TcpListener::bind(addr.to_string().as_str(), DEFAULT_PORT).unwrap());

        return Box::pin(async move {
            loop {
                let accept_fut = self.listener.as_ref().unwrap().accept().boxed();
                let exit_fut = executor.wait_for_exit().boxed();

                let accept = match select(accept_fut, exit_fut).await {
                    Either::Left((accept, _)) => accept,
                    Either::Right((_, _)) => Err(LwipError::ConnectionAborted),
                };

                match accept {
                    Ok(stream) => {
                        let tcp_stream = AnyHttpStream::Http(stream);

                        if let Err(err) = Self::handle_connection(
                            self.dispatcher.clone(),
                            executor.clone(),
                            tcp_stream,
                        )
                        .await
                        {
                            error!("Failed to handle connection: {err:?}");
                        }
                    }
                    Err(err) => {
                        if err == LwipError::ConnectionAborted {
                            return;
                        }
                        error!("Failed to accept connection: {err:?}");
                    }
                }
            }
        });
    }
}
