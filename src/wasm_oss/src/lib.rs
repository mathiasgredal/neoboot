mod asyncio;
mod executor;
mod ffi;
mod hyper_tls;
mod logging;
mod lwip_error;
mod panic;
mod tls;
mod util;
use asyncio::sleep_ms;
use executor::Executor;
use futures_lite::StreamExt;
use getrandom::register_custom_getrandom;
use hyper_tls::test_hyper_tls;
use log::info;
use logging::init_with_level;
use rand::{RngCore, SeedableRng};
use util::ip_addr_to_u32;

async fn mainloop_2() {
    info!("Creating client");
    let mut client = asyncio::http::client::Client::new();
    info!("Making request");
    let response = client.get("http://10.0.2.2:3000/out").await;

    info!("Got response");
    if response.is_err() {
        log::error!("Failed to get response: {}", response.err().unwrap());
        return;
    }

    let mut stream = response.unwrap().stream().await;

    let mut total_bytes = 0;
    while let Some(chunk) = stream.next().await {
        if chunk.is_ok() {
            total_bytes += chunk.unwrap().len();
            log::info!("Total bytes: {}", total_bytes);
        }
        // log::info!(
        //     "Chunk: {}",
        //     String::from_utf8_lossy(chunk.unwrap().to_vec().as_slice())
        // );
    }

    info!("Done");

    // log::info!("Response: {}", response.unwrap().text().await.unwrap());
}

pub fn unsecure_getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let seed = 1234;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    rng.fill_bytes(buf);
    Ok(())
}

#[no_mangle]
pub extern "C" fn main() {
    {
        panic::set_once();
        init_with_level(log::Level::Trace).unwrap();
        register_custom_getrandom!(unsecure_getrandom);
        let setup_result = unsafe { ffi::env_net_setup() };
        if setup_result != 0 {
            log::error!("Failed to setup network: {}", setup_result);
            return;
        }
        unsafe { ffi::env_net_dns_set_server(ip_addr_to_u32("8.8.8.8").unwrap()) };
        let executor = Executor::new();

        let mut executor_2 = executor.clone();
        executor.spawn(async move {
            sleep_ms(20000).await;
            executor_2.exit();
        });

        let executor_3 = executor.clone();
        executor.spawn(async move {
            // test_hyper(executor_3).await;
            test_hyper_tls(executor_3).await;
            // mainloop_2().await;
        });

        executor.run();
        info!("Exiting...");
    }
    unsafe { ffi::env_net_teardown() };
}
