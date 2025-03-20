use crate::executor::Executor;

use super::{CommandDispatcher, CommandHandler, CommandRole};
use bytes::Bytes;
use futures::Stream;
use log::info;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    StatusClientRequest, StatusClientResponse,
};
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};

pub struct StatusCommandHandler<'b> {
    pub executor: Executor<'b>,
}

impl<'b> CommandHandler for StatusCommandHandler<'b> {
    fn cmd_pattern(&self) -> &'static str {
        "status"
    }

    fn cmd_description(&self) -> &'static str {
        "Get the status of the application"
    }

    fn cmd_roles(&self) -> Vec<CommandRole> {
        vec![CommandRole::Console]
    }

    fn parse_args(
        &self,
        args: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>> {
        Ok(client_request_inner::Payload::StatusRequest(
            StatusClientRequest {},
        ))
    }

    fn handle<'a>(
        &self,
        _: &CommandDispatcher,
        request: &client_request_inner::Payload,
        _: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        info!("Active tasks: {:?}", self.executor.active_tasks());
        let _ = match request {
            client_request_inner::Payload::StatusRequest(_) => "Status".to_string(),
            _ => "".to_string(),
        };
        Box::pin(
            async move { client_response_inner::Payload::StatusResponse(StatusClientResponse {}) },
        )
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
