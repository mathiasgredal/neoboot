use commands::{
    boot::BootCommandHandler,
    chain::{self, ChainCommandHandler},
    help::{self, HelpCommandHandler},
    nonce::{self, NonceCommandHandler},
    print::{self, PrintCommandHandler},
    quit::{self, QuitCommandHandler},
    CommandDispatcher,
};
use executor::Executor;
use log::error;
use proto_rs::schema::{
    BootClientRequest, ChainClientRequest, HelpClientRequest, NonceClientRequest,
    PrintClientRequest, QuitClientRequest,
};
use services::ServiceRegistry;
use std::{cell::RefCell, rc::Rc};
use utils::sys_print;

mod asyncio;
mod commands;
mod controllers;
mod errors;
mod executor;
mod ffi;
mod services;
mod utils;

#[no_mangle]
pub extern "C" fn main() {
    sys_print("Welcome to the NeoBoot WASM Bootloader! Initializing the system...\n");

    // Setup panic handler
    utils::panic::set_once();

    // Setup logging
    match utils::logging::init_with_level(log::Level::Info) {
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

    // Setup controllers
    let mut boot_controller = controllers::boot::BootController::new();
    {
        // Setup command dispatcher
        let mut dispatcher = CommandDispatcher::new();
        dispatcher.register_handler::<NonceClientRequest>(NonceCommandHandler {});
        dispatcher.register_handler::<HelpClientRequest>(HelpCommandHandler {});
        dispatcher.register_handler::<PrintClientRequest>(PrintCommandHandler {});
        dispatcher.register_handler::<QuitClientRequest>(QuitCommandHandler {});
        dispatcher.register_handler::<ChainClientRequest>(ChainCommandHandler {});
        dispatcher.register_handler::<BootClientRequest>(BootCommandHandler {
            boot_controller: boot_controller.clone(),
        });
        let dispatcher = Rc::new(RefCell::new(dispatcher));

        // Setup service registry
        let mut service_registry = ServiceRegistry::new();
        service_registry.register(services::console::ConsoleService::new(dispatcher.clone()));
        service_registry.register(services::server::ServerService::new(dispatcher.clone()));
        service_registry.spawn_all(&executor);

        // Run executor
        executor.run_forever();

        // Teardown network
        unsafe { ffi::env_net_teardown() };
    }
    // Boot
    let mut boot_controller = std::sync::Arc::<
        futures::lock::Mutex<controllers::boot::BootController>,
    >::try_unwrap(boot_controller)
    .unwrap()
    .into_inner();
    let result = boot_controller.boot();
    if result.is_err() {
        // TODO: We need to restart the executor to prevent shutdown
        error!("Boot failed: {:?}", result.err());
    }
}
