use super::request::RequestConfig;
use super::response::Response;
use super::response::ResponseChunk;
use super::response::ResponseData;
use super::response::ResponseMetadata;
use crate::asyncio::dns::Dns;
use crate::asyncio::tcp::TcpSocket;
use async_fn_stream::try_fn_stream;
use bytes::Bytes;
use embedded_io_async::Read;
use log::info;
use reqwless::client::HttpClient;
use reqwless::client::TlsConfig;
use reqwless::request::Method;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::time::Duration;

pub struct Client {
    socket: TcpSocket,
    tls_read_buffer: Vec<u8>,
    tls_write_buffer: Vec<u8>,
    dns: Dns,
    base_url: String,
    default_headers: HashMap<String, String>,
    timeout: Duration,
    max_retries: u32,
    retry_delay: Duration,
}

impl Client {
    pub fn new() -> Self {
        Self {
            socket: TcpSocket::create().unwrap(),
            tls_read_buffer: vec![0; 4096 * 4],
            tls_write_buffer: vec![0; 4096 * 4],
            dns: Dns::new(),
            base_url: String::new(),
            default_headers: HashMap::new(),
            timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
        }
    }

    fn client(&mut self) -> HttpClient<'_, TcpSocket, Dns> {
        HttpClient::new_with_tls(
            &self.socket,
            &self.dns,
            TlsConfig::new(
                0,
                &mut self.tls_read_buffer,
                &mut self.tls_write_buffer,
                reqwless::client::TlsVerify::None,
            ),
        )
    }

    pub async fn request(
        &mut self,
        method: Method,
        url: String,
        config: RequestConfig,
    ) -> Result<Response, Box<dyn StdError>> {
        let body_stream = try_fn_stream(|emitter| async move {
            let mut client = self.client();
            let request = client.request(Method::GET, &url);
            let mut request = request.await?;
            let mut header_buf = [0; 4096 * 2];
            let response = request.send(&mut header_buf).await?;

            emitter
                .emit(ResponseData::Metadata(ResponseMetadata {
                    status_code: response.status.0,
                    headers: response
                        .headers()
                        .map(|(k, v)| (k.to_string(), String::from_utf8_lossy(v).to_string()))
                        .collect(),
                    url: url.clone(),
                    method,
                    request_config: config,
                }))
                .await;

            let mut body_reader = response.body().reader();
            let mut body_buf = [0; 512];
            loop {
                let bytes_read = body_reader.read(&mut body_buf).await?;
                if bytes_read == 0 {
                    break;
                }
                emitter
                    .emit(ResponseData::Stream(ResponseChunk {
                        data: Bytes::copy_from_slice(&body_buf[..bytes_read]),
                    }))
                    .await;
            }

            Ok(())
        });

        let response = Response::new(body_stream).await?;
        Ok(response)
    }

    pub async fn get(&mut self, url: &str) -> Result<Response, Box<dyn StdError>> {
        self.request(Method::GET, url.to_string(), RequestConfig::default())
            .await
    }
}
