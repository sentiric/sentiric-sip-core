// src/header.rs

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HeaderName {
    Via,
    From,
    To,
    CallId,
    CSeq,
    Contact,
    ContentType,
    ContentLength,
    RecordRoute, // Sippy için kritik
    Route,
    UserAgent,
    Server,
    Allow,
    Supported,
    MaxForwards,
    Other(String), // Standart dışı başlıklar için
}

impl HeaderName {
    pub fn as_str(&self) -> &str {
        match self {
            HeaderName::Via => "Via",
            HeaderName::From => "From",
            HeaderName::To => "To",
            HeaderName::CallId => "Call-ID",
            HeaderName::CSeq => "CSeq",
            HeaderName::Contact => "Contact",
            HeaderName::ContentType => "Content-Type",
            HeaderName::ContentLength => "Content-Length",
            HeaderName::RecordRoute => "Record-Route",
            HeaderName::Route => "Route",
            HeaderName::UserAgent => "User-Agent",
            HeaderName::Server => "Server",
            HeaderName::Allow => "Allow",
            HeaderName::Supported => "Supported",
            HeaderName::MaxForwards => "Max-Forwards",
            HeaderName::Other(s) => s.as_str(),
        }
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: HeaderName,
    pub value: String,
}

impl Header {
    pub fn new(name: HeaderName, value: String) -> Self {
        Header { name, value }
    }

    /// String'den HeaderName üretir (Parsing sırasında kullanılır)
    pub fn from_str(key: &str) -> HeaderName {
        // RFC 3261 Compact Form desteği (v, f, t, i, m, l)
        match key.trim().to_lowercase().as_str() {
            "via" | "v" => HeaderName::Via,
            "from" | "f" => HeaderName::From,
            "to" | "t" => HeaderName::To,
            "call-id" | "i" => HeaderName::CallId,
            "cseq" => HeaderName::CSeq,
            "contact" | "m" => HeaderName::Contact,
            "content-type" | "c" => HeaderName::ContentType,
            "content-length" | "l" => HeaderName::ContentLength,
            "record-route" => HeaderName::RecordRoute,
            "route" => HeaderName::Route,
            "user-agent" => HeaderName::UserAgent,
            "server" => HeaderName::Server,
            "max-forwards" => HeaderName::MaxForwards,
            _ => HeaderName::Other(key.trim().to_string()),
        }
    }
}