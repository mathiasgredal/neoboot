use crate::asyncio::tcp;
use crate::executor::Executor;
use crate::lwip_error::LwipError;
use crate::{executor, ffi};
use bytes::{BufMut, Bytes};
use futures::io::{AsyncRead, AsyncWrite};
use futures_lite::{AsyncReadExt, AsyncWriteExt, FutureExt};
use futures_rustls::TlsConnector;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use log::info;
use rustls_pki_types::DnsName;
use rustls_rustcrypto::provider;

use std::future::Future;
use std::sync::Arc;
use std::task::Poll;
use std::vec;

use crate::asyncio::socket::Socket;

struct TcpStream {
    socket: Socket,
}

impl AsyncRead for TcpStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        mut buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        unsafe { ffi::env_net_rx() };
        // let mut read_buf_len = 512;
        // if buf.remaining_mut() < read_buf_len {
        //     read_buf_len = buf.remaining_mut();
        // }
        // let mut read_buf = vec![0; read_buf_len];
        let read_bytes = unsafe {
            ffi::env_net_socket_read(
                self.socket.inner.borrow().socket,
                buf.as_mut_ptr() as *mut u8,
                buf.remaining_mut() as u32,
            )
        };

        if read_bytes == LwipError::WouldBlock.to_code() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if read_bytes < 0 {
            return Poll::Ready(Err(LwipError::from_code(read_bytes).into()));
        }
        info!("poll_read: read_bytes = {}", read_bytes);
        unsafe { buf.advance_mut(read_bytes as usize) };
        Poll::Ready(Ok(read_bytes as usize))
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        info!("poll_write: len = {}", buf.len());
        unsafe { ffi::env_net_rx() };
        let write_bytes = unsafe {
            ffi::env_net_socket_write(
                self.socket.inner.borrow().socket,
                buf.as_ptr() as *const u8,
                buf.len() as u32,
            )
        };

        if write_bytes < 0 {
            return Poll::Ready(Err(LwipError::from_code(write_bytes).into()));
        }

        return Poll::Ready(Ok(buf.len() as usize));
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        unsafe { ffi::env_net_rx() };
        let err = unsafe { ffi::env_net_socket_write_poll(self.socket.inner.borrow().socket) };

        if err == LwipError::WouldBlock.to_code() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if err == LwipError::Ok.to_code() {
            return Poll::Ready(Ok(()));
        }

        return Poll::Ready(Err(LwipError::from_code(err).into()));
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.socket = tcp::TcpSocket::create().unwrap().socket;
        Poll::Ready(Ok(()))
    }
}

struct HttpStream {
    socket: Socket,
}

impl hyper::rt::Read for HttpStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        unsafe { ffi::env_net_rx() };
        let mut read_buf_len = 512;
        if buf.remaining() < read_buf_len {
            read_buf_len = buf.remaining();
        }
        let mut read_buf = vec![0; read_buf_len];
        let read_bytes = unsafe {
            ffi::env_net_socket_read(
                self.socket.inner.borrow().socket,
                read_buf.as_mut_ptr() as *mut u8,
                read_buf_len as u32,
            )
        };

        if read_bytes == LwipError::WouldBlock.to_code() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if read_bytes < 0 {
            return Poll::Ready(Err(LwipError::from_code(read_bytes).into()));
        }

        buf.put_slice(&read_buf[..read_bytes as usize]);
        Poll::Ready(Ok(()))
    }
}

impl hyper::rt::Write for HttpStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let write_bytes = unsafe {
            ffi::env_net_socket_write(
                self.socket.inner.borrow().socket,
                buf.as_ptr() as *const u8,
                buf.len() as u32,
            )
        };

        if write_bytes < 0 {
            return Poll::Ready(Err(LwipError::from_code(write_bytes).into()));
        }

        return Poll::Ready(Ok(buf.len() as usize));
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        unsafe { ffi::env_net_rx() };
        let err = unsafe { ffi::env_net_socket_write_poll(self.socket.inner.borrow().socket) };

        if err == LwipError::WouldBlock.to_code() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if err == LwipError::Ok.to_code() {
            return Poll::Ready(Ok(()));
        }

        return Poll::Ready(Err(LwipError::from_code(err).into()));
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl<Fut> hyper::rt::Executor<Fut> for Executor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        self.spawn(async move {
            fut.await;
        });
    }
}

pub async fn test_hyper(executor: Executor) {
    let url = "http://httpbin.org/get".parse::<hyper::Uri>().unwrap();

    let mut socket = tcp::TcpSocket::create().unwrap();

    socket.connect("52.22.198.150", 80).await.unwrap();

    let stream = HttpStream {
        socket: socket.socket,
    };

    // Create the Hyper client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(stream).await.unwrap();

    info!("Handshake complete");
    // Spawn a task to poll the connection, driving the HTTP state
    executor.spawn(async move {
        if let Err(err) = conn.await {
            info!("Connection failed: {:?}", err);
        }
    });
    info!("Connected");

    // The authority of our URL will be the hostname of the httpbin remote
    let authority = url.authority().unwrap().clone();

    // Create an HTTP request with an empty body and a HOST header
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())
        .unwrap();

    // Await the response...
    info!("Sending request");

    let mut res = sender.send_request(req).await.unwrap();

    info!("Response status: {}", res.status());

    // Stream the body, writing each frame to stdout as it arrives
    while let Some(next) = res.frame().await {
        let frame = next.unwrap();
        if let Some(chunk) = frame.data_ref() {
            info!("Chunk: {}", String::from_utf8_lossy(chunk));
        }
    }
}

pub async fn test_tls() {
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = rustls::ClientConfig::builder_with_provider(provider().into())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let config = TlsConnector::from(Arc::new(config));

    let mut socket = tcp::TcpSocket::create().unwrap();

    socket.connect("23.192.228.84", 443).await.unwrap();

    let stream = TcpStream {
        socket: socket.socket,
    };
    // let dnsname = DNSNameRef::try_from_ascii_str("example.com").unwrap();
    info!("Connecting...");
    let dnsname = DnsName::try_from_str("example.com").unwrap();
    let mut stream = config
        .connect(rustls_pki_types::ServerName::DnsName(dnsname), stream)
        .await
        .unwrap();

    info!("Connected");

    stream
        .write_all(
            concat!(
                "GET / HTTP/1.1\r\n",
                "Host: example.com\r\n",
                "Connection: close\r\n",
                "Accept-Encoding: identity\r\n",
                "\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let ciphersuite = stream.get_mut().1.negotiated_cipher_suite().unwrap();
    info!("Current ciphersuite: {:?}", ciphersuite.suite());
    let mut plaintext = Vec::new();
    stream.read_to_end(&mut plaintext).await.unwrap();
    info!("Plaintext: {:?}", plaintext);
}





fn mainloop_2(executor: Executor) {
    executor.clone().spawn(async move {
        let socket = TcpSocket::create();
        if socket.is_err() {
            log::error!("Failed to create socket: {}", socket.err().unwrap());
            return;
        }

        let socket = socket.unwrap();

        let result = socket.bind("0.0.0.0", 8080);
        if result.is_err() {
            log::error!("Failed to bind to socket: {}", result.err().unwrap());
            return;
        }

        let result = socket.listen(10);
        if result.is_err() {
            log::error!("Failed to listen on socket: {}", result.err().unwrap());
            return;
        }

        loop {
            let result = socket.accept().await;
            if result.is_err() {
                log::error!("Failed to accept connection: {}", result.err().unwrap());
                return;
            }

            let client_socket = result.unwrap();

            let result = client_socket.socket.read(1024).await;
            if result.is_err() {
                log::error!("Failed to read from socket: {}", result.err().unwrap());
                return;
            }

            let buf = result.unwrap();
            log::info!("Read {} bytes: {:?}", buf.len(), buf);
        }
    });
}

fn mainloop_3(executor: Executor) {
    executor.clone().spawn(async move {
        let mut socket = TcpSocket::create();
        if socket.is_err() {
            log::error!("Failed to create socket: {}", socket.err().unwrap());
            return;
        }

        let mut socket = socket.unwrap();

        let result = socket.connect_raw("10.0.2.2", 8081).await;
        if result.is_err() {
            log::error!("Failed to connect to socket: {}", result.err().unwrap());
            return;
        }

        loop {
            let result = socket.socket.read(1024).await;
            if result.is_err() {
                log::error!("Failed to read from socket: {}", result.err().unwrap());
                return;
            }

            let buf = result.unwrap();
            log::info!("Read {} bytes", buf.len());
        }

        // let mut client = Client::new();

        // let result = client.get("http://192.168.1.120:8081/large.bin").await;

        // match result {
        //     Ok(response) => {
        //         let mut stream = response.stream().await;
        //         while let Some(result) = stream.next().await {
        //             let chunk = result.unwrap();
        //             log::info!("Chunk: {}", chunk.len());
        //         }
        //     }
        //     Err(e) => {
        //         log::error!("Failed to get from socket: {:?}", e);
        //     }
        // }
    });
}




mod asyncio;
mod ffi;
mod logging;
mod lwip_error;
mod panic;
mod util;
use asyncio::sleep_ms;
use core::cell::Cell;
use log::info;
use logging::init_with_level;
use simple_async_local_executor::Executor;
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{self, Device, DeviceCapabilities, Loopback, Medium};
use smoltcp::socket::tcp;
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};
use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::{cell::RefCell, rc::Rc};

type PhyFrame = [u8; 1514];
type PhyQueue = VecDeque<PhyFrame>;

#[derive(Debug)]
pub struct UBootEthernet {
    pub(crate) queue: PhyQueue,
    medium: Medium,
}

impl UBootEthernet {
    pub fn new(medium: Medium) -> UBootEthernet {
        UBootEthernet {
            queue: VecDeque::new(),
            medium,
        }
    }
}

// Following this guide: https://github.com/mars-research/redleaf/blob/7194295d1968c8013ae6b3d104a9192f03516449/domains/lib/smolnet/src/lib.rs
// Also check out stm32-eth 
impl Device for UBootEthernet {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut capabilities = DeviceCapabilities::default();
        capabilities.medium = self.medium;
        capabilities.max_transmission_unit = 1500;
        capabilities
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        self.queue.pop_front().map(move |buffer| {
            let rx = RxToken { buffer };
            let tx = TxToken {
                queue: &mut self.queue,
            };
            (rx, tx)
        })
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            queue: &mut self.queue,
        })
    }
}

pub struct RxToken {
    buffer: Vec<u8>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buffer)
    }
}

#[derive(Debug)]
pub struct TxToken<'a> {
    queue: &'a mut VecDeque<Vec<u8>>,
}

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        self.queue.push_back(buffer);
        result
    }
}

#[derive(Clone)]
pub struct Clock(Cell<Instant>);

impl Clock {
    pub fn new() -> Clock {
        Clock(Cell::new(Instant::from_millis(unsafe {
            ffi::env_now() as i64
        })))
    }

    pub fn advance(&self, duration: Duration) {
        self.0.set(self.0.get() + duration)
    }

    pub fn elapsed(&self) -> Instant {
        self.0.get()
    }
}

fn mainloop(executor: Executor) {
    executor.clone().spawn(async move {
        let clock = Clock::new();
        let mut device = Loopback::new(Medium::Ethernet);

        // Create interface
        let config = Config::new(smoltcp::wire::HardwareAddress::Ethernet(EthernetAddress([
            0x02, 0x00, 0x00, 0x00, 0x00, 0x01,
        ])));

        let mut iface = Interface::new(config, &mut device, Instant::now());
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs
                .push(IpCidr::new(IpAddress::v4(10, 0, 2, 15), 24))
                .unwrap();
        });

        iface
            .routes_mut()
            .add_default_ipv4_route(Ipv4Addr::new(10, 0, 2, 1))
            .unwrap();

        let client_socket = {
            static mut TCP_CLIENT_RX_DATA: [u8; 1024] = [0; 1024];
            static mut TCP_CLIENT_TX_DATA: [u8; 1024] = [0; 1024];
            let tcp_rx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_CLIENT_RX_DATA[..] });
            let tcp_tx_buffer = tcp::SocketBuffer::new(unsafe { &mut TCP_CLIENT_TX_DATA[..] });
            tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
        };

        let start = clock.elapsed();

        let mut sockets: [_; 2] = Default::default();
        let mut socket_set = SocketSet::new(&mut sockets[..]);
        let client_handle = socket_set.add(client_socket);
        let mut did_connect = false;

        while clock.elapsed() - start < Duration::from_secs(10) {
            iface.poll(clock.elapsed(), &mut device, &mut socket_set);
            let socket = socket_set.get_mut::<tcp::Socket>(client_handle);

            let cx = iface.context();
            if !socket.is_open() {
                if !did_connect {
                    info!("connecting");
                    socket
                        .connect(cx, (IpAddress::v4(192, 168, 1, 120), 8081), 65000)
                        .unwrap();
                    did_connect = true;
                }
            }

            if socket.can_recv() {
                info!(
                    "got {:?}",
                    socket.recv(|buffer| { (buffer.len(), String::from_utf8_lossy(buffer)) })
                );
            }
        }
    });
}



// fn mainloop_3(executor: Executor) {
//     executor.spawn(async move {
//         let socket = TcpSocket::create();
//         if socket.is_err() {
//             log::error!("Failed to create socket: {}", socket.err().unwrap());
//             return;
//         }

//         let mut socket = socket.unwrap();

//         let result = socket.connect("10.0.2.2", 8081).await;
//         if result.is_err() {
//             log::error!("Failed to connect to socket: {}", result.err().unwrap());
//             return;
//         }

//         return;

//         let mut numBytes = 0;
//         let mut last_print = 0;

//         loop {
//             // let result = socket.socket.write(b"Hello, world!").await;
//             // if result.is_err() {
//             //     log::error!("Failed to write to socket: {}", result.err().unwrap());
//             //     return;
//             // }

//             let result = socket.socket.read(512).await;
//             if result.is_err() {
//                 log::error!("Failed to read from socket: {}", result.err().unwrap());
//                 return;
//             }

//             let buf = result.unwrap();
//             numBytes += buf.len();

//             if numBytes - last_print > 10000 {
//                 last_print = numBytes;
//                 log::info!("Read {} bytes", numBytes);
//             }
//         }
//     });
// }