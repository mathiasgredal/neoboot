use crate::lwip_error::LwipError;

use super::tcp;

#[derive(Default)]
pub struct TcpStream {
    socket: Option<tcp::TcpSocket>,
}

impl embedded_io_async::ErrorType for TcpStream {
    type Error = LwipError;
}

impl embedded_io_async::Read for TcpStream {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.socket.is_none() {
            return Err(LwipError::NotConnected);
        }

        let result = self.socket.as_mut().unwrap().read(buf.len() as u16).await;
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        let data = result.unwrap();
        for i in 0..data.len() {
            buf[i] = data[i];
        }
        Ok(data.len())
    }
}

impl embedded_io_async::Write for TcpStream {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if self.socket.is_none() {
            return Err(LwipError::NotConnected);
        }

        let result = self.socket.as_mut().unwrap().write(buf).await;
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        let data = result.unwrap();
        Ok(data)
    }
}

impl embedded_nal_async::TcpConnect for TcpStream {
    type Error = LwipError;

    type Connection<'a> = TcpStream;

    async fn connect<'a>(
        &'a self,
        remote: std::net::SocketAddr,
    ) -> Result<Self::Connection<'a>, Self::Error> {
        let socket = tcp::TcpSocket::create().await;
        if socket.is_err() {
            let err = socket.err().unwrap();
            return Err(err.into());
        }

        let mut socket = socket.unwrap();
        let result = socket
            .connect_raw(remote.ip().to_string().as_str(), remote.port())
            .await;
        if result.is_err() {
            let err = result.err().unwrap();
            return Err(err.into());
        }

        Ok(TcpStream {
            socket: Some(socket),
        })
    }
}
