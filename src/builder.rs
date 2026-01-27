// sentiric-sip-core/src/builder.rs
// ✅ YENİ: SIP Header Builder Helpers

use crate::header::{Header, HeaderName};
use crate::utils::generate_branch_id as generate_branch;

/// Via başlığı oluşturur (Proxy routing için kritik)
/// 
/// Example: Via: SIP/2.0/UDP proxy-service.service.sentiric.cloud:13074;branch=z9hG4bK776asdhds
pub fn build_via_header(advertised_host: &str, sip_port: u16, protocol: &str) -> Header {
    let branch = generate_branch();
    let value = format!(
        "SIP/2.0/{} {}:{};branch={}",
        protocol.to_uppercase(), // UDP veya TCP
        advertised_host,
        sip_port,
        branch
    );
    
    Header::new(HeaderName::Via, value)
}

/// Contact başlığı oluşturur (Response routing için kritik)
///
/// Example: Contact: <sip:proxy@proxy-service.service.sentiric.cloud:13074>
pub fn build_contact_header(username: &str, advertised_host: &str, sip_port: u16) -> Header {
    let value = format!(
        "<sip:{}@{}:{}>",
        username,
        advertised_host,
        sip_port
    );
    
    Header::new(HeaderName::Contact, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_via_header_format() {
        let via = build_via_header("proxy-service.service.sentiric.cloud", 13074, "UDP");
        assert!(via.value.starts_with("SIP/2.0/UDP"));
        assert!(via.value.contains("proxy-service.service.sentiric.cloud:13074"));
        assert!(via.value.contains("branch=z9hG4bK"));
    }

    #[test]
    fn test_contact_header_format() {
        let contact = build_contact_header("proxy", "proxy-service.service.sentiric.cloud", 13074);
        assert_eq!(contact.value, "<sip:proxy@proxy-service.service.sentiric.cloud:13074>");
    }
}
