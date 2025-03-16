use super::request::RequestConfig;
use crate::errors::lwip_error::LwipError;
use bytes::Bytes;
use futures::StreamExt;
use futures_lite::Stream;
use http::Method;
use log::info;
use serde_json::Value;
use std::pin::Pin;

pub enum ResponseData {
    Metadata(ResponseMetadata),
    Stream(ResponseChunk),
}

#[derive(Debug)]
pub struct ResponseMetadata {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub url: String,
    pub method: Method,
    pub request_config: RequestConfig,
}

pub struct ResponseChunk {
    pub data: Bytes,
}

pub struct Response<'a> {
    pub metadata: ResponseMetadata,
    body_stream: Pin<Box<dyn Stream<Item = Result<ResponseData, Box<dyn std::error::Error>>> + 'a>>,
}

impl<'a> Response<'a> {
    pub async fn new(
        body_stream: impl Stream<Item = Result<ResponseData, Box<dyn std::error::Error>>> + 'a,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut body_stream = Box::pin(body_stream);
        let metadata = body_stream.next().await;

        if metadata.is_none() {
            return Err(Box::new(LwipError::InvalidValue));
        }

        let metadata = metadata.unwrap();

        match metadata {
            Ok(ResponseData::Metadata(metadata)) => Ok(Self {
                metadata,
                body_stream: body_stream,
            }),
            Ok(ResponseData::Stream(_)) => Err(Box::new(LwipError::InvalidValue)),
            Err(e) => {
                info!("Error: {:?}", e);
                Err(e)
            }
        }
    }

    pub async fn text(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let body = self.bytes().await?;
        let body = String::from_utf8_lossy(&body).to_string();
        Ok(body)
    }

    pub async fn json(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let body = self.text().await?;
        let json = serde_json::from_str(&body);
        match json {
            Ok(json) => Ok(json),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn stream(
        self,
    ) -> impl Stream<Item = Result<Bytes, Box<dyn std::error::Error>>> + use<'a> {
        return self.body_stream.map(|chunk| match chunk {
            Ok(ResponseData::Stream(chunk)) => Ok(chunk.data),
            Ok(ResponseData::Metadata(_)) => {
                let err: Box<dyn std::error::Error> = Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Expected stream data, got metadata",
                ));
                Err(err)
            }
            Err(e) => {
                let err: Box<dyn std::error::Error> = e;
                Err(err)
            }
        });
    }

    pub async fn bytes(&mut self) -> Result<Bytes, Box<dyn std::error::Error>> {
        let mut body_bytes = Vec::new();
        loop {
            let chunk = self.body_stream.next().await;
            if chunk.is_none() {
                break;
            } else if let Some(Err(e)) = chunk {
                return Err(e);
            } else if let Some(Ok(ResponseData::Stream(chunk))) = chunk {
                body_bytes.extend_from_slice(&chunk.data);
            }
        }
        Ok(Bytes::from(body_bytes))
    }
}
