pub mod embedded_nal_async;
pub mod keyboard;
pub mod sleep;
pub mod tcp;

pub async fn sleep_ms(duration_ms: u64) {
    let sleep = sleep::Sleep::new(duration_ms);
    sleep.await;
}

pub async fn get_keypress() -> i32 {
    let keypress = keyboard::KeyPress;
    keypress.await
}
