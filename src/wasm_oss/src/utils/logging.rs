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
