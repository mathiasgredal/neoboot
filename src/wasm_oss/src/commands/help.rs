use super::{CommandDispatcher, CommandHandler};
use crate::commands::CommandRole;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    HelpClientRequest, HelpClientResponse,
};
use std::{collections::HashMap, error::Error};

pub struct HelpCommandHandler {
    pub dispatcher: CommandDispatcher,
}

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

    fn handle(&self, _: &client_request_inner::Payload) -> client_response_inner::Payload {
        let commands = self.dispatcher.handlers.borrow();
        let mut output: Vec<String> = Vec::new();
        output.push("Available commands:".to_string());
        for (_, handler) in commands.iter() {
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

        return client_response_inner::Payload::HelpResponse(HelpClientResponse { message });
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        match response {
            client_response_inner::Payload::HelpResponse(help_response) => {
                help_response.message.clone()
            }
            _ => "".to_string(),
        }
    }
}
