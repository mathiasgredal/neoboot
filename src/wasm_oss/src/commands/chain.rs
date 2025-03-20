use crate::ffi;

use super::{CommandDispatcher, CommandHandler, CommandRole};
use bytes::Bytes;
use futures::Stream;
use futures_lite::StreamExt;
use log::info;
use proto_rs::schema::{
    client_request::client_request_inner,
    client_response::client_response_inner::{self},
    ChainClientResponse, ErrorClientResponse,
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};

pub struct ChainCommandHandler {}

impl CommandHandler for ChainCommandHandler {
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
        stream: Option<Pin<Box<dyn Stream<Item = Result<Bytes, hyper::Error>> + Send + 'a>>>,
    ) -> Pin<Box<dyn Future<Output = client_response_inner::Payload> + Send + 'a>> {
        info!("Handling chain request");
        let message = match request {
            client_request_inner::Payload::ChainRequest(chain_request) => {
                Some(chain_request.clone())
            }
            _ => None,
        };

        let shutdown_flag = dispatcher.shutdown_flag.clone();
        Box::pin(async move {
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

            // Allocate a buffer to store the stream data
            // TODO: Figure out when to free this buffer
            let buf_ptr = unsafe { ffi::env_malloc(message.payload_size as u32) };
            let buf_len = message.payload_size as u32;
            let mut buf_hasher = Sha256::new();
            let mut offset = 0;

            while let Some(item) = stream.next().await {
                let item = item.unwrap();
                // info!("Received item of size {} bytes", item.len());
                // Print the hex of the item
                unsafe { ffi::env_memcpy(item.as_ptr(), buf_ptr + offset, item.len() as u32) };
                offset += item.len() as u64;
                buf_hasher.update(item);
            }
            let buf_hash = buf_hasher.finalize();
            info!("chainload payload hash: {:x}", buf_hash);

            unsafe {
                ffi::env_set_wasm_chainload(buf_ptr, buf_len);
            }

            *shutdown_flag.lock().unwrap() = true;

            client_response_inner::Payload::ChainResponse(ChainClientResponse {})
        })
    }

    fn response_as_string(&self, _: &client_response_inner::Payload) -> String {
        unimplemented!()
    }

    fn on_shutdown(&self) {}
}
