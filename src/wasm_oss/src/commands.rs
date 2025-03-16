use proto_rs::schema::{
    client_request::{
        client_request_inner::{self},
        ClientRequestInner,
    },
    client_response::{client_response_inner, ClientResponseInner},
    ClientRequest, HelpClientRequest, NonceClientRequest, PrintClientRequest,
};
use std::{any::TypeId, cell::RefCell, collections::HashMap, error::Error, rc::Rc};

mod help;
mod nonce;
mod print;

#[derive(PartialEq)]
enum CommandRole {
    Console, // Defines a command which
    System,  // Can be used by the system
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
    fn handle(&self, request: &client_request_inner::Payload) -> client_response_inner::Payload;
    fn response_as_string(&self, response: &client_response_inner::Payload) -> String;
}

#[derive(Clone)]
pub struct CommandDispatcher {
    handlers: Rc<RefCell<HashMap<TypeId, Box<dyn CommandHandler>>>>,
}

impl CommandDispatcher {
    pub fn blank() -> Self {
        Self {
            handlers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn default() -> Self {
        let mut dispatcher = Self::blank();
        dispatcher.register_handler::<NonceClientRequest>(Box::new(nonce::NonceCommandHandler));
        dispatcher.register_handler::<PrintClientRequest>(Box::new(print::PrintCommandHandler));
        dispatcher.register_handler::<HelpClientRequest>(Box::new(help::HelpCommandHandler {
            dispatcher: dispatcher.clone(),
        }));
        dispatcher
    }

    fn register_handler<T: 'static>(&mut self, handler: Box<dyn CommandHandler>) {
        self.handlers
            .borrow_mut()
            .insert(TypeId::of::<T>(), handler);
    }

    pub fn execute(&self, command: &str) -> Result<String, Box<dyn Error>> {
        let parts: Vec<String> = shell_words::split(command)?;
        if parts.is_empty() {
            return Err("Empty command".into());
        }

        // Find matching handler by command pattern
        for (_, handler) in self.handlers.borrow().iter() {
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
                let response = self.dispatch(&request)?;
                return Ok(handler.response_as_string(&response.payload.unwrap()));
            }
        }

        Err(format!("No matching command found: {}", command).into())
    }

    pub fn dispatch(&self, request: &ClientRequest) -> Result<ClientResponseInner, Box<dyn Error>> {
        // TODO: Handle signature verification

        let inner = request.inner.as_ref().ok_or("No inner payload")?;
        let payload = inner.payload.as_ref().ok_or("No payload")?;
        let type_id = match payload {
            client_request_inner::Payload::HelpRequest(_) => TypeId::of::<HelpClientRequest>(),
            client_request_inner::Payload::PrintRequest(_) => TypeId::of::<PrintClientRequest>(),
            client_request_inner::Payload::NonceRequest(_) => TypeId::of::<NonceClientRequest>(),
        };

        let response_payload = match self.handlers.borrow().get(&type_id) {
            Some(handler) => handler.handle(payload),
            None => return Err(format!("No handler registered for {:?}", type_id).into()),
        };

        // TODO: Handle signing of response

        Ok(ClientResponseInner {
            payload: Some(response_payload),
            nonce: "".to_string(),
        })
    }
}
