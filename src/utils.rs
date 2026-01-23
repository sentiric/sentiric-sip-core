// sentiric-sip-core/src/utils.rs

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use regex::Regex;
use once_cell::sync::Lazy;

// Regex güncellendi: < ve > karakterlerini dışlayan gruplar
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
    // 1. Temizlik: Başındaki/Sonundaki boşlukları ve < > karakterlerini at.
    let clean_uri = uri.trim().trim_matches(|c| c == '<' || c == '>');

    // 2. Regex ile Ayrıştırma
    if let Some(caps) = SIP_URI_RE.captures(clean_uri) {
        let user = caps.get(1).map_or("", |m| m.as_str());
        let host = caps.get(2).map_or("", |m| m.as_str());
        
        // Host içinde port varsa temizle (örn: host:5060 -> host)
        let clean_host = if let Some(idx) = host.find(':') {
             &host[..idx]
        } else {
            host
        };
        return format!("{}@{}", user, clean_host);
    }

    // 3. Fallback (Manuel parsing) - Regex başarısız olursa
    let start = clean_uri.find("sip:").map(|i| i + 4).unwrap_or(0);
    let end = clean_uri.find(';').unwrap_or(clean_uri.len());
    let bare = &clean_uri[start..end];
    
    // Port temizliği
    if let Some(colon) = bare.rfind(':') {
        if let Some(at) = bare.find('@') {
            if colon > at {
                return bare[..colon].to_string();
            }
        }
    }
    bare.to_string()
}