use super::{socket::Socket, tcp::TcpSocket, udp::UdpSocket};
use crate::lwip_error::LwipError;

impl embedded_io_async::ErrorType for Socket {
    type Error = LwipError;
}

impl embedded_io_async::Read for Socket {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let result = Socket::read(self, buf.len() as u16).await;
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        let data = result.unwrap();
        buf[..data.len()].copy_from_slice(&data);
        Ok(data.len())
    }
}

impl embedded_io_async::Write for Socket {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let result = Socket::write(self, buf).await;
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        Ok(result.unwrap())
    }
}

impl embedded_nal_async::TcpConnect for TcpSocket {
    type Error = LwipError;

    type Connection<'a> = Socket;

    async fn connect<'a>(
        &'a self,
        remote: std::net::SocketAddr,
    ) -> Result<Self::Connection<'a>, Self::Error> {
        let socket = TcpSocket::create();
        if socket.is_err() {
            let err = socket.err().unwrap();
            return Err(err.into());
        }

        let mut socket = socket.unwrap();
        let result =
            TcpSocket::connect(&mut socket, remote.ip().to_string().as_str(), remote.port()).await;
        if result.is_err() {
            let err = result.err().unwrap();
            return Err(err.into());
        }

        Ok(socket.socket)
    }
}

impl embedded_nal_async::ConnectedUdp for UdpSocket {
    type Error = LwipError;

    async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let result = Socket::write(&mut self.socket, data).await;
        if result.is_err() {
            let err = result.err().unwrap();
            return Err(err.into());
        }

        Ok(())
    }

    async fn receive_into(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let result = Socket::read(&mut self.socket, buffer.len() as u16).await;
        if result.is_err() {
            let err = result.err().unwrap();
            return Err(err.into());
        }

        let data = result.unwrap();
        buffer[..data.len()].copy_from_slice(&data);
        Ok(data.len())
    }
}

impl embedded_nal_async::UnconnectedUdp for UdpSocket {
    type Error = LwipError;

    async fn send(
        &mut self,
        _: std::net::SocketAddr,
        remote: std::net::SocketAddr,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let result = UdpSocket::connect(self, remote.ip().to_string().as_str(), remote.port());
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        let result = Socket::write(&mut self.socket, data).await;
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        Ok(())
    }

    async fn receive_into(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(usize, std::net::SocketAddr, std::net::SocketAddr), Self::Error> {
        let result = Socket::read(&mut self.socket, buffer.len() as u16).await;
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        let data = result.unwrap();
        buffer[..data.len()].copy_from_slice(&data);

        // TODO: Get remote and local addresses
        let remote_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(0.into()), 0);
        let local_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(0.into()), 0);
        Ok((data.len(), remote_addr, local_addr))
    }
}

impl embedded_nal_async::UdpStack for UdpSocket {
    type Error = LwipError;

    type Connected = UdpSocket;

    type UniquelyBound = UdpSocket;

    type MultiplyBound = UdpSocket;

    async fn connect_from(
        &self,
        local: std::net::SocketAddr,
        remote: std::net::SocketAddr,
    ) -> Result<(std::net::SocketAddr, Self::Connected), Self::Error> {
        let result = UdpSocket::bind(self, local.ip().to_string().as_str(), local.port());
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        let result = UdpSocket::connect(self, remote.ip().to_string().as_str(), remote.port());
        if result.is_err() {
            let err = result.err().unwrap();
            return Err(err.into());
        }

        Ok((local, self.clone()))
    }

    async fn bind_single(
        &self,
        local: std::net::SocketAddr,
    ) -> Result<(std::net::SocketAddr, Self::UniquelyBound), Self::Error> {
        let result = UdpSocket::bind(self, local.ip().to_string().as_str(), local.port());
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        Ok((local, self.clone()))
    }

    async fn bind_multiple(
        &self,
        local: std::net::SocketAddr,
    ) -> Result<Self::MultiplyBound, Self::Error> {
        let result = UdpSocket::bind(self, local.ip().to_string().as_str(), local.port());
        if result.is_err() {
            return Err(result.err().unwrap().into());
        }

        Ok(self.clone())
    }
}
