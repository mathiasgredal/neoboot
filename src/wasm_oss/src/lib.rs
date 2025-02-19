mod asyncio;
mod ffi;
mod logging;
mod lwip_error;
mod panic;
mod util;
use std::{cell::RefCell, rc::Rc};

use asyncio::{get_keypress, http::client::Client, sleep_ms, tcp::TcpSocket};
use futures_lite::StreamExt;
use log::info;
use logging::init_with_level;
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
        // let http_client = HttpClient::new();
        // let result = http_client.get("http://example.com").await;
        // if result.is_err() {
        //     log::error!("Failed to get from socket: {:?}", result.err().unwrap());
        //     return;
        // }
        // let body = result.unwrap().collect::<Vec<_>>().await;
        // for chunk in body {
        //     log::info!("Chunk: {} bytes", chunk.unwrap().len());
        // }
        // log::info!("Response: {} bytes", body.len());
        let mut client = Client::new();
        let result = client.get("http://example.com").await;
        if result.is_err() {
            log::error!("Failed to get from socket: {:?}", result.err().unwrap());
            return;
        }

        let body = result.unwrap().text().await;
        if body.is_err() {
            log::error!("Failed to get body: {:?}", body.err().unwrap());
            return;
        }
        log::info!("Body: {}", body.unwrap());
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
