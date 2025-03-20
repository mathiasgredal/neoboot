use crate::errors::lwip_error::LwipError;
use crate::ffi;
use std::net::Ipv4Addr;

// Utility functions
pub fn ip_addr_to_u32(addr: &str) -> Result<u32, LwipError> {
    let addr: Ipv4Addr = addr.parse().map_err(|_| LwipError::IllegalArgument)?;
    Ok(u32::from_be_bytes(addr.octets()).to_be())
}

pub fn sys_print(s: &str) {
    unsafe {
        ffi::env_print(s.as_ptr(), s.len() as u32);
    }
}

pub mod logging {
    use crate::ffi;
    use log::{Level, Log, Metadata, Record, SetLoggerError};

    static LOGGER: Logger = Logger {};

    struct Logger {}

    impl Log for Logger {
        #[inline]
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= log::max_level()
        }

        fn log(&self, record: &Record) {
            if !self.enabled(record.metadata()) {
                return;
            }

            let res = format!(
                "[{level}] - {text}\n",
                level = record.level(),
                text = record.args()
            );
            let res_one_newline = format!("{}\n", res.trim());
            let s = res_one_newline.as_bytes();
            unsafe { ffi::env_print(s.as_ptr(), s.len() as u32) };
        }

        fn flush(&self) {}
    }

    pub fn init_with_level(level: Level) -> core::result::Result<(), SetLoggerError> {
        log::set_logger(&LOGGER)?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }
}

pub mod panic {
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
}
