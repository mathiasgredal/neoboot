use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use serde_json::Value;

pub struct RequestConfig {
    timeout: Duration,
    headers: HashMap<String, String>,
    params: HashMap<String, String>,
    json: Option<Value>,
    data: Option<Bytes>,
    auth: Option<(String, String)>,
    allow_redirects: bool,
    max_redirects: u32,
}

impl RequestConfig {
    pub fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            headers: HashMap::new(),
            params: HashMap::new(),
            json: None,
            data: None,
            auth: None,
            allow_redirects: true,
            max_redirects: 10,
        }
    }
}
