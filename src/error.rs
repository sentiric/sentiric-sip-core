// sentiric-sip-core/src/error.rs

use std::fmt;

#[derive(Debug)]
pub enum SipError {
    Utf8Error,
    ParseError(String),
    SocketError(String),
    NetworkError(String),
    ProtocolError(String),
}

impl fmt::Display for SipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SipError::Utf8Error => write!(f, "UTF-8 Dönüşüm Hatası"),
            SipError::ParseError(msg) => write!(f, "SIP Parse Hatası: {}", msg),
            SipError::SocketError(msg) => write!(f, "Soket Hatası: {}", msg),
            SipError::NetworkError(msg) => write!(f, "Ağ Hatası: {}", msg),
            SipError::ProtocolError(msg) => write!(f, "Protokol İhlali: {}", msg),
        }
    }
}

impl std::error::Error for SipError {}

impl From<std::str::Utf8Error> for SipError {
    fn from(_: std::str::Utf8Error) -> Self {
        SipError::Utf8Error
    }
}

impl From<std::io::Error> for SipError {
    fn from(err: std::io::Error) -> Self {
        SipError::SocketError(err.to_string())
    }
}