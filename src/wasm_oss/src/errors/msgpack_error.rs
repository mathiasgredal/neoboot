#[derive(Debug)]
pub enum MessagePackError {
    UnexpectedFormat(String),
    InvalidString(std::string::FromUtf8Error),
    UnexpectedEof,
}

impl std::fmt::Display for MessagePackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedFormat(msg) => write!(f, "Unexpected format: {}", msg),
            Self::InvalidString(err) => write!(f, "Invalid string: {}", err),
            Self::UnexpectedEof => write!(f, "Unexpected end of stream"),
        }
    }
}

impl std::error::Error for MessagePackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidString(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::string::FromUtf8Error> for MessagePackError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::InvalidString(err)
    }
}
