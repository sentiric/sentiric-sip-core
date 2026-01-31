// sentiric-sip-core/src/routing.rs

use crate::header::{Header, HeaderName};
use crate::utils::generate_branch_id;
use std::net::SocketAddr;

/// SIP Mesajlarını yönlendirmek için yardımcı fonksiyonlar.
pub struct SipRouter;

impl SipRouter {
    /// Standart bir Via başlığı oluşturur.
    pub fn build_via(host: &str, port: u16, transport: &str) -> Header {
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

    /// Standart bir Record-Route başlığı oluşturur (Loose Routing 'lr' parametresi ile).
    pub fn build_record_route(host: &str, port: u16) -> Header {
        let value = format!("<sip:{}:{};lr>", host, port);
        Header::new(HeaderName::RecordRoute, value)
    }

    /// Yanıtın döneceği adresi `Via` başlığından çözer (RFC 3261 + NAT Traversal).
    /// SBC ve Proxy servislerinde tekrar eden mantığı burası yönetir.
    pub fn resolve_response_target(via_val: &str, default_port: u16) -> Option<SocketAddr> {
        let parts: Vec<&str> = via_val.split_whitespace().collect();
        if parts.len() < 2 { return None; }
        
        let protocol_part = parts[1];
        let params: Vec<&str> = protocol_part.split(';').collect();
        let mut host_part = params[0].to_string(); 
        
        let mut rport: Option<String> = None;
        let mut received: Option<String> = None;

        for param in &params[1..] {
             let p_trim = param.trim();
            if let Some((k, v)) = p_trim.split_once('=') {
                if k == "received" { received = Some(v.to_string()); }
                if k == "rport" { rport = Some(v.to_string()); }
            } else if p_trim == "rport" {
                // RFC 3581: rport parametresi var ama değeri yoksa (istek),
                // yanıt dönerken burası doldurulur. Ancak biz yanıtı route ediyorsak
                // ve değer boşsa, received IP'sini ve varsayılan portu kullanmalıyız.
                rport = Some("".to_string());
            }
        }

        // 1. NAT Traversal Önceliği: received ve rport
        if let (Some(rec), Some(rp)) = (received, rport) {
            if !rp.is_empty() {
                // rport değeri varsa onu kullan
                return format!("{}:{}", rec, rp).parse().ok();
            } else {
                // rport var ama boş, ve received var. 
                // Port bilgisini host kısmından çıkarmaya çalış veya default kullan.
                if host_part.contains(':') {
                    if let Some(port_part) = host_part.split(':').last() {
                        return format!("{}:{}", rec, port_part).parse().ok();
                    }
                }
                return format!("{}:{}", rec, default_port).parse().ok();
            }
        }

        // 2. Direct Routing
        if !host_part.contains(':') {
             host_part = format!("{}:{}", host_part, default_port);
        }
        host_part.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_response_target_nat() {
        let via = "SIP/2.0/UDP 192.168.1.50:5060;received=88.100.1.1;rport=45678;branch=z9hG4bK...";
        let target = SipRouter::resolve_response_target(via, 5060).unwrap();
        assert_eq!(target.to_string(), "88.100.1.1:45678");
    }

    #[test]
    fn test_resolve_response_target_direct() {
        let via = "SIP/2.0/UDP 10.0.0.1:5060;branch=z9hG4bK...";
        let target = SipRouter::resolve_response_target(via, 5060).unwrap();
        assert_eq!(target.to_string(), "10.0.0.1:5060");
    }
}