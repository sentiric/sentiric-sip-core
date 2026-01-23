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
static AOR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\s*"?([^"]*)"?\s*<sips?:([^>]+)>|sips?:([\w.-]+@[\w.-]+)"#).unwrap()
});

pub fn extract_aor(raw_val: &str) -> String {
    if let Some(caps) = AOR_REGEX.captures(raw_val) {
        // "<sip:user@domain>" formatını yakala
        if let Some(addr_spec) = caps.get(2) {
            let mut aor = addr_spec.as_str().to_string();
            // Portu temizle
            if let Some(at_pos) = aor.find('@') {
                if let Some(colon_pos) = aor[at_pos..].find(':') {
                    aor.truncate(at_pos + colon_pos);
                }
            }
            return aor;
        }
        // "sip:user@domain" formatını yakala
        if let Some(addr_spec) = caps.get(3) {
            return addr_spec.as_str().to_string();
        }
    }
    // Hiçbir şey eşleşmezse, en iyi tahmin olarak ham değeri temizle.
    raw_val
        .replace('<', "")
        .replace('>', "")
        .trim()
        .to_string()
}