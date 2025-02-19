// use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// use super::{dns::Dns, tcp::TcpSocket, udp::UdpSocket};

// use embedded_io_async::{BufRead, Read};
// use log::info;
// use reqwless::{headers::TransferEncoding, request::Method, response::BodyReader, Error};

// pub struct HttpClient {
//     socket: TcpSocket,
//     dns: Dns<UdpSocket>,
// }

// impl HttpClient {
//     pub fn new() -> Self {
//         let socket = TcpSocket::create().unwrap();
//         let nameserver = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
//         let udp_socket = UdpSocket::create().unwrap();
//         let dns = Dns::new(udp_socket, nameserver);
//         Self { socket, dns }
//     }

//     fn client(&self) -> reqwless::client::HttpClient<'_, TcpSocket, Dns<UdpSocket>> {
//         reqwless::client::HttpClient::new(&self.socket, &self.dns)
//     }

//     pub async fn get(&self, url: &str) -> Result<String, Error> {
//         let mut client = self.client();
//         let mut request = client.request(Method::GET, url).await?;
//         let mut header_buf = [0; 4096];
//         let response = request.send(&mut header_buf).await?;
//         let mut body = vec![];
//         let mut body_buf = [0; 512];
//         let mut body_reader = response.body().reader();
//         loop {
//             let buf: usize = body_reader.read(&mut body_buf).await?;
//             if buf == 0 {
//                 break;
//             }
//             body.extend(body_buf.iter().take(buf));
//             info!("{}", String::from_utf8_lossy(&body_buf[..buf]));
//         }

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

//         Ok(String::from_utf8(body).unwrap())
//     }
// }
use super::{dns::Dns, tcp::TcpSocket, udp::UdpSocket};
use async_fn_stream::try_fn_stream;
use bytes::Bytes;
use futures_lite::Stream;
use reqwless::{
    client::HttpClient as ReqwlessHttpClient, request::Method, response::BodyReader, Error,
};
use serde_json::Value;

pub struct RequestConfig {
    timeout: Duration,
    headers: Vec<(String, String)>,
    params: Vec<(String, String)>,
    json: Option<Value>,
    data: Option<Bytes>,
    auth: Option<(String, String)>,
    allow_redirects: bool,
    max_redirects: u32,
}

pub struct Response<B> {
    status_code: u16,
    headers: Vec<(String, String)>,
    url: String,
    method: Method,
    body_reader: BodyReader<B>,
    request_config: RequestConfig,
}

impl<B> Response<B> {
    pub fn new(
        status_code: u16,
        headers: Vec<(String, String)>,
        url: String,
        method: Method,
        body_reader: BodyReader<B>,
        request_config: RequestConfig,
    ) -> Self {
        Self {
            status_code,
            headers,
            url,
            method,
            body_reader,
            request_config,
        }
    }

    pub async fn text(&mut self) -> Result<String, Error> {
        todo!()
    }

    pub async fn json(&mut self) -> Result<Value, Error> {
        todo!()
    }

    pub async fn stream(&mut self) -> Result<impl Stream<Item = Result<i32, Error>>, Error> {
        Ok(try_fn_stream(|emitter| async move {
            for i in 0..3 {
                emitter.emit(i).await;
            }

            Ok(())
        }))
    }

    pub async fn bytes(&mut self) -> Result<Bytes, Error> {
        todo!()
    }
}
pub struct HttpClient {
    socket: TcpSocket,
    dns: Dns<UdpSocket>,
    base_url: String,
    default_headers: Vec<(String, String)>,
    timeout: Duration,
    max_retries: u32,
    retry_delay: Duration,
}

impl HttpClient {
    pub fn new() -> Self {
        let socket = TcpSocket::create().unwrap();
        // TODO: get nameserver from dhcp
        let nameserver = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        let udp_socket = UdpSocket::create().unwrap();
        let dns = Dns::new(udp_socket, nameserver);
        Self { socket, dns }
    }

    fn client(&self) -> ReqwlessHttpClient<'_, TcpSocket, Dns<UdpSocket>> {
        ReqwlessHttpClient::new(&self.socket, &self.dns)
    }

    fn read_numbers() -> impl Stream<Item = Result<i32, Error>> {
        try_fn_stream(|emitter| async move {
            for i in 0..3 {
                // yield elements from stream via `emitter`
                emitter.emit(i).await;
            }

            Ok(())
        })
    }

    // pub async fn get(&self, url: &str) -> impl Stream<Item = Result<Bytes, Error>> {
    //     let url = url.to_string();
    //     try_fn_stream(|emitter| async move {
    //         let mut client = self.client();
    //         let mut request = client.request(Method::GET, &url).await?;
    //         let mut header_buf = [0; 4096];
    //         let response = request.send(&mut header_buf).await?;
    //         let mut body_reader = response.body().reader();
    //         let mut body_buf = [0; 512];
    //         loop {
    //             let bytes_read = body_reader.read(&mut body_buf).await?;
    //             if bytes_read == 0 {
    //                 break;
    //             }
    //             emitter.emit(Bytes::copy_from_slice(&body_buf[..bytes_read]));
    //         }
    //     })
    // }
}
