use crate::ffi;

#[inline]
pub fn set_once() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(hook));
    });
}

pub fn hook(info: &std::panic::PanicHookInfo) {
    let msg = info.to_string() + "\n";
    unsafe { ffi::env_print(msg.as_ptr(), msg.len() as u32) };
    // TODO: print stack backtrace
}
