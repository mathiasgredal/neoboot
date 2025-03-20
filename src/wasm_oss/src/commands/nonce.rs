use super::{CommandDispatcher, CommandHandler, CommandRole};
use bytes::Bytes;
use futures::Stream;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    NonceClientRequest, NonceClientResponse,
};
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};

pub struct NonceCommandHandler;

impl CommandHandler for NonceCommandHandler {
    fn cmd_pattern(&self) -> &'static str {
        "nonce"
    }

    fn cmd_description(&self) -> &'static str {
        "Generate a random nonce"
    }

    fn cmd_roles(&self) -> Vec<CommandRole> {
        vec![CommandRole::System, CommandRole::Console]
    }

    fn parse_args(
        &self,
        _: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>> {
        Ok(client_request_inner::Payload::NonceRequest(
            NonceClientRequest {},
        ))
    }

    fn handle<'a>(
        &self,
        _: &CommandDispatcher,
        _: &client_request_inner::Payload,
        _: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        Box::pin(async move {
            client_response_inner::Payload::NonceResponse(NonceClientResponse {
                nonce: "".to_string(),
            })
        })
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        match response {
            client_response_inner::Payload::NonceResponse(nonce_response) => {
                nonce_response.nonce.clone()
            }
            _ => "".to_string(),
        }
    }

    fn on_shutdown(&self) {}
}
