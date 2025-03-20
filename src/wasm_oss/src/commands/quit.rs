use super::{CommandDispatcher, CommandHandler, CommandRole};
use bytes::Bytes;
use futures::Stream;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    QuitClientRequest, QuitClientResponse,
};
use std::{
    collections::HashMap,
    error::Error,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

pub struct QuitCommandHandler {}

impl CommandHandler for QuitCommandHandler {
    fn cmd_pattern(&self) -> &'static str {
        "quit"
    }

    fn cmd_description(&self) -> &'static str {
        "Quit the application"
    }

    fn cmd_roles(&self) -> Vec<CommandRole> {
        vec![CommandRole::Console]
    }

    fn parse_args(
        &self,
        _: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>> {
        Ok(client_request_inner::Payload::QuitRequest(
            QuitClientRequest {},
        ))
    }

    fn handle<'a>(
        &self,
        dispatcher: &CommandDispatcher,
        _: &client_request_inner::Payload,
        _: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        *dispatcher.shutdown_flag.lock().unwrap() = true;
        Box::pin(async move { client_response_inner::Payload::QuitResponse(QuitClientResponse {}) })
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        match response {
            client_response_inner::Payload::QuitResponse(_) => "Quitting...".to_string(),
            _ => "".to_string(),
        }
    }

    fn on_shutdown(&self) {}
}
