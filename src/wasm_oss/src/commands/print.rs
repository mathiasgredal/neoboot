use super::{CommandDispatcher, CommandHandler, CommandRole};
use bytes::Bytes;
use futures::Stream;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    PrintClientRequest, PrintClientResponse,
};
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};

pub struct PrintCommandHandler;

impl CommandHandler for PrintCommandHandler {
    fn cmd_pattern(&self) -> &'static str {
        "print <message>"
    }

    fn cmd_description(&self) -> &'static str {
        "Print a message"
    }

    fn cmd_roles(&self) -> Vec<CommandRole> {
        vec![CommandRole::Console]
    }

    fn parse_args(
        &self,
        args: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>> {
        Ok(client_request_inner::Payload::PrintRequest(
            PrintClientRequest {
                message: args["message"].clone(),
            },
        ))
    }

    fn handle<'a>(
        &self,
        _: &CommandDispatcher,
        request: &client_request_inner::Payload,
        _: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        let message = match request {
            client_request_inner::Payload::PrintRequest(print_request) => {
                print_request.message.clone()
            }
            _ => "".to_string(),
        };
        Box::pin(async move {
            client_response_inner::Payload::PrintResponse(PrintClientResponse { message })
        })
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        match response {
            client_response_inner::Payload::PrintResponse(print_response) => {
                print_response.message.clone()
            }
            _ => "".to_string(),
        }
    }

    fn on_shutdown(&self) {}
}
