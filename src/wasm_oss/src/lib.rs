mod asyncio;
mod ffi;
mod logging;
mod lwip_error;
mod panic;

use asyncio::{get_keypress, tcp::TcpSocket};
use logging::init_with_level;
use simple_async_local_executor::Executor;

#[no_mangle]
pub extern "C" fn main() {
    panic::set_once();
    init_with_level(log::Level::Info).unwrap();
    let executor = Executor::default();

    let setup_result = unsafe { ffi::env_setup_network() };
    if setup_result != 0 {
        log::error!("Failed to setup network: {}", setup_result);
        return;
    }
    executor.spawn(async {
        let mut socket = TcpSocket::connect("192.168.1.120", 8081).await;

        if socket.is_err() {
            log::error!("Failed to connect to server: {}", socket.err().unwrap());
            return;
        }


        log::info!("Press a key to write to the socket");
        loop {
            let keycode = get_keypress().await;
            let buf: String;

            if keycode == 13 {
                buf = "\n".to_string();
            }
            else if keycode == 8 {
                buf = "^H".to_string();
            } else {
                buf = format!("{}", std::char::from_u32(keycode.try_into().unwrap()).unwrap());
            }

            if buf == "q" {
                break;
            }

            let result = socket.as_mut().unwrap().write(buf.as_bytes()).await;
            if result.is_err() {
                log::error!("Failed to write to socket: {}", result.err().unwrap());
                return;
            }

            log::info!("Wrote {} bytes", result.unwrap());
        }

    });

    // executor.spawn(async {
    //     log::info!("Awaiting keypress B");
    //     let key = get_keypress().await;
    //     log::info!("Key pressed: {}", key);
    // });

    loop {
        // TODO: Make some kind of reactor design, to handle this more modularly
        let more_tasks = executor.step();
        if !more_tasks {
            break;
        }
    }

    unsafe { ffi::env_teardown_network() };
}
