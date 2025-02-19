pub mod dns;
pub mod embedded_traits;
pub mod http;
pub mod keyboard;
pub mod sleep;
pub mod socket;
pub mod tcp;
pub mod udp;

pub async fn sleep_ms(duration_ms: u64) {
    let sleep = sleep::Sleep::new(duration_ms);
    sleep.await;
}

pub async fn get_keypress() -> i32 {
    let keypress = keyboard::KeyPress;
    keypress.await
}
