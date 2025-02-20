mod asyncio;
mod ffi;
mod logging;
mod lwip_error;
mod panic;
mod util;
use asyncio::{sleep_ms, tcp::TcpSocket};
use futures_lite::future::yield_now;
use log::info;
use logging::init_with_level;
use simple_async_local_executor::Executor;
use std::{cell::RefCell, rc::Rc};

fn mainloop_3(executor: Executor) {
    executor.spawn(async move {
        let socket = TcpSocket::create();
        if socket.is_err() {
            log::error!("Failed to create socket: {}", socket.err().unwrap());
            return;
        }

        let mut socket = socket.unwrap();

        let result = socket.connect_raw("192.168.1.120", 8081).await;
        if result.is_err() {
            log::error!("Failed to connect to socket: {}", result.err().unwrap());
            return;
        }

        let mut numBytes = 0;
        let mut last_print = 0;

        loop {
            // let result = socket.socket.write(b"Hello, world!").await;
            // if result.is_err() {
            //     log::error!("Failed to write to socket: {}", result.err().unwrap());
            //     return;
            // }
            let result = socket.socket.read(512).await;
            if result.is_err() {
                log::error!("Failed to read from socket: {}", result.err().unwrap());
                return;
            }

            let buf = result.unwrap();
            numBytes += buf.len();

            if numBytes - last_print > 10000 {
                last_print = numBytes;
                log::info!("Read {} bytes", numBytes);
            }
        }
    });
}

#[no_mangle]
pub extern "C" fn main() {
    {
        panic::set_once();
        init_with_level(log::Level::Info).unwrap();
        let setup_result = unsafe { ffi::env_net_setup() };
        if setup_result != 0 {
            log::error!("Failed to setup network: {}", setup_result);
            return;
        }
        info!("Starting mainloop");
        let executor = Executor::default();
        let exit: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        mainloop_3(executor.clone());

        let exit_2 = exit.clone();
        executor.spawn(async move {
            info!("Current time: {}", unsafe { ffi::env_now() });
            sleep_ms(10000).await;
            info!("Current time: {}", unsafe { ffi::env_now() });
            *exit_2.borrow_mut() = true;
        });

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
