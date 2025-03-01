use crate::asyncio::socket::Socket;
use crate::asyncio::tcp;
use crate::executor::Executor;
use crate::ffi;
use crate::lwip_error::LwipError;
use crate::tls::create_tls_connector;
use bytes::{BufMut, Bytes};
use futures::io::{AsyncRead, AsyncWrite};
use futures_rustls::client::TlsStream;
use http_body_util::{BodyExt, Empty};
use hyper::{
    rt::{self},
    Request,
};
// use hyper_util::client::legacy::connect::{Connected, Connection};
use log::info;
use rustls_pki_types::DnsName;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

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

        if read_bytes == 0 {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if read_bytes < 0 {
            return Poll::Ready(Err(LwipError::from_code(read_bytes).into()));
        }

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

pub enum AnyHttpStream<T>
where
    T: AsyncRead + AsyncWrite,
{
    Http(T),
    Https(TlsStream<T>),
}

impl<T: AsyncRead + AsyncWrite> From<T> for AnyHttpStream<T> {
    fn from(inner: T) -> Self {
        Self::Http(inner)
    }
}

impl<T: AsyncRead + AsyncWrite> From<TlsStream<T>> for AnyHttpStream<T> {
    fn from(inner: TlsStream<T>) -> Self {
        Self::Https(inner)
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> rt::Read for AnyHttpStream<T> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        mut buf: rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let mut buf_size = 512;
        if buf_size > buf.remaining() {
            buf_size = buf.remaining();
        }
        let mut ibuf = vec![0u8; buf_size];

        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_read(cx, &mut ibuf).map_ok(|n| {
                    buf.put_slice(&ibuf[..n]);
                    ()
                })
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_read(cx, &mut ibuf).map_ok(|n| {
                    buf.put_slice(&ibuf[..n]);
                    ()
                })
            }
        }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> rt::Write for AnyHttpStream<T> {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_write(cx, buf)
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_write(cx, buf)
            }
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_flush(cx)
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_flush(cx)
            }
        }
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match Pin::get_mut(self) {
            Self::Http(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_close(cx)
            }
            Self::Https(s) => {
                let pinned = std::pin::pin!(s);
                pinned.poll_close(cx)
            }
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        return false;
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        todo!()
    }
}

pub async fn test_hyper_tls(executor: Executor) {
    let url = "https://stackoverflow.com/questions/40557031/command-prompt-to-check-tls-version-required-by-a-host".parse::<hyper::Uri>().unwrap();

    let connector = create_tls_connector();

    let mut socket = tcp::TcpSocket::create().unwrap();
    socket.connect("172.64.155.249", 443).await.unwrap();

    let dnsname = DnsName::try_from_str("stackoverflow.com").unwrap();

    let stream = TcpStream {
        socket: socket.socket,
    };

    info!("Connecting tls");
    let stream = connector
        .connect(rustls_pki_types::ServerName::DnsName(dnsname), stream)
        .await
        .unwrap();
    info!("Connected tls");

    let stream = AnyHttpStream::Https(stream);

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
        .method("GET")
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .header("User-Agent", "curl/7.64.1")
        .header("Connection", "keep-alive")
        .body(Empty::<Bytes>::new())
        .unwrap();

    // Await the response...
    info!("Sending request");

    let mut res = sender.send_request(req).await.unwrap();

    info!("Response status: {}", res.status());

    // let ciphersuite = stream.get_mut().1.negotiated_cipher_suite().unwrap();
    // info!("Current ciphersuite: {:?}", ciphersuite.suite());

    // Stream the body, writing each frame as it arrives
    while let Some(next) = res.frame().await {
        if next.is_err() {
            info!("Error: {:?}", next.err());
            break;
        }
        let frame = next.unwrap();
        if let Some(chunk) = frame.data_ref() {
            info!("Chunk: {}", String::from_utf8_lossy(chunk));
            // info!("Chunk length: {}", chunk.len())
        }
    }
}
