// sentiric-sip-core/src/utils.rs

use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
// Regex bağımlılığına gerek kalmadı, manuel parsing daha güvenli ve hızlı.

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
    // Örnek Girdiler:
    // 1. "Azmi" <sip:1001@1.2.3.4>;tag=...
    // 2. <sip:1001@1.2.3.4>
    // 3. sip:1001@1.2.3.4
    // 4. 1001@1.2.3.4

    // Adım 1: "sip:" ön ekini bul
    let start_idx = if let Some(idx) = uri.find("sip:") {
        idx + 4
    } else {
        0
    };

    let remainder = &uri[start_idx..];

    // Adım 2: Bitiş karakterlerini bul (>, ;, boşluk)
    // En erken hangisi gelirse orada kes.
    let end_idx = remainder.find(|c| c == '>' || c == ';' || c == ' ').unwrap_or(remainder.len());
    
    let mut clean_aor = remainder[..end_idx].to_string();

    // Adım 3: Port temizliği (İsteğe bağlı, AOR genelde portsuzdur ama bazen portlu kaydedilir)
    // AOR standardı gereği user@domain olmalı.
    
    // Eğer user part varsa (user@domain)
    if let Some(at_idx) = clean_aor.find('@') {
         // Domain kısmında port var mı? (user@domain:port)
         // IPv6 ([...]) hariç tutmak için basit kontrol: son ':' '@'den sonra mı?
         if let Some(colon_idx) = clean_aor.rfind(':') {
             if colon_idx > at_idx {
                 // Portu at
                 clean_aor = clean_aor[..colon_idx].to_string();
             }
         }
    }

    // Son temizlik (Trim)
    clean_aor.trim().to_string()
}