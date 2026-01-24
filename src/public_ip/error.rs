use std::fmt::write;

#[derive(Debug)]
pub enum PublicIpLookupError {
    BadIpAddr(String),
    HttpError(reqwest::Error),
}

impl std::error::Error for PublicIpLookupError {}

impl From<std::net::AddrParseError> for PublicIpLookupError {
    fn from(e: std::net::AddrParseError) -> Self {
        Self::BadIpAddr(e.to_string())
    }
}

impl From<reqwest::Error> for PublicIpLookupError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e)
    }
}

impl std::fmt::Display for PublicIpLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadIpAddr(addr) => write!(f, "Invalid IP Address: {}", addr),
            Self::HttpError(err) => write!(f, "HTTP error: {}", err),
        }
    }
}
