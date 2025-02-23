use super::socket::Socket;
use crate::asyncio::socket::SocketInner;
use crate::ffi;
use crate::lwip_error::LwipError;
use crate::util::ip_addr_to_u32;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct TcpSocket {
    pub socket: Socket,
}

impl TcpSocket {
    pub fn create() -> Result<Self, LwipError> {
        let socket = unsafe { ffi::env_net_socket_new_tcp() };
        if socket < 0 {
            return Err(LwipError::from_code(socket));
        }
        Ok(Self {
            socket: Socket {
                inner: Rc::new(RefCell::new(SocketInner { socket })),
            },
        })
    }

    pub async fn connect(&mut self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        struct TcpConnection {
            socket: i32,
        }

        impl Future for TcpConnection {
            type Output = Result<(), LwipError>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };

                let err = unsafe { ffi::env_net_socket_connect_poll(self.socket) };

                if err == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                if err == LwipError::Ok.to_code() {
                    return Poll::Ready(Ok(()));
                }

                log::error!("Failed to connect to socket poll: {}", err);
                return Poll::Ready(Err(LwipError::from_code(err)));
            }
        }

        let addr = ip_addr_to_u32(addr_str)?;
        let result: i32 = unsafe {
            ffi::env_net_socket_connect(self.socket.inner.borrow().socket, addr, port.into())
        };

        if result != LwipError::Ok.to_code() {
            log::error!("Failed to connect to socket: {}", result);
            return Err(LwipError::from_code(result));
        }

        let result = TcpConnection {
            socket: self.socket.inner.borrow().socket,
        }
        .await;

        if result.is_err() {
            return Err(result.err().unwrap());
        }

        Ok(())
    }

    pub fn bind(&self, addr_str: &str, port: u16) -> Result<(), LwipError> {
        let addr = ip_addr_to_u32(addr_str)?;
        let result = unsafe {
            ffi::env_net_socket_bind(self.socket.inner.borrow().socket, addr, port.into())
        };
        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }
        Ok(())
    }

    pub fn listen(&self, backlog: u32) -> Result<(), LwipError> {
        let result =
            unsafe { ffi::env_net_socket_listen(self.socket.inner.borrow().socket, backlog) };

        if result != LwipError::Ok.to_code() {
            return Err(LwipError::from_code(result));
        }

        Ok(())
    }

    pub async fn accept(&self) -> Result<Self, LwipError> {
        struct TcpAccept {
            socket: Socket,
        }

        impl Future for TcpAccept {
            type Output = Result<TcpSocket, LwipError>;

            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                unsafe { ffi::env_net_rx() };
                let result =
                    unsafe { ffi::env_net_socket_accept_poll(self.socket.inner.borrow().socket) };

                if result == LwipError::WouldBlock.to_code() {
                    return Poll::Pending;
                }

                if result < 0 {
                    return Poll::Ready(Err(LwipError::from_code(result)));
                }

                return Poll::Ready(Ok(TcpSocket {
                    socket: Socket {
                        inner: Rc::new(RefCell::new(SocketInner { socket: result })),
                    },
                }));
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
