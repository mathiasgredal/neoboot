use std::panic;

use crate::ffi::env_print;

pub fn hook(info: &panic::PanicHookInfo) {
    let msg = info.to_string() + "\n";
    unsafe { env_print(msg.as_ptr(), msg.len() as u32) };
    // TODO: print stack backtrace
}

#[inline]
pub fn set_once() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook));
    });
}