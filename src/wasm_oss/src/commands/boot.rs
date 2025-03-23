use super::{CommandDispatcher, CommandHandler, CommandRole, HandleStream};
use crate::{
    controllers::boot::{BootController, PayloadType},
    utils::msgpack::MessagePackByteStream,
};
use bytes::Bytes;
use futures::{lock::Mutex, Stream};
use futures_lite::StreamExt;
use log::{error, info};
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    BootClientResponse, ErrorClientResponse, NonceClientRequest, NonceClientResponse,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
    str::FromStr,
    sync::Arc,
};

pub struct BootCommandHandler {
    pub boot_controller: Arc<Mutex<BootController>>,
}

impl CommandHandler for BootCommandHandler {
    fn cmd_pattern(&self) -> &'static str {
        unimplemented!()
    }

    fn cmd_description(&self) -> &'static str {
        unimplemented!()
    }

    fn cmd_roles(&self) -> Vec<CommandRole> {
        vec![CommandRole::System, CommandRole::Stream]
    }

    fn parse_args(
        &self,
        _: &HashMap<String, String>,
    ) -> Result<client_request_inner::Payload, Box<dyn Error>> {
        unimplemented!()
    }

    fn handle<'a>(
        &self,
        dispatcher: &CommandDispatcher,
        request: &client_request_inner::Payload,
        stream: Option<HandleStream<'a>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        let boot_controller = self.boot_controller.clone();
        let message = match request {
            client_request_inner::Payload::BootRequest(boot_request) => Some(boot_request.clone()),
            _ => None,
        };

        let shutdown_flag = dispatcher.shutdown_flag.clone();
        Box::pin(async move {
            let boot_controller = boot_controller.clone();
            let message = match message {
                Some(message) => message,
                None => {
                    return client_response_inner::Payload::ErrorResponse(ErrorClientResponse {
                        error: "No message provided".to_string(),
                    });
                }
            };

            let mut stream = match stream {
                Some(stream) => stream,
                None => {
                    return client_response_inner::Payload::ErrorResponse(ErrorClientResponse {
                        error: "No stream provided".to_string(),
                    });
                }
            };

            let mut msgpack_stream = MessagePackByteStream::new();

            // TODO: Check hash and size

            while let Some(item) = stream.next().await {
                msgpack_stream.extend_buffer(item.unwrap());

                while let Some(keyed_bytes) = msgpack_stream.process_bytes() {
                    if (keyed_bytes.is_err()) {
                        info!("Error processing bytes: {:?}", keyed_bytes.err());
                        break;
                    }

                    let keyed_bytes = keyed_bytes.unwrap();
                    let payload_type = PayloadType::from_str(&keyed_bytes.key);
                    if (payload_type.is_err()) {
                        info!("Error parsing payload type: {:?}", payload_type.err());
                        continue;
                    }

                    let result = boot_controller
                        .lock()
                        .await
                        .put_payload_bytes(
                            payload_type.unwrap(),
                            keyed_bytes.length,
                            keyed_bytes.data,
                        )
                        .await;

                    if result.is_err() {
                        info!("Error putting payload bytes: {:?}", result.err());
                    }
                }
            }
            *shutdown_flag.lock().unwrap() = true;
            client_response_inner::Payload::BootResponse(BootClientResponse {})
        })
    }

    fn response_as_string(&self, response: &client_response_inner::Payload) -> String {
        unimplemented!()
    }

    fn on_shutdown(&self) {}
}
