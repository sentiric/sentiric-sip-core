// sentiric-sip-core/src/profiles.rs

use crate::header::{Header, HeaderName};
use crate::packet::SipPacket;
use std::net::SocketAddr;

/// SIP Davranış Profili (Vendor Stratejisi).
/// Farklı operatörlerin (SBC) gereksinimlerine göre SIP mesajlarını şekillendirir.
pub trait SipProfile: Send + Sync {
    /// Profilin adı (Loglama için)
    fn name(&self) -> &str;

    /// Yanıt başlıklarını düzenler (Contact, Record-Route vb.)
    fn add_response_headers(
        &self, 
        resp: &mut SipPacket, 
        req: &SipPacket, 
        public_ip: &str, 
        sip_port: u16
    );

    /// SDP gövdesini oluşturur
    fn build_sdp_body(
        &self, 
        public_ip: &str, 
        rtp_port: u16, 
        codec_id: u8, 
        codec_name: &str
    ) -> String;

    /// Paketin gönderileceği hedef adresi belirler (NAT/Latching için)
    fn determine_target(&self, req: &SipPacket, src: SocketAddr) -> SocketAddr;

    /// Re-INVITE durumunda 100 Trying gönderilip gönderilmeyeceği
    fn should_send_trying_on_reinvite(&self) -> bool;

    /// Cevaplamadan önce 180 Ringing gönderilip gönderilmeyeceği
    fn pre_answer_ringing(&self) -> bool;

    /// User-Agent stringi
    fn user_agent(&self) -> String;
}

// --- Fabrika Fonksiyonu ---
pub fn create_profile(key: &str) -> Box<dyn SipProfile> {
    match key.to_lowercase().as_str() {
        "legacy" | "karel" | "roitel" => Box::new(LegacyProfile),
        _ => Box::new(StandardProfile),
    }
}

// ============================================================================
// STANDARD PROFILE (RFC 3261)
// ============================================================================
pub struct StandardProfile;

impl SipProfile for StandardProfile {
    fn name(&self) -> &str { "Standard (RFC3261)" }

    fn add_response_headers(&self, resp: &mut SipPacket, req: &SipPacket, public_ip: &str, sip_port: u16) {
        // RFC Kuralı: Record-Route başlıkları kopyalanır (Loose Routing)
        for h in &req.headers {
            if h.name == HeaderName::RecordRoute {
                resp.headers.push(h.clone());
            }
        }
        
        resp.headers.push(Header::new(HeaderName::UserAgent, self.user_agent()));
        resp.headers.push(Header::new(HeaderName::Allow, "INVITE, ACK, BYE, CANCEL, OPTIONS".to_string()));
        resp.headers.push(Header::new(HeaderName::Supported, "replaces, timer".to_string()));

        let user = extract_user_from_uri(&req.uri);
        let contact = format!("<sip:{}@{}:{};transport=udp>", user, public_ip, sip_port);
        resp.headers.push(Header::new(HeaderName::Contact, contact));
    }

    fn build_sdp_body(&self, public_ip: &str, rtp_port: u16, codec_id: u8, codec_name: &str) -> String {
        format!(
            "v=0\r\n\
            o=- 1000 1000 IN IP4 {}\r\n\
            s=Sentiric\r\n\
            c=IN IP4 {}\r\n\
            t=0 0\r\n\
            m=audio {} RTP/AVP {} 101\r\n\
            a=rtpmap:{} {}/8000\r\n\
            a=rtpmap:101 telephone-event/8000\r\n\
            a=fmtp:101 0-16\r\n\
            a=ptime:20\r\n\
            a=sendrecv\r\n",
            public_ip, public_ip, rtp_port, codec_id, codec_id, codec_name
        )
    }

    fn determine_target(&self, _req: &SipPacket, src: SocketAddr) -> SocketAddr { src }
    fn should_send_trying_on_reinvite(&self) -> bool { true }
    fn pre_answer_ringing(&self) -> bool { false }
    fn user_agent(&self) -> String { "Sentiric Media Gateway/1.0".to_string() }
}

// ============================================================================
// LEGACY PROFILE (SBC / Strict Firewall Compatibility)
// ============================================================================
pub struct LegacyProfile;

impl SipProfile for LegacyProfile {
    fn name(&self) -> &str { "Legacy (SBC - NO Record-Route)" }

    fn add_response_headers(&self, resp: &mut SipPacket, _req: &SipPacket, public_ip: &str, sip_port: u16) {
        // KAREL/SIPPY FIX: Record-Route KESİNLİKLE silinir.
        resp.headers.push(Header::new(HeaderName::UserAgent, "SentiricGW".to_string()));
        resp.headers.push(Header::new(HeaderName::Allow, "INVITE, ACK, BYE, CANCEL, OPTIONS".to_string()));

        // User Part olmadan, sadece IP:PORT
        let contact = format!("<sip:{}:{}>", public_ip, sip_port);
        resp.headers.push(Header::new(HeaderName::Contact, contact));
    }

    fn build_sdp_body(&self, public_ip: &str, rtp_port: u16, codec_id: u8, codec_name: &str) -> String {
        // Minimalist SDP
        format!(
            "v=0\r\n\
            o=- 0 0 IN IP4 {}\r\n\
            s=sentiric\r\n\
            c=IN IP4 {}\r\n\
            t=0 0\r\n\
            m=audio {} RTP/AVP {}\r\n\
            a=rtpmap:{} {}/8000\r\n\
            a=sendrecv\r\n",
            public_ip, public_ip, rtp_port, codec_id, codec_id, codec_name
        )
    }

    fn determine_target(&self, _req: &SipPacket, src: SocketAddr) -> SocketAddr { src }
    fn should_send_trying_on_reinvite(&self) -> bool { false }
    fn pre_answer_ringing(&self) -> bool { false }
    fn user_agent(&self) -> String { "SentiricGW".to_string() }
}

// --- Helper ---
fn extract_user_from_uri(uri: &str) -> String {
    let start = if let Some(idx) = uri.find("sip:") { idx + 4 } else { 0 };
    let rest = &uri[start..];
    if let Some(end) = rest.find('@') { return rest[..end].to_string(); }
    if let Some(end) = rest.find(':') { return rest[..end].to_string(); }
    rest.to_string()
}