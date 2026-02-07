// sentiric-sip-core/src/builder.rs

use crate::header::{Header, HeaderName};
use crate::packet::SipPacket;
use crate::utils::generate_branch_id;

pub struct SipResponseFactory;

impl SipResponseFactory {
    pub fn create_100_trying(req: &SipPacket) -> SipPacket {
        SipPacket::create_response_for(req, 100, "Trying".into())
    }

    pub fn create_180_ringing(req: &SipPacket) -> SipPacket {
        SipPacket::create_response_for(req, 180, "Ringing".into())
    }

    pub fn create_200_ok(req: &SipPacket) -> SipPacket {
        SipPacket::create_response_for(req, 200, "OK".into())
    }

    pub fn create_error(req: &SipPacket, code: u16, reason: &str) -> SipPacket {
        SipPacket::create_response_for(req, code, reason.into())
    }
}

/// Via başlığı oluşturur (RFC 3261)
pub fn build_via_header(host: &str, port: u16, transport: &str) -> Header {
    let branch = generate_branch_id();
    let value = format!(
        "SIP/2.0/{} {}:{};branch={}",
        transport.to_uppercase(),
        host,
        port,
        branch
    );
    Header::new(HeaderName::Via, value)
}

/// Contact başlığı oluşturur (RFC 3261)
pub fn build_contact_header(username: &str, host: &str, port: u16) -> Header {
    let value = format!("<sip:{}@{}:{}>", username, host, port);
    Header::new(HeaderName::Contact, value)
}