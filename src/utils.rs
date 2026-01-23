// sentiric-sip-core/src/utils.rs

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

pub fn extract_aor(raw_val: &str) -> String {
    // 1. ÖNCE TEMİZLİK: < ve > karakterlerinden kurtul.
    let clean_str = raw_val.replace('<', "").replace('>', "");
    
    // 2. "sip:" veya "sips:" başlangıcını bul ve atla.
    let no_scheme = if let Some(stripped) = clean_str.strip_prefix("sip:") {
        stripped
    } else if let Some(stripped) = clean_str.strip_prefix("sips:") {
        stripped
    } else {
        &clean_str
    };

    // 3. Parametreleri (; ile başlar) at
    let end = no_scheme.find(';').unwrap_or(no_scheme.len());
    let clean_uri = &no_scheme[..end];
    
    // 4. Port varsa temizle
    if let Some(at_pos) = clean_uri.find('@') {
        if let Some(colon_pos) = clean_uri[at_pos..].find(':') {
            let absolute_colon = at_pos + colon_pos;
            return clean_uri[..absolute_colon].to_string();
        }
    } else if let Some(colon_pos) = clean_uri.find(':') {
        return clean_uri[..colon_pos].to_string();
    }

    clean_uri.to_string()
}