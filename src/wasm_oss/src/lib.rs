mod asyncio;
mod dns;
mod ffi;
mod logging;
mod lwip_error;
mod panic;
use std::{
    cell::RefCell,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    rc::Rc,
};

use asyncio::{
    get_keypress, sleep_ms,
    tcp::{self, TcpSocket, UdpSocket},
};
use dns::ItsDns;
use log::info;
use logging::init_with_level;
use reqwless::{
    client::HttpClient,
    headers::ContentType,
    request::{Method, RequestBuilder},
};
use simple_async_local_executor::Executor;

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
        // let socket = UdpSocket::create();
        // if socket.is_err() {
        //     log::error!("Failed to create socket: {}", socket.err().unwrap());
        //     return;
        // }
        // info!("Socket created");

        // let socket = socket.unwrap();

        // let result = socket.bind("0.0.0.0", 8080);
        // if result.is_err() {
        //     log::error!("Failed to connect to socket: {}", result.err().unwrap());
        //     return;
        // }
        // info!("Socket bound to 0.0.0.0:8080");

        // let result = socket.connect("192.168.1.120", 8080);
        // if result.is_err() {
        //     log::error!("Failed to connect to socket: {}", result.err().unwrap());
        //     return;
        // }
        // info!("Socket connected to 192.168.1.120:8080");

        // let result = tcp::Socket::write(&mut socket.socket.borrow_mut(), b"Hello, world!").await;
        // if result.is_err() {
        //     log::error!("Failed to write to socket: {}", result.err().unwrap());
        //     return;
        // }

        // sleep_ms(5000).await;

        let nameserver: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        let stack = tcp::UdpSocket::create();
        if stack.is_err() {
            log::error!("Failed to create stack: {}", stack.err().unwrap());
            return;
        }

        let stack = stack.unwrap();
        let client = ItsDns::new(stack, nameserver);

        let host = "example.com";
        println!("Resolving {}...", host);
        let ip = client
            .get_host_by_name(host, embedded_nal_async::AddrType::IPv4)
            .await;

        let ip = ip.unwrap();

        info!("Resolved {} to {}", host, ip);
        // loop {
        //     let url = format!("http://192.168.1.120:8081");
        //     let binding = TcpSocket::create();
        //     if binding.is_err() {
        //         log::error!("Failed to create socket: {}", binding.err().unwrap());
        //         return;
        //     }

        //     let binding = binding.unwrap();
        //     let mut client = HttpClient::new(&binding, &StaticDns);
        //     let mut rx_buf = [0; 4096];
        //     let mut request = client.request(Method::GET, &url).await.unwrap();
        //     let response = request.send(&mut rx_buf).await.unwrap();
        //     let body = response.body().read_to_end().await.unwrap();
        //     let body_str = String::from_utf8(body.to_vec()).unwrap();
        //     log::info!("Response: {:?}", body_str);
        // }
    });
}

#[no_mangle]
pub extern "C" fn main() {
    {
        panic::set_once();
        init_with_level(log::Level::Info).unwrap();
        info!("Starting mainloop");
        let executor = Executor::default();
        let setup_result = unsafe { ffi::env_net_setup() };
        if setup_result != 0 {
            log::error!("Failed to setup network: {}", setup_result);
            return;
        }
        let exit: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

        // mainloop(executor.clone());
        let exit_2 = exit.clone();
        executor.spawn(async move {
            sleep_ms(10000).await;
            *exit_2.borrow_mut() = true;
        });

        mainloop_3(executor.clone());

        loop {
            // TODO: Make some kind of reactor design, to handle this more modularly
            let more_tasks = executor.step();
            if !more_tasks {
                break;
            }

            if exit.borrow().clone() {
                break;
            }
        }

        info!("Mainloop finished");
    }
    unsafe { ffi::env_net_teardown() };
}
