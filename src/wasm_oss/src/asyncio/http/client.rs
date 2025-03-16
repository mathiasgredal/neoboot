use super::request::RequestBody;
use super::request::RequestConfig;
use super::response::Response;
use super::response::ResponseChunk;
use super::response::ResponseData;
use super::response::ResponseMetadata;
use super::stream::AnyHttpStream;
use super::timeout::setup_timeout_task;
use super::timeout::timeout_with_controller;
use super::timeout::TimeoutController;
use super::tls::create_tls_connector;
use crate::asyncio::dns::GLOBAL_DNS_RESOLVER;
use crate::asyncio::net::TcpStream;
use crate::executor::Executor;
use async_fn_stream::try_fn_stream;
use bytes::Bytes;
use http::Method;
use http::Request;
use http_body_util::BodyExt;
use log::warn;
use rustls_pki_types::DnsName;
use std::collections::HashMap;
use url::Url;

pub struct Client {
    executor: Executor,
    base_url: String,
    default_headers: HashMap<String, String>,
}

impl Client {
    pub fn new(executor: Executor) -> Self {
        Self {
            executor,
            base_url: String::new(),
            default_headers: HashMap::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    fn build_full_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else if self.base_url.is_empty() {
            format!("https://{}", url)
        } else {
            let base = self.base_url.trim_end_matches('/');
            let path = url.trim_start_matches('/');
            format!("{}/{}", base, path)
        }
    }

    async fn perform_request(
        &mut self,
        method: Method,
        url: String,
        config: RequestConfig,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let body_stream = try_fn_stream(|emitter| {
            let method_clone = method.clone();
            let config_clone = config.clone();
            let executor_clone = self.executor.clone();
            let timeout_ms = config_clone.timeout.as_millis() as u64;

            async move {
                let url = url.parse::<hyper::Uri>()?;
                let is_https = url.scheme_str() == Some("https");

                // Create timeout controller for connection phase
                let conn_timeout_controller = TimeoutController::new(timeout_ms);
                setup_timeout_task(&executor_clone, conn_timeout_controller.clone()).await;

                // DNS resolution
                let host = url.host().ok_or("Missing host in URL")?;

                let ip = match GLOBAL_DNS_RESOLVER.get_host_by_name(host).await {
                    Ok(ip) => ip,
                    Err(e) => return Err(format!("DNS resolution failed: {}", e).into()),
                };

                // Determine port
                let port =
                    url.port()
                        .map(|p| p.as_u16())
                        .unwrap_or(if is_https { 443 } else { 80 });

                // Connect to server
                let tcp_stream = match timeout_with_controller(
                    conn_timeout_controller.clone(),
                    TcpStream::connect(ip.to_string().as_str(), port),
                )
                .await
                {
                    Ok(stream) => stream?,
                    Err(e) => return Err(format!("Connection failed: {}", e).into()),
                };

                // Set up HTTP or HTTPS stream
                let mut stream = AnyHttpStream::Http(tcp_stream.clone());
                if is_https {
                    let host_str = String::from(host);
                    let dnsname = DnsName::try_from_str(&host_str)?;
                    let server_name = rustls_pki_types::ServerName::DnsName(dnsname.to_owned());

                    let connector = create_tls_connector();
                    match timeout_with_controller(
                        conn_timeout_controller.clone(),
                        connector.connect(server_name, tcp_stream.clone()),
                    )
                    .await
                    {
                        Ok(Ok(tls_stream)) => stream = AnyHttpStream::Https(tls_stream),
                        Ok(Err(e)) => return Err(format!("TLS handshake failed: {}", e).into()),
                        Err(_) => return Err("TLS handshake timed out".into()),
                    }
                }

                // HTTP handshake
                let (mut sender, conn) = match timeout_with_controller(
                    conn_timeout_controller.clone(),
                    hyper::client::conn::http1::handshake(stream),
                )
                .await
                {
                    Ok(result) => result?,
                    Err(_) => return Err("HTTP handshake timed out".into()),
                };

                // Spawn connection handler
                self.executor.spawn(async move {
                    if let Err(err) = conn.await {
                        warn!("Connection failed: {:?}", err);
                    }
                });

                // Build request
                let authority = url.authority().ok_or("Missing authority in URL")?.clone();
                let mut req_builder = Request::builder()
                    .method(method_clone.clone())
                    .uri(url.clone())
                    .header(hyper::header::HOST, authority.as_str());

                // Add headers
                for (key, value) in &self.default_headers {
                    req_builder = req_builder.header(key, value);
                }

                for (key, value) in &config_clone.headers {
                    req_builder = req_builder.header(key, value);
                }

                // Add body if present
                let req = if let Some(body) = &config_clone.body {
                    match body {
                        RequestBody::Json(json) => {
                            req_builder = req_builder
                                .header(hyper::header::CONTENT_TYPE, "application/json")
                                .header(
                                    hyper::header::CONTENT_LENGTH,
                                    json.to_string().len().to_string(),
                                );
                            req_builder.body(http_body_util::Full::new(json.to_string().into()))?
                        }
                        RequestBody::Data(data) => {
                            req_builder = req_builder
                                .header(hyper::header::CONTENT_TYPE, "application/octet-stream")
                                .header(hyper::header::CONTENT_LENGTH, data.len().to_string());
                            req_builder.body(http_body_util::Full::new(data.clone()))?
                        }
                    }
                } else {
                    req_builder.body(http_body_util::Full::new(Bytes::new()))?
                };

                // Send request
                let mut res = match timeout_with_controller(
                    conn_timeout_controller.clone(),
                    sender.send_request(req),
                )
                .await
                {
                    Ok(res) => res?,
                    Err(_) => return Err("Request timed out".into()),
                };

                // Emit metadata
                emitter
                    .emit(ResponseData::Metadata(ResponseMetadata {
                        status_code: res.status().into(),
                        headers: res
                            .headers()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect(),
                        url: url.to_string(),
                        method: method_clone,
                        request_config: config_clone,
                    }))
                    .await;

                // Process response body
                let body_timeout_controller = TimeoutController::new(timeout_ms);
                setup_timeout_task(&executor_clone, body_timeout_controller.clone()).await;

                // Process response body with reusable timeout
                loop {
                    // Reset the timeout for each frame
                    let frame_result =
                        match timeout_with_controller(body_timeout_controller.clone(), res.frame())
                            .await
                        {
                            Ok(result) => result,
                            Err(_) => return Err("Response body read timed out".into()),
                        };

                    match frame_result {
                        Some(Ok(frame)) => {
                            if let Some(chunk) = frame.data_ref() {
                                emitter
                                    .emit(ResponseData::Stream(ResponseChunk {
                                        data: chunk.clone(),
                                    }))
                                    .await;
                            }
                        }
                        Some(Err(e)) => {
                            warn!("Error reading frame: {:?}", e);
                            break;
                        }
                        None => break, // End of stream
                    }
                }

                Ok(())
            }
        });

        match Response::new(body_stream).await {
            Ok(response) => Ok(response),
            Err(e) => Err(e),
        }
    }

    pub async fn request(
        &mut self,
        method: Method,
        url: impl AsRef<str>,
        config: RequestConfig,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let full_url = self.build_full_url(url.as_ref());
        let full_url = self.add_params_to_url(&full_url, &config.params);
        self.perform_request(method, full_url, config).await
    }

    fn add_params_to_url(&self, url: &str, params: &HashMap<String, String>) -> String {
        let mut url = Url::parse(url).unwrap();
        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, value);
        }
        url.to_string()
    }
}
