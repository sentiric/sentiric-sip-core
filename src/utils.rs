// sentiric-sip-core/src/utils.rs

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use regex::Regex;
use once_cell::sync::Lazy;

// [FIX] Daha sağlam Regex: <sip:...> veya sip:... formatlarını destekler
static SIP_URI_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"sip:([^@;>]+)@([^;>]+)").unwrap());

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

pub fn extract_aor(uri: &str) -> String {
    // Regex ile temiz ayrıştırma
    if let Some(caps) = SIP_URI_RE.captures(uri) {
        let user = caps.get(1).map_or("", |m| m.as_str());
        let host = caps.get(2).map_or("", |m| m.as_str());
        
        // Host içinde port varsa temizle (örn: host:5060)
        let clean_host = if let Some(idx) = host.find(':') {
             &host[..idx]
        } else {
            host
        };
        return format!("{}@{}", user, clean_host);
    }

    // Fallback (Manuel parsing) - Daha güvenli hale getirildi
    let start = uri.find("sip:").map(|i| i + 4).unwrap_or(0);
    // Bitiş karakterleri: ; veya > veya string sonu
    let end = uri.find(|c| c == ';' || c == '>').unwrap_or(uri.len());
    
    let clean = &uri[start..end];
    
    // Port varsa temizle
    if let Some(colon) = clean.rfind(':') {
        if let Some(at) = clean.find('@') {
            if colon > at {
                return clean[..colon].to_string();
            }
        }
    }
    clean.to_string()
}