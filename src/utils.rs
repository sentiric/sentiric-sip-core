// sentiric-sip-core/src/utils.rs

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::SocketAddr;
use crate::packet::SipPacket;
use crate::header::{Header, HeaderName};

pub fn generate_branch_id() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("z9hG4bK{:x}", now)
}

pub fn generate_tag(seed: &str) -> String {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    now.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

static AOR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\s*"?([^"]*)"?\s*<sips?:([^>]+)>|sips?:([\w.-]+@[\w.-]+)(;[^>]+)?"#).unwrap()
});

pub fn extract_aor(raw_val: &str) -> String {
    if let Some(caps) = AOR_REGEX.captures(raw_val) {
        if let Some(addr_spec) = caps.get(2).or(caps.get(3)) {
            let mut aor = addr_spec.as_str().to_string();
            if let Some(semi_pos) = aor.find(';') { aor.truncate(semi_pos); }
            return aor;
        }
    }
    raw_val.replace('<', "").replace('>', "").trim().to_string()
}

pub fn extract_username_from_uri(uri: &str) -> String {
    let clean = uri.trim();
    let without_scheme = if let Some(idx) = clean.find(':') { &clean[idx+1..] } else { clean };
    let user_part = if let Some(idx) = without_scheme.find('@') { &without_scheme[..idx] } else { without_scheme };
    
    let pure_user = if let Some(idx) = user_part.find(';') { &user_part[..idx] } else { user_part };
    pure_user.replace('<', "").replace('>', "")
}

pub fn extract_socket_addr(uri: &str) -> Option<SocketAddr> {
    let mut s = uri.trim();
    s = s.trim_start_matches('<').trim_end_matches('>');
    if s.starts_with("sip:") { s = &s[4..]; } else if s.starts_with("sips:") { s = &s[5..]; }
    
    let host_port_part = if let Some(semi_idx) = s.find(';') { &s[..semi_idx] } else { s };
    let host_port = if let Some(at_idx) = host_port_part.find('@') { &host_port_part[at_idx + 1..] } else { host_port_part };
    
    if !host_port.contains(':') { 
        format!("{}:5060", host_port).parse().ok() 
    } else { 
        host_port.parse().ok() 
    }
}

// --- [FIX]: STRICT TOPOLOGY HIDING ---
// Artık sadece IP değil, PORT eşleşmesini de kontrol ediyoruz.
pub fn apply_topology_hiding(packet: &mut SipPacket, public_ip: &str, public_port: u16) -> bool {
    let old_contact_val = match packet.get_header_value(HeaderName::Contact) {
        Some(v) => v.clone(),
        None => {
            let new_contact = format!("<sip:sbc@{}:{}>", public_ip, public_port);
            packet.headers.push(Header::new(HeaderName::Contact, new_contact));
            return true;
        }
    };

    // [YENİ MANTIK]: Beklenen "PublicIP:PublicPort" kombinasyonunu tam olarak arıyoruz.
    // Eğer port farklıysa (örn: 13084), IP aynı olsa bile rewrite yapılmalı.
    let expected_signature = format!("{}:{}", public_ip, public_port);
    
    if old_contact_val.contains(&expected_signature) {
        return false; // Zaten doğru (34.122...:5060)
    }

    // Kullanıcı adını koru
    let user_part = extract_username_from_uri(&old_contact_val);
    let final_user = if user_part.is_empty() { "sbc" } else { &user_part };

    let new_contact = format!("<sip:{}@{}:{}>", final_user, public_ip, public_port);

    for h in &mut packet.headers {
        if h.name == HeaderName::Contact {
            h.value = new_contact.clone();
            return true;
        }
    }
    
    false
}