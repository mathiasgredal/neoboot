use crate::executor::Executor;
use bytes::Bytes;
use futures_lite::Stream;
use proto_rs::schema::{
    client_request::{
        client_request_inner::{self},
        ClientRequestInner,
    },
    client_response::{client_response_inner, ClientResponseInner},
    ChainClientRequest, ClientRequest, ClientResponse, HelpClientRequest, NonceClientRequest,
    PrintClientRequest, QuitClientRequest, StatusClientRequest,
};
use std::{
    any::TypeId,
    collections::HashMap,
    error::Error,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

mod chain;
mod help;
mod nonce;
mod print;
mod quit;
mod status;

#[derive(PartialEq)]
enum CommandRole {
    Console, // Defines a command which
    System,  // Can be used by the system
    Stream,  // Can handle a stream of data
}

// Trait for command handlers
trait CommandHandler {
    fn cmd_pattern(&self) -> &'static str;
    fn cmd_description(&self) -> &'static str;
    fn cmd_roles(&self) -> Vec<CommandRole>;
    fn parse_args(
        &self,
        args: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>>;
    fn handle<'a>(
        &self,
        dispatcher: &CommandDispatcher<'a>,
        request: &client_request_inner::Payload,
        stream: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>>;
    fn response_as_string(&self, response: &client_response_inner::Payload) -> String;
    fn on_shutdown(&self);
}

pub struct CommandDispatcher<'a> {
    handlers: HashMap<TypeId, Box<dyn CommandHandler + 'a>>,
    shutdown_flag: Arc<Mutex<bool>>,
}

impl<'a> CommandDispatcher<'a> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            shutdown_flag: Arc::new(Mutex::new(false)),
        }
    }

    pub fn default() -> Self {
        let mut dispatcher = Self::new();
        dispatcher.register_handler::<NonceClientRequest>(nonce::NonceCommandHandler {});
        dispatcher.register_handler::<HelpClientRequest>(help::HelpCommandHandler {});
        dispatcher.register_handler::<PrintClientRequest>(print::PrintCommandHandler {});
        dispatcher.register_handler::<QuitClientRequest>(quit::QuitCommandHandler {});
        dispatcher.register_handler::<ChainClientRequest>(chain::ChainCommandHandler {});
        dispatcher
    }

    fn register_handler<T: 'static>(&mut self, handler: impl CommandHandler + 'a) {
        self.handlers.insert(TypeId::of::<T>(), Box::new(handler));
    }

    pub async fn execute(&self, command: &str) -> Result<String, Box<dyn Error>> {
        let parts: Vec<String> = shell_words::split(command)?;
        if parts.is_empty() {
            return Err("Empty command".into());
        }

        // Find matching handler by command pattern
        for (_, handler) in self.handlers.iter() {
            if !handler.cmd_roles().contains(&CommandRole::Console) {
                continue;
            }

            let pattern_parts: Vec<&str> = handler.cmd_pattern().split_whitespace().collect();
            if pattern_parts.len() != parts.len() {
                continue;
            }

            // Check if pattern matches
            let mut args = HashMap::new();
            let mut matches = true;
            for (pattern, value) in pattern_parts.iter().zip(parts.iter()) {
                if pattern.starts_with('<') && pattern.ends_with('>') {
                    // This is a parameter - extract name without < >
                    let param_name = &pattern[1..pattern.len() - 1];
                    args.insert(param_name.to_string(), value.to_string());
                } else if *pattern != *value {
                    matches = false;
                    break;
                }
            }

            if matches {
                // Found matching handler
                let request_inner = handler.parse_args(&args)?;
                let request = ClientRequest {
                    inner: Some(ClientRequestInner {
                        payload: Some(request_inner),
                        nonce: "".to_string(),
                    }),
                    signature_type: None,
                };
                let response = self.dispatch(&request, None).await?;
                return Ok(handler.response_as_string(&response.inner.unwrap().payload.unwrap()));
            }
        }

        Err(format!("No matching command found: {}", command).into())
    }

    pub async fn dispatch(
        &self,
        request: &ClientRequest,
        stream: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Result<ClientResponse, Box<dyn Error>> {
        // TODO: Handle signature verification

        let inner = request.inner.as_ref().ok_or("No inner payload")?;
        let payload = inner.payload.as_ref().ok_or("No payload")?;
        let type_id = match payload {
            client_request_inner::Payload::HelpRequest(_) => TypeId::of::<HelpClientRequest>(),
            client_request_inner::Payload::PrintRequest(_) => TypeId::of::<PrintClientRequest>(),
            client_request_inner::Payload::NonceRequest(_) => TypeId::of::<NonceClientRequest>(),
            client_request_inner::Payload::QuitRequest(_) => TypeId::of::<QuitClientRequest>(),
            client_request_inner::Payload::ChainRequest(_) => TypeId::of::<ChainClientRequest>(),
            client_request_inner::Payload::StatusRequest(_) => TypeId::of::<StatusClientRequest>(),
        };

        let response_payload = match self.handlers.get(&type_id) {
            Some(handler) => handler.handle(self, payload, stream).await,
            None => return Err(format!("No handler registered for {:?}", type_id).into()),
        };

        // TODO: Handle signing of response

        Ok(ClientResponse {
            inner: Some(ClientResponseInner {
                payload: Some(response_payload),
                nonce: "".to_string(),
            }),
            signature_type: None,
        })
    }

    /// Safely finalizes a shutdown sequence if requested by a command handler.
    ///
    /// This method should be called after command dispatch has completed to ensure
    /// that any shutdown request is handled in a coordinated way, preventing race conditions
    /// between returning command results and shutting down the system.
    /// In normal operation (no shutdown requested), this method does nothing.
    pub fn finalize_shutdown_if_requested<'b: 'a>(&self, executor: &Executor<'b>) {
        // Only proceed if a shutdown has been requested
        if !(*self.shutdown_flag.lock().unwrap()) {
            return;
        }

        // Notify all handlers about the shutdown
        for (_, handler) in self.handlers.iter() {
            handler.on_shutdown();
        }

        // Finally terminate the application
        executor.exit();
    }
}
