use super::{CommandHandler, CommandRole};
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    PrintClientRequest, PrintClientResponse,
};
use std::{collections::HashMap, error::Error};

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

    fn handle(&self, request: &client_request_inner::Payload) -> client_response_inner::Payload {
        let message = match request {
            client_request_inner::Payload::PrintRequest(print_request) => {
                print_request.message.clone()
            }
            _ => "".to_string(),
        };
        return client_response_inner::Payload::PrintResponse(PrintClientResponse { message });
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        match response {
            client_response_inner::Payload::PrintResponse(print_response) => {
                print_response.message.clone()
            }
            _ => "".to_string(),
        }
    }
}
