// src/packet.rs

use crate::header::{Header, HeaderName};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Invite,
    Ack,
    Bye,
    Cancel,
    Options,
    Register,
    Other(String),
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Method::Invite => write!(f, "INVITE"),
            Method::Ack => write!(f, "ACK"),
            Method::Bye => write!(f, "BYE"),
            Method::Cancel => write!(f, "CANCEL"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Register => write!(f, "REGISTER"),
            Method::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Version {
    V2,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SIP/2.0")
    }
}

#[derive(Debug, Clone)]
pub struct SipPacket {
    pub is_request: bool,
    pub method: Method,
    pub uri: String,         // Request URI
    pub status_code: u16,    // Response Code
    pub reason: String,      // Response Reason
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

impl SipPacket {
    pub fn new_request(method: Method, uri: String) -> Self {
        SipPacket {
            is_request: true,
            method,
            uri,
            status_code: 0,
            reason: String::new(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn new_response(code: u16, reason: String) -> Self {
        SipPacket {
            is_request: false,
            method: Method::Other(String::new()),
            uri: String::new(),
            status_code: code,
            reason,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Belirli bir header'ın değerini güvenli şekilde alır
    pub fn get_header_value(&self, name: HeaderName) -> Option<&String> {
        self.headers.iter().find(|h| h.name == name).map(|h| &h.value)
    }

    /// Paketi ağa gönderilecek byte dizisine çevirir
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        
        // 1. Start Line
        if self.is_request {
            out.extend_from_slice(format!("{} {} SIP/2.0\r\n", self.method, self.uri).as_bytes());
        } else {
            out.extend_from_slice(format!("SIP/2.0 {} {}\r\n", self.status_code, self.reason).as_bytes());
        }

        // 2. Headers
        for header in &self.headers {
            out.extend_from_slice(format!("{}: {}\r\n", header.name, header.value).as_bytes());
        }

        // 3. Content-Length (Otomatik Ekle)
        // Eğer body varsa ve header eklenmemişse biz ekleriz.
        let has_content_length = self.headers.iter().any(|h| h.name == HeaderName::ContentLength);
        if !has_content_length {
            out.extend_from_slice(format!("Content-Length: {}\r\n", self.body.len()).as_bytes());
        }

        // 4. Empty Line (Separator)
        out.extend_from_slice(b"\r\n");

        // 5. Body
        out.extend_from_slice(&self.body);

        out
    }
}