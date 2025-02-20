
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
