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

pub fn extract_aor(uri: &str) -> String {
    // 1. "sip:" veya "sips:" başlangıcını bul. Yoksa stringin başından başla.
    let start_idx = uri.find("sip").unwrap_or(0);
    let working_slice = &uri[start_idx..];

    // 2. İlk yasaklı karakteri bul (Bitiş noktası)
    // > : Header bitişi (<sip:...>)
    // ; : Parametre başlangıcı (sip:...;transport=udp)
    // ? : Header parametreleri (sip:...?)
    // Boşluk : Hatalı format
    let end_idx = working_slice.find(|c| c == '>' || c == ';' || c == '?' || c == ' ').unwrap_or(working_slice.len());

    // 3. Kes ve Döndür
    let clean_aor = &working_slice[..end_idx];
    
    // Güvenlik: Eğer sonuçta hala @ yoksa veya boşsa, ham halini döndür (logda hatayı görmek için)
    if !clean_aor.contains('@') && !clean_aor.is_empty() {
         return clean_aor.to_string();
    }

    clean_aor.to_string()
}