// sentiric-sip-core/src/builder.rs

use crate::packet::SipPacket;

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