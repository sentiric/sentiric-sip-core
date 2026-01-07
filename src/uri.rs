// src/uri.rs

#[derive(Debug, Clone)]
pub struct SipUri {
    pub scheme: String, // sip or sips
    pub user: Option<String>,
    pub host: String,
    pub port: Option<u16>,
}

// Şimdilik karmaşık URI parsing'e gerek yok, string operasyonları ile halledeceğiz.