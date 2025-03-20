use commands::CommandDispatcher;
use executor::Executor;
use services::ServiceRegistry;
use std::{cell::RefCell, rc::Rc};
use util::sys_print;

mod asyncio;
mod commands;
mod errors;
mod executor;
mod ffi;
mod services;
mod util;

#[no_mangle]
pub extern "C" fn main() {
    sys_print("Welcome to the NeoBoot WASM Bootloader! Initializing the system...\n");

    // Setup panic handler
    util::panic::set_once();

    // Setup logging
    match util::logging::init_with_level(log::Level::Info) {
        Ok(_) => (),
        Err(e) => {
            sys_print(format!("Failed to initialize logging: {}", e).as_str());
        }
    }

    // Setup network
    let setup_result = unsafe { ffi::env_net_setup() };
    if setup_result != 0 {
        log::error!("Failed to setup network: {}", setup_result);
        return;
    }

    // Setup executor
    let executor = Executor::new();
    let dispatcher = Rc::new(RefCell::new(CommandDispatcher::default()));
    let mut service_registry = ServiceRegistry::new();
    service_registry.register(services::console::ConsoleService::new(dispatcher.clone()));
    service_registry.register(services::server::ServerService::new(dispatcher.clone()));
    service_registry.spawn_all(&executor);
    executor.run_forever();

    // Teardown network
    unsafe { ffi::env_net_teardown() };
}
