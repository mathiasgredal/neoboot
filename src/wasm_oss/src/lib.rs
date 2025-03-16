use commands::CommandDispatcher;
use executor::Executor;
use services::ServiceRegistry;

mod asyncio;
mod commands;
mod errors;
mod executor;
mod ffi;
mod services;
mod util;

#[no_mangle]
pub extern "C" fn main() {
    util::panic::set_once();
    util::logging::init_with_level(log::Level::Info).unwrap();

    // Setup network
    let setup_result = unsafe { ffi::env_net_setup() };
    if setup_result != 0 {
        log::error!("Failed to setup network: {}", setup_result);
        return;
    }

    // Setup executor
    let executor = Executor::new();
    let dispatcher = CommandDispatcher::default();

    // Register services
    let service_registry = ServiceRegistry::new();
    service_registry.register_all(vec![
        // Box::new(services::boot::BootService::new()),
        Box::new(services::console::ConsoleService::new(dispatcher.clone())),
        Box::new(services::server::ServerService::new(dispatcher.clone())),
    ]);

    // Spawn services
    if let Some(e) = service_registry.spawn_all(&executor) {
        log::error!("Failed to spawn services: {}", e);
    }

    // Run executor
    executor.run_forever();

    // Teardown network
    unsafe { ffi::env_net_teardown() };
}

// let data = serialize::write_message_to_words(&message);
// println!("{:?}", data);

// let mut executor_2 = executor.clone();
// executor.spawn(async move {
//     sleep_ms(1000000).await;
//     executor_2.exit();
// });

// let executor_3 = executor.clone();
// executor.spawn(async move {
//     mainloop(executor_3).await;
// });

// let executor_4 = executor.clone();
// executor.spawn(async move {
//     let _ = run_server(&executor_4).await;
// });

// executor.run();
// info!("Exiting...");

// async fn mainloop(executor: Executor) {
//     let mut client = asyncio::http::client::Client::new(executor);
//     let response = client
//         .request(
//             http::Method::GET,
//             "https://google.com",
//             RequestConfig::default(),
//         )
//         .await;

//     if response.is_err() {
//         log::error!("Failed to get response: {}", response.err().unwrap());
//         return;
//     }

//     let mut response = response.unwrap();
//     info!("Metadata: {:?}", response.metadata);

//     info!("Response: {}", response.text().await.unwrap());

//     // let mut stream = response.unwrap().stream().await;

//     // while let Some(chunk) = stream.next().await {
//     //     if chunk.is_err() {
//     //         log::error!("Failed to get chunk: {}", chunk.err().unwrap());
//     //         return;
//     //     }

//     //     let chunk = chunk.unwrap();
//     //     log::info!(
//     //         "Chunk: {}",
//     //         String::from_utf8_lossy(chunk.to_vec().as_slice())
//     //     );
//     // }

//     info!("Done");
// }
