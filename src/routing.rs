// sentiric-sip-core/src/routing.rs

use crate::header::{Header, HeaderName};
use crate::packet::SipPacket;
use crate::utils::generate_branch_id;
use std::net::SocketAddr;

/// SIP Mesajlarını yönlendirmek ve manipüle etmek için merkezi mantık.
pub struct SipRouter;

impl SipRouter {
    /// Paketin en başına yeni bir Via başlığı ekler (Client Transaction).
    pub fn add_via(packet: &mut SipPacket, host: &str, port: u16, transport: &str) {
        let header = Self::build_via(host, port, transport);
        packet.headers.insert(0, header);
    }

    /// Paketin en üstündeki Via başlığını kaldırır (Response Processing).
    pub fn strip_top_via(packet: &mut SipPacket) -> Option<Header> {
        if let Some(pos) = packet.headers.iter().position(|h| h.name == HeaderName::Via) {
            return Some(packet.headers.remove(pos));
        }
        None
    }

    /// Pakete NAT uyumlu (rport, received) parametrelerini işler.
    pub fn fix_nat_via(packet: &mut SipPacket, src_addr: SocketAddr) {
        if let Some(via_header) = packet.headers.iter_mut().find(|h| h.name == HeaderName::Via) {
            let mut new_val = via_header.value.clone();
            
            if !new_val.contains("received=") {
                new_val.push_str(&format!(";received={}", src_addr.ip()));
            }
            
            if new_val.contains(";rport") && !new_val.contains(";rport=") {
                 new_val = new_val.replace(";rport", &format!(";rport={}", src_addr.port()));
            } else if !new_val.contains("rport") {
                 new_val.push_str(&format!(";rport={}", src_addr.port()));
            }
            
            via_header.value = new_val;
        }
    }

    /// Paketin en başına Record-Route başlığı ekler.
    pub fn add_record_route(packet: &mut SipPacket, host: &str, port: u16) {
        let header = Self::build_record_route(host, port);
        packet.headers.insert(0, header);
    }

    /// Yanıtın döneceği adresi `Via` başlığından çözer.
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

        if let Some(ip) = received {
            let port = rport.unwrap_or(default_port);
            return format!("{}:{}", ip, port).parse().ok();
        }

        let host_port = params[0];
        if !host_port.contains(':') {
             format!("{}:{}", host_port, default_port).parse().ok()
        } else {
             host_port.parse().ok()
        }
    }

    /// Loop Detection (Döngü Tespiti)
    /// Proxy'nin kendi imzasını Via başlıklarında arar.
    pub fn detect_loop(packet: &SipPacket, own_host: &str, own_port: u16) -> bool {
        let signature = format!("{}:{}", own_host, own_port);
        for h in &packet.headers {
            if h.name == HeaderName::Via && h.value.contains(&signature) {
                return true;
            }
        }
        false
    }

    /// Max-Forwards değerini kontrol eder ve azaltır.
    /// Eğer 0'a ulaşırsa hata döner.
    pub fn decrement_max_forwards(packet: &mut SipPacket) -> Result<(), ()> {
        let mut mf_idx = None;
        let mut mf_val = 70; // Varsayılan

        for (i, h) in packet.headers.iter().enumerate() {
            if h.name == HeaderName::MaxForwards {
                if let Ok(v) = h.value.parse::<i32>() {
                    mf_val = v;
                    mf_idx = Some(i);
                }
                break;
            }
        }

        mf_val -= 1;
        
        if mf_val <= 0 {
            return Err(());
        }

        if let Some(idx) = mf_idx {
            packet.headers[idx].value = mf_val.to_string();
        } else {
            packet.headers.push(Header::new(HeaderName::MaxForwards, mf_val.to_string()));
        }
        
        Ok(())
    }

    // --- HELPER BUILDERS (Restore Edildi) ---

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

    /// Standart bir Record-Route başlığı oluşturur.
    pub fn build_record_route(host: &str, port: u16) -> Header {
        let value = format!("<sip:{}:{};lr>", host, port);
        Header::new(HeaderName::RecordRoute, value)
    }
}