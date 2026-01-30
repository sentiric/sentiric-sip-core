// sentiric-sip-core/src/utils.rs

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

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

// Regex ile URI ayrıştırma. Sadece bir kez derlenir.
// GÜNCELLEME: ;user=phone gibi parametreleri de temizleyecek şekilde genişletildi.
static AOR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\s*"?([^"]*)"?\s*<sips?:([^>]+)>|sips?:([\w.-]+@[\w.-]+)(;[^>]+)?"#).unwrap()
});

pub fn extract_aor(raw_val: &str) -> String {
    if let Some(caps) = AOR_REGEX.captures(raw_val) {
        // "<sip:user@domain;params>" formatını yakala
        if let Some(addr_spec) = caps.get(2) {
            let mut aor = addr_spec.as_str().to_string();
            // Portu ve parametreleri temizle
            if let Some(at_pos) = aor.find('@') {
                if let Some(colon_pos) = aor[at_pos..].find(':') {
                    aor.truncate(at_pos + colon_pos);
                }
            }
            if let Some(semi_pos) = aor.find(';') { // Yeni: Parametreleri temizle
                aor.truncate(semi_pos);
            }
            return aor;
        }
        // "sip:user@domain;params" formatını yakala
        if let Some(addr_spec) = caps.get(3) {
            let mut aor = addr_spec.as_str().to_string();
            if let Some(semi_pos) = aor.find(';') { // Yeni: Parametreleri temizle
                aor.truncate(semi_pos);
            }
            return aor;
        }
    }
    // Hiçbir şey eşleşmezse, en iyi tahmin olarak ham değeri temizle.
    raw_val
        .replace('<', "")
        .replace('>', "")
        .trim()
        .to_string()
}

/// URI'den sadece kullanıcı adını çeker.
/// Örnek: "sip:2001@domain.com" -> "2001"
/// GÜNCELLEME: ;user=phone gibi parametreleri de temizleyecek şekilde genişletildi.
pub fn extract_username_from_uri(uri: &str) -> String {
    let clean = uri.trim();
    // sip: prefixini at
    let without_scheme = if let Some(idx) = clean.find(':') {
        &clean[idx+1..]
    } else {
        clean
    };
    
    // @ işaretine kadar al
    let user_part = if let Some(idx) = without_scheme.find('@') {
        &without_scheme[..idx]
    } else {
        without_scheme // Domain yoksa kendisi kullanıcıdır (nadiren)
    };

    // Semikolon ve sonrası parametreleri temizle (örn: ;user=phone)
    if let Some(idx) = user_part.find(';') {
        user_part[..idx].to_string()
    } else {
        user_part.to_string()
    }
    .replace('<', "") // Köşeli parantez kalıntılarını temizle
    .replace('>', "")
}