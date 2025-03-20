use super::{CommandDispatcher, CommandHandler};
use crate::commands::CommandRole;
use bytes::Bytes;
use futures::Stream;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    HelpClientRequest, HelpClientResponse,
};
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};

pub struct HelpCommandHandler {}

impl CommandHandler for HelpCommandHandler {
    fn cmd_pattern(&self) -> &'static str {
        "help"
    }

    fn cmd_description(&self) -> &'static str {
        "Display available commands"
    }

    fn cmd_roles(&self) -> Vec<CommandRole> {
        vec![CommandRole::Console, CommandRole::System]
    }

    fn parse_args(
        &self,
        _: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>> {
        Ok(client_request_inner::Payload::HelpRequest(
            HelpClientRequest {},
        ))
    }

    fn handle<'a>(
        &self,
        dispatcher: &CommandDispatcher,
        _: &client_request_inner::Payload,
        _: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        let commands = dispatcher.handlers.iter();
        let mut output: Vec<String> = Vec::new();
        output.push("Available commands:".to_string());
        for (_, handler) in commands {
            if !handler.cmd_roles().contains(&CommandRole::Console) {
                continue;
            }

            output.push(format!(
                "  {} - {}",
                handler.cmd_pattern(),
                handler.cmd_description()
            ));
        }
        let message = output.join("\n");

        Box::pin(async move {
            client_response_inner::Payload::HelpResponse(HelpClientResponse { message })
        })
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        match response {
            client_response_inner::Payload::HelpResponse(help_response) => {
                help_response.message.clone()
            }
            _ => "".to_string(),
        }
    }

    fn on_shutdown(&self) {}
}
