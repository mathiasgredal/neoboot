use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use serde_json::Value;

#[derive(Clone, Debug)]
pub enum RequestBody {
    Json(Value),
    Data(Bytes),
}

#[derive(Clone, Debug)]
pub struct RequestConfig {
    pub timeout: Duration,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub body: Option<RequestBody>,
}

impl RequestConfig {
    pub fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            headers: HashMap::new(),
            params: HashMap::new(),
            body: None,
        }
    }

    pub fn with_body(mut self, body: RequestBody) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = params;
        self
    }
}
