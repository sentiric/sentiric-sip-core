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
    // DEBUG: Gelen ham veriyi görelim
    println!("DEBUG_AOR_RAW: '{}'", raw_val);

    // 1. Basit Temizlik (Trim ve < > kaldır)
    let mut clean = raw_val.trim().to_string();
    clean = clean.replace("<", "").replace(">", "");

    // 2. "sip:" ile başlamıyorsa ekle (Standardizasyon)
    if !clean.starts_with("sip:") && !clean.starts_with("sips:") {
         // Eğer raw değer "Azmi" <sip:1001...> formatındaysa, replace sonrası bozulmuş olabilir.
         // Bu yüzden tekrar sip: arayalım.
         if let Some(idx) = clean.find("sip:") {
             clean = clean[idx..].to_string();
         } else {
             // Hiç sip: yoksa, biz ekleyelim (Blind fix)
             clean = format!("sip:{}", clean);
         }
    }

    // 3. Parametreleri at (; sonrası çöp)
    if let Some(idx) = clean.find(';') {
        clean = clean[..idx].to_string();
    }

    // DEBUG: Çıkan sonucu görelim
    println!("DEBUG_AOR_CLEAN: '{}'", clean);

    clean
}