use crate::errors::lwip_error::LwipError;
use crate::ffi;
use crate::util::ip_addr_to_u32;
use bytes::BufMut;
use futures::{AsyncRead, AsyncWrite};
use log::{error, info};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

// region: Socket
struct SocketInner {
    socket: i32,
}

impl Drop for SocketInner {
    fn drop(&mut self) {
        info!("Closing socket: {}", self.socket);
        let result = unsafe { ffi::env_net_socket_free(self.socket) };
        if result != LwipError::Ok.to_code() {
            error!("Failed to close socket: {}", LwipError::from_code(result));
        }
    }
}

#[derive(Clone)]
struct Socket {
    inner: Rc<RefCell<SocketInner>>,
}

unsafe impl Send for Socket {}
unsafe impl Sync for Socket {}

impl Socket {
    fn create_tcp() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_net_socket_new_tcp() };
        if socket < 0 {
            return Err(LwipError::from_code(socket));
        }
        info!("Creating TCP socket: {}", socket);
        Ok(Socket {
            inner: Rc::new(RefCell::new(SocketInner { socket })),
        })
    }

    fn create_udp() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_net_socket_new_udp() };
        if socket < 0 {
            return Err(LwipError::from_code(socket));
        }
        Ok(Socket {
            inner: Rc::new(RefCell::new(SocketInner { socket })),
        })
    }
}

impl AsyncRead for Socket {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        mut buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        unsafe { ffi::env_net_rx() };
        let read_bytes = unsafe {
            ffi::env_net_socket_read(
                self.inner.borrow().socket,
                buf.as_mut_ptr(),
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

impl AsyncWrite for Socket {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        unsafe { ffi::env_net_rx() };
        let write_bytes = unsafe {
            ffi::env_net_socket_write(self.inner.borrow().socket, buf.as_ptr(), buf.len() as u32)
        };

        if write_bytes < 0 {
            return Poll::Ready(Err(LwipError::from_code(write_bytes).into()));
        }

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        unsafe { ffi::env_net_rx() };
        let err = unsafe { ffi::env_net_socket_write_poll(self.inner.borrow().socket) };

        if err == LwipError::WouldBlock.to_code() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if err == LwipError::Ok.to_code() {
            return Poll::Ready(Ok(()));
        }

        Poll::Ready(Err(LwipError::from_code(err).into()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        drop(self.inner.borrow_mut());
        Poll::Ready(Ok(()))
    }
}

// endregion: Socket

// region: TCP
pub struct TcpListener {
    socket: Socket,
}

impl TcpListener {
    pub fn bind(addr_str: &str, port: u16) -> Result<Self, LwipError> {
        let socket = Socket::create_tcp()?;
        let addr = ip_addr_to_u32(addr_str)?;
        let result =
            unsafe { ffi::env_net_socket_bind(socket.inner.borrow().socket, addr, port.into()) };
        if result != LwipError::Ok.to_code() {
            info!("Failed to bind TCP listener: {}", result);
            return Err(LwipError::from_code(result));
        }

        let result = unsafe { ffi::env_net_socket_listen(socket.inner.borrow().socket, 8) };
        if result != LwipError::Ok.to_code() {
            info!("Failed to listen on TCP listener: {}", result);
            return Err(LwipError::from_code(result));
        }

        Ok(Self { socket })
    }

    pub async fn accept(&self) -> Result<TcpStream, LwipError> {
        struct TcpAccept {
            socket: Socket,
        }

        impl Future for TcpAccept {
            type Output = Result<TcpStream, LwipError>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };
                let result =
                    unsafe { ffi::env_net_socket_accept_poll(self.socket.inner.borrow().socket) };

                if result == LwipError::WouldBlock.to_code() {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }

                if result < 0 {
                    return Poll::Ready(Err(LwipError::from_code(result)));
                }

                Poll::Ready(Ok(TcpStream {
                    socket: Socket {
                        inner: Rc::new(RefCell::new(SocketInner { socket: result })),
                    },
                }))
            }
        }

        let result = unsafe { ffi::env_net_socket_accept(self.socket.inner.borrow().socket) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        let accept_result = TcpAccept {
            socket: self.socket.clone(),
        }
        .await;

        if accept_result.is_err() {
            return Err(accept_result.err().unwrap());
        }

        Ok(accept_result.unwrap())
    }
}

#[derive(Clone)]
pub struct TcpStream {
    socket: Socket,
}

impl TcpStream {
    pub async fn connect(ip: &str, port: u16) -> Result<Self, LwipError> {
        struct TcpConnection {
            socket: i32,
        }

        impl Future for TcpConnection {
            type Output = Result<(), LwipError>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };

                let err = unsafe { ffi::env_net_socket_connect_poll(self.socket) };

                if err == LwipError::WouldBlock.to_code() {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }

                if err == LwipError::Ok.to_code() {
                    return Poll::Ready(Ok(()));
                }

                log::error!("Failed to connect to socket poll: {}", err);
                Poll::Ready(Err(LwipError::from_code(err)))
            }
        }

        let socket = Socket::create_tcp();

        if socket.is_err() {
            return Err(socket.err().unwrap());
        }

        let socket = socket.unwrap();
        let addr = ip_addr_to_u32(ip)?;
        let result: i32 =
            unsafe { ffi::env_net_socket_connect(socket.inner.borrow().socket, addr, port.into()) };

        if result != LwipError::Ok.to_code() {
            log::error!("Failed to connect to socket: {}", result);
            return Err(LwipError::from_code(result));
        }

        let result = TcpConnection {
            socket: socket.inner.borrow().socket,
        }
        .await;

        if result.is_err() {
            return Err(result.err().unwrap());
        }

        Ok(Self { socket })
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_close(cx)
    }
}
// endregion: TCP

// region: UDP

struct UdpSocket {
    socket: Socket,
}

impl UdpSocket {
    pub fn bind(addr_str: &str, port: u16) -> Result<Self, LwipError> {
        let socket = Socket::create_udp()?;
        let addr = ip_addr_to_u32(addr_str)?;
        let result =
            unsafe { ffi::env_net_socket_bind(socket.inner.borrow().socket, addr, port.into()) };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        Ok(Self { socket })
    }

    pub fn connect(self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result = unsafe {
            ffi::env_net_socket_connect(self.socket.inner.borrow().socket, addr, port.into())
        };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        Ok(())
    }
}

impl AsyncRead for UdpSocket {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_read(cx, buf)
    }
}

impl AsyncWrite for UdpSocket {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pinned = std::pin::pin!(self.socket.clone());
        pinned.poll_close(cx)
    }
}

// endregion: UDP
