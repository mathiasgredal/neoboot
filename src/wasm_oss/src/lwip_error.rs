use std::fmt;
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LwipError {
    /// No error, everything OK
    Ok,
    /// Out of memory error
    OutOfMemory,
    /// Buffer error
    Buffer,
    /// Timeout
    Timeout,
    /// Routing problem
    Routing,
    /// Operation in progress
    InProgress,
    /// Illegal value
    InvalidValue,
    /// Operation would block
    WouldBlock,
    /// Address in use
    AddressInUse,
    /// Already connecting
    AlreadyConnecting,
    /// Connection already established
    AlreadyConnected,
    /// Not connected
    NotConnected,
    /// Low-level network interface error
    NetworkInterface,
    /// Connection aborted
    ConnectionAborted,
    /// Connection reset
    ConnectionReset,
    /// Connection closed
    ConnectionClosed,
    /// Illegal argument
    IllegalArgument,
}

impl LwipError {
    /// Convert from the lwIP error code to LwipError
    pub fn from_code(code: i32) -> Self {
        match code {
            0 => LwipError::Ok,
            -1 => LwipError::OutOfMemory,
            -2 => LwipError::Buffer,
            -3 => LwipError::Timeout,
            -4 => LwipError::Routing,
            -5 => LwipError::InProgress,
            -6 => LwipError::InvalidValue,
            -7 => LwipError::WouldBlock,
            -8 => LwipError::AddressInUse,
            -9 => LwipError::AlreadyConnecting,
            -10 => LwipError::AlreadyConnected,
            -11 => LwipError::NotConnected,
            -12 => LwipError::NetworkInterface,
            -13 => LwipError::ConnectionAborted,
            -14 => LwipError::ConnectionReset,
            -15 => LwipError::ConnectionClosed,
            -16 => LwipError::IllegalArgument,
            _ => LwipError::InvalidValue,
        }
    }

    /// Convert LwipError to the lwIP error code
    pub fn to_code(&self) -> i32 {
        match self {
            LwipError::Ok => 0,
            LwipError::OutOfMemory => -1,
            LwipError::Buffer => -2,
            LwipError::Timeout => -3,
            LwipError::Routing => -4,
            LwipError::InProgress => -5,
            LwipError::InvalidValue => -6,
            LwipError::WouldBlock => -7,
            LwipError::AddressInUse => -8,
            LwipError::AlreadyConnecting => -9,
            LwipError::AlreadyConnected => -10,
            LwipError::NotConnected => -11,
            LwipError::NetworkInterface => -12,
            LwipError::ConnectionAborted => -13,
            LwipError::ConnectionReset => -14,
            LwipError::ConnectionClosed => -15,
            LwipError::IllegalArgument => -16,
        }
    }
}

impl fmt::Display for LwipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            LwipError::Ok => "Ok",
            LwipError::OutOfMemory => "Out of memory error",
            LwipError::Buffer => "Buffer error",
            LwipError::Timeout => "Timeout",
            LwipError::Routing => "Routing problem",
            LwipError::InProgress => "Operation in progress",
            LwipError::InvalidValue => "Illegal value",
            LwipError::WouldBlock => "Operation would block",
            LwipError::AddressInUse => "Address in use",
            LwipError::AlreadyConnecting => "Already connecting",
            LwipError::AlreadyConnected => "Already connected",
            LwipError::NotConnected => "Not connected",
            LwipError::NetworkInterface => "Low-level netif error",
            LwipError::ConnectionAborted => "Connection aborted",
            LwipError::ConnectionReset => "Connection reset",
            LwipError::ConnectionClosed => "Connection closed",
            LwipError::IllegalArgument => "Illegal argument",
        };
        write!(f, "{}", msg)
    }
}

impl Error for LwipError {}

/// Type alias for Result with LwipError as the error type
pub type LwipResult<T> = Result<T, LwipError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        assert_eq!(LwipError::from_code(-1), LwipError::OutOfMemory);
        assert_eq!(LwipError::OutOfMemory.to_code(), -1);
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            LwipError::OutOfMemory.to_string(),
            "Out of memory error"
        );
    }
}