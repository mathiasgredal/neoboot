use crate::asyncio::{sleep_ms, tcp};
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
use rustls::client::danger::ServerCertVerifier;
use rustls::server::NoClientAuth;
use rustls::ClientConfig;
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

        info!("poll_read: buf_len = {}", buf.remaining_mut());
        info!("poll_read: read_bytes = {}", read_bytes);

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

#[derive(Debug)]
struct NoVerifyCert {}

impl rustls::client::danger::ServerCertVerifier for NoVerifyCert {
    fn verify_server_cert(
        &self,
        end_entity: &rustls_pki_types::CertificateDer<'_>,
        intermediates: &[rustls_pki_types::CertificateDer<'_>],
        server_name: &rustls_pki_types::ServerName<'_>,
        ocsp_response: &[u8],
        now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls_pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

pub fn create_tls_connector() -> TlsConnector {
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut config = rustls::ClientConfig::builder_with_provider(provider().into())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let mut config = rustls::ClientConfig::dangerous(&mut config);
    config.set_certificate_verifier(Arc::new(NoVerifyCert {}));
    let client_config: ClientConfig = config.cfg.clone();
    let connector = TlsConnector::from(Arc::new(client_config));
    return connector;
}

pub async fn test_tls() {
    let connector = create_tls_connector();
    let mut socket = tcp::TcpSocket::create().unwrap();

    socket.connect("172.64.155.249", 443).await.unwrap();

    let stream = TcpStream {
        socket: socket.socket,
    };
    // let dnsname = DNSNameRef::try_from_ascii_str("example.com").unwrap();
    info!("Connecting...");
    let dnsname = DnsName::try_from_str("stackoverflow.com").unwrap();
    let mut stream = connector
        .connect(rustls_pki_types::ServerName::DnsName(dnsname), stream)
        .await
        .unwrap();

    info!("Connected");

    stream
        .write_all(
            concat!(
                "GET /questions/40557031/command-prompt-to-check-tls-version-required-by-a-host HTTP/1.1\r\n",
                "Host: stackoverflow.com\r\n",
                "User-Agent: curl/7.64.1\r\n",
                "Accept: */*\r\n",
                "Connection: keep-alive\r\n",
                "Accept-Encoding: en-GB,en;q=0.9\r\n",
                "\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let ciphersuite = stream.get_mut().1.negotiated_cipher_suite().unwrap();
    info!("Current ciphersuite: {:?}", ciphersuite.suite());

    let mut body_buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut body_buf).await;
        info!("Bytes read: {:?}", bytes_read);
        if bytes_read.is_ok() {
            let bytes_read = bytes_read.unwrap();
            if bytes_read == 0 {
                break;
            }
            info!(
                "Chunk: {}",
                String::from_utf8_lossy(&body_buf[..bytes_read])
            );
        }
    }

    // let mut plaintext = Vec::new();
    // stream.read_to_end(&mut plaintext).await.unwrap();
    // info!("Plaintext: {:?}", String::from_utf8_lossy(&plaintext));
}
