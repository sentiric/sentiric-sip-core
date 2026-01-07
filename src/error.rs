// sentiric-sip-core/src/error.rs

use std::fmt;

#[derive(Debug)]
pub enum SipError {
    Utf8Error,
    ParseError(String),
    NetworkError(String),
}

impl fmt::Display for SipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SipError::Utf8Error => write!(f, "UTF-8 Dönüşüm Hatası"),
            SipError::ParseError(msg) => write!(f, "SIP Parse Hatası: {}", msg),
            SipError::NetworkError(msg) => write!(f, "Ağ Hatası: {}", msg),
        }
    }
}

impl std::error::Error for SipError {}

// std::str::Utf8Error -> SipError dönüşümü
impl From<std::str::Utf8Error> for SipError {
    fn from(_: std::str::Utf8Error) -> Self {
        SipError::Utf8Error
    }
}