mod asyncio;
mod dns;
mod ffi;
mod logging;
mod lwip_error;
mod panic;
use std::{cell::RefCell, rc::Rc};

use asyncio::{embedded_nal_async::TcpStream, get_keypress, sleep_ms, tcp::TcpSocket};
use dns::StaticDns;
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
        let socket = TcpSocket::create().await;
        if socket.is_err() {
            log::error!("Failed to create socket: {}", socket.err().unwrap());
            return;
        }

        let socket = socket.unwrap();

        let result = socket.bind("0.0.0.0", 8080).await;
        if result.is_err() {
            log::error!("Failed to bind to socket: {}", result.err().unwrap());
            return;
        }

        let result = socket.listen(10).await;
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

            let result = client_socket.read(1024).await;
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
        loop {
            let url = format!("http://192.168.1.120:8081");
            let binding = TcpStream::default();
            let mut client = HttpClient::new(&binding, &StaticDns);
            let mut rx_buf = [0; 4096];
            let mut request = client.request(Method::GET, &url).await.unwrap();
            let response = request.send(&mut rx_buf).await.unwrap();
            let body = response.body().read_to_end().await.unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            log::info!("Response: {:?}", body_str);
        }
    });
}

#[no_mangle]
pub extern "C" fn main() {
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
    unsafe { ffi::env_net_teardown() };
}
