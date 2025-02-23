mod asyncio;
mod executor;
mod ffi;
mod logging;
mod lwip_error;
mod panic;
mod util;
use asyncio::sleep_ms;
use executor::Executor;
use futures_lite::StreamExt;
use log::info;
use logging::init_with_level;
use std::{cell::RefCell, rc::Rc};
use util::ip_addr_to_u32;

// fn mainloop_3(executor: Executor) {
//     executor.spawn(async move {
//         let socket = TcpSocket::create();
//         if socket.is_err() {
//             log::error!("Failed to create socket: {}", socket.err().unwrap());
//             return;
//         }

//         let mut socket = socket.unwrap();

//         let result = socket.connect("10.0.2.2", 8081).await;
//         if result.is_err() {
//             log::error!("Failed to connect to socket: {}", result.err().unwrap());
//             return;
//         }

//         return;

//         let mut numBytes = 0;
//         let mut last_print = 0;

//         loop {
//             // let result = socket.socket.write(b"Hello, world!").await;
//             // if result.is_err() {
//             //     log::error!("Failed to write to socket: {}", result.err().unwrap());
//             //     return;
//             // }

//             let result = socket.socket.read(512).await;
//             if result.is_err() {
//                 log::error!("Failed to read from socket: {}", result.err().unwrap());
//                 return;
//             }

//             let buf = result.unwrap();
//             numBytes += buf.len();

//             if numBytes - last_print > 10000 {
//                 last_print = numBytes;
//                 log::info!("Read {} bytes", numBytes);
//             }
//         }
//     });
// }

async fn mainloop_2() {
    let mut client = asyncio::http::client::Client::new();
    let response = client
        .get("http://cs.stanford.edu/people/karpathy/char-rnn/warpeace_input.txt")
        .await;
    if response.is_err() {
        log::error!("Failed to get response: {}", response.err().unwrap());
        return;
    }

    let mut stream = response.unwrap().stream().await;

    while let Some(chunk) = stream.next().await {
        log::info!(
            "Chunk: {}",
            String::from_utf8_lossy(chunk.unwrap().to_vec().as_slice())
        );
    }

    // log::info!("Response: {}", response.unwrap().text().await.unwrap());
}

#[no_mangle]
pub extern "C" fn main() {
    {
        panic::set_once();
        init_with_level(log::Level::Trace).unwrap();
        let setup_result = unsafe { ffi::env_net_setup() };
        if setup_result != 0 {
            log::error!("Failed to setup network: {}", setup_result);
            return;
        }
        unsafe { ffi::env_net_dns_set_server(ip_addr_to_u32("8.8.8.8").unwrap()) };
        info!("Starting mainloop");
        let executor = Executor::default();
        let exit: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

        let exit_2 = exit.clone();
        executor.spawn(async move {
            info!("Current time: {}", unsafe { ffi::env_now() });
            sleep_ms(10000).await;
            info!("Current time: {}", unsafe { ffi::env_now() });
            *exit_2.borrow_mut() = true;
        });

        executor.spawn(async move {
            mainloop_2().await;
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
