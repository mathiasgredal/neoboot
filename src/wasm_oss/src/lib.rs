mod asyncio;
mod ffi;
mod logging;
mod lwip_error;
mod panic;

use std::{cell::RefCell, rc::Rc};

use asyncio::{get_keypress, sleep_ms, tcp::TcpSocket};
use log::info;
use logging::init_with_level;
use simple_async_local_executor::Executor;

fn mainloop(executor: Executor) {
    executor.clone().spawn(async move {
        info!("Connecting to server");
        let socket = TcpSocket::create().await;

        if socket.is_err() {
            log::error!("Failed to create socket: {}", socket.err().unwrap());
            return;
        }

        let mut socket = socket.unwrap();

        let result = socket.connect("192.168.1.120", 8080).await;

        if result.is_err() {
            log::error!("Failed to connect to server: {}", result.err().unwrap());
            return;
        }

        let socket_2 = socket.clone();

        executor.clone().spawn(async move {
            loop {
                let result = socket_2.write(b"Hello from WASM!\n").await;

                if result.is_err() {
                    log::error!("Failed to write to socket: {}", result.err().unwrap());
                    return;
                }
                sleep_ms(1000).await;
            }
        });

        loop {
            let result = socket.read(2).await;
            if result.is_err() {
                log::error!("Failed to read from socket: {}", result.err().unwrap());
                return;
            }

            let buf = result.unwrap();
            log::info!("Read {} bytes: {:?}", buf.len(), buf);
        }
    });
}

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

            client_socket.close().await;
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
        sleep_ms(100000).await;
        *exit_2.borrow_mut() = true;
    });

    mainloop_2(executor.clone());

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
