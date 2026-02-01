// sentiric-sip-core/src/routing.rs

use crate::header::{Header, HeaderName};
use crate::packet::SipPacket;
use crate::utils::generate_branch_id;
use std::net::SocketAddr;

/// SIP Mesajlarını yönlendirmek ve manipüle etmek için merkezi mantık.
pub struct SipRouter;

impl SipRouter {
    /// Paketin en başına yeni bir Via başlığı ekler (Client Transaction).
    /// RFC 3261: Via başlıkları stack (LIFO) mantığıyla çalışır.
    pub fn add_via(packet: &mut SipPacket, host: &str, port: u16, transport: &str) {
        let branch = generate_branch_id();
        let value = format!(
            "SIP/2.0/{} {}:{};branch={}",
            transport.to_uppercase(),
            host,
            port,
            branch
        );
        // Via her zaman en üstte olmalıdır.
        packet.headers.insert(0, Header::new(HeaderName::Via, value));
    }

    /// Paketin en üstündeki Via başlığını kaldırır (Response Processing).
    /// Bir yanıt alındığında, sunucu kendi eklediği Via'yı kaldırıp paketi bir sonraki hop'a iletir.
    pub fn strip_top_via(packet: &mut SipPacket) -> Option<Header> {
        if let Some(pos) = packet.headers.iter().position(|h| h.name == HeaderName::Via) {
            return Some(packet.headers.remove(pos));
        }
        None
    }

    /// Pakete NAT uyumlu (rport, received) parametrelerini işler.
    /// SBC ve Proxy servisleri, gelen isteğin kaynağını Via başlığına işlemelidir.
    pub fn fix_nat_via(packet: &mut SipPacket, src_addr: SocketAddr) {
        if let Some(via_header) = packet.headers.iter_mut().find(|h| h.name == HeaderName::Via) {
            let mut new_val = via_header.value.clone();
            
            // Received parametresi
            if !new_val.contains("received=") {
                new_val.push_str(&format!(";received={}", src_addr.ip()));
            }
            
            // Rport parametresi (RFC 3581)
            // Eğer "rport" varsa ama değeri yoksa veya hiç yoksa ekle.
            if new_val.contains(";rport") && !new_val.contains(";rport=") {
                 new_val = new_val.replace(";rport", &format!(";rport={}", src_addr.port()));
            } else if !new_val.contains("rport") {
                 new_val.push_str(&format!(";rport={}", src_addr.port()));
            }
            
            via_header.value = new_val;
        }
    }

    /// Paketin en başına Record-Route başlığı ekler (Proxy/SBC Persistence).
    /// Loose Routing (lr) parametresi ile.
    pub fn add_record_route(packet: &mut SipPacket, host: &str, port: u16) {
        let value = format!("<sip:{}:{};lr>", host, port);
        // Record-Route, Via'dan sonra gelmelidir ama basitlik adına başa ekliyoruz.
        // Via eklenmeden önce çağrılırsa doğru yerleşir.
        packet.headers.insert(0, Header::new(HeaderName::RecordRoute, value));
    }

    /// Yanıtın döneceği adresi `Via` başlığından çözer (RFC 3261 + NAT Traversal).
    pub fn resolve_response_target(via_val: &str, default_port: u16) -> Option<SocketAddr> {
        let parts: Vec<&str> = via_val.split_whitespace().collect();
        if parts.len() < 2 { return None; }
        
        let protocol_part = parts[1];
        let params: Vec<&str> = protocol_part.split(';').collect();
        
        let mut rport: Option<u16> = None;
        let mut received: Option<String> = None;

        for param in &params[1..] {
            let p_trim = param.trim();
            if let Some((k, v)) = p_trim.split_once('=') {
                if k == "received" { received = Some(v.to_string()); }
                if k == "rport" { rport = v.parse::<u16>().ok(); }
            }
        }

        // Eğer 'received' varsa, bu dış IP'dir. 
        // Eğer 'rport' varsa, bu dış porttur.
        if let Some(ip) = received {
            let port = rport.unwrap_or(default_port);
            return format!("{}:{}", ip, port).parse().ok();
        }

        // Fallback: Standart host:port parse
        let host_port = params[0];
        if !host_port.contains(':') {
             format!("{}:{}", host_port, default_port).parse().ok()
        } else {
             host_port.parse().ok()
        }
    }
    
    /// Standart bir Via başlığı oluşturur (Eski metod - backward compatibility)
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

    /// Standart bir Record-Route başlığı oluşturur (Eski metod - backward compatibility)
    pub fn build_record_route(host: &str, port: u16) -> Header {
        let value = format!("<sip:{}:{};lr>", host, port);
        Header::new(HeaderName::RecordRoute, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::{SipPacket, Method};

    #[test]
    fn test_add_via() {
        let mut pkt = SipPacket::new_request(Method::Invite, "sip:test".to_string());
        SipRouter::add_via(&mut pkt, "1.2.3.4", 5060, "UDP");
        assert_eq!(pkt.headers[0].name, HeaderName::Via);
        assert!(pkt.headers[0].value.contains("1.2.3.4:5060"));
    }

    #[test]
    fn test_strip_via() {
        let mut pkt = SipPacket::new_response(200, "OK".to_string());
        SipRouter::add_via(&mut pkt, "1.2.3.4", 5060, "UDP");
        assert!(!pkt.headers.is_empty());
        SipRouter::strip_top_via(&mut pkt);
        assert!(pkt.headers.is_empty()); // Başka header yoksa boş olmalı
    }
}