// sentiric-sip-core/src/sdp.rs

use regex::Regex;
use once_cell::sync::Lazy;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

// [FIX]: Regex güncellemeleri
// (?m) bayrağı "multiline" modunu açar, böylece ^ satır başını yakalar.
// c= satırı: IP adresini yakalar.
static SDP_CONNECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^c=IN IP4 [^\r\n]+").unwrap());

// [CRITICAL FIX]: m=audio satırı.
// Grup 1 (\d+): Eski Port (Örn: 40000)
// Grup 2 (.*): Satırın geri kalanı (Örn: " RTP/AVP 18 101") -> Bunu korumalıyız!
static SDP_AUDIO_MEDIA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^m=audio (\d+)(.*)").unwrap());

pub struct SdpManipulator;

impl SdpManipulator {
    /// Gelen SDP gövdesindeki IP ve Port bilgilerini SBC'nin bilgileriyle değiştirir.
    pub fn rewrite_connection_info(sdp_body: &[u8], new_ip: &str, new_port: u16) -> Option<Vec<u8>> {
        let sdp_str = match std::str::from_utf8(sdp_body) {
            Ok(s) => s,
            Err(_) => return None,
        };

        // 1. IP Adreslerini Değiştir (Global ve Media seviyesindeki c= satırları)
        // c=IN IP4 1.2.3.4 -> c=IN IP4 <SBC_PUBLIC_IP>
        let sdp_ip_replaced = SDP_CONNECTION_REGEX.replace_all(sdp_str, format!("c=IN IP4 {}", new_ip));
        
        // 2. Portu Değiştir (m=audio satırı)
        // m=audio 12345 RTP/AVP 18 101 -> m=audio <RELAY_PORT> RTP/AVP 18 101
        // $2, regex'teki ikinci yakalama grubunu (protokol ve kodekler) korur.
        let sdp_final = SDP_AUDIO_MEDIA_REGEX.replace(&sdp_ip_replaced, format!("m=audio {} $2", new_port));

        // Eğer hiçbir değişiklik olmadıysa (regex eşleşmediyse), None dön.
        if sdp_str == sdp_final {
            None 
        } else {
            Some(sdp_final.as_bytes().to_vec())
        }
    }
}

// --- SDP Builder (Yeni Çağrılar İçin) ---

#[derive(Debug, Clone)]
pub struct Codec {
    pub id: u8,
    pub name: String,
    pub rate: u32,
    pub params: Option<String>,
}

pub struct SdpBuilder {
    ip_address: String,
    port: u16,
    codecs: Vec<Codec>,
    ptime: u8,
    session_id: u64, // [FIX]: Dinamik Session ID için eklendi
}

impl SdpBuilder {
    pub fn new(ip_address: String, port: u16) -> Self {
        // Session ID olarak şimdiki zamanı (saniye) kullan (RFC 3261 önerisi)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        Self {
            ip_address,
            port,
            codecs: Vec::new(),
            ptime: 20,
            session_id: now,
        }
    }

    pub fn add_codec(mut self, id: u8, name: &str, rate: u32, params: Option<&str>) -> Self {
        self.codecs.push(Codec {
            id,
            name: name.to_string(),
            rate,
            params: params.map(|s| s.to_string()),
        });
        self
    }

    /// Platform standartlarına uygun kodekleri ekler.
    /// Şu an için G.729 (Bandwidth tasarrufu) ve PCMU (Evrensel uyumluluk) aktiftir.
    pub fn with_standard_codecs(mut self) -> Self {
        // 1. G.729 (Payload 18) - Öncelikli Kodek
        self = self.add_codec(18, "G729", 8000, Some("annexb=no"));
        
        // 2. PCMU (Payload 0) - Yedek/Fallback (Neredeyse her cihaz destekler)
        self = self.add_codec(0, "PCMU", 8000, None);
        
        // 3. DTMF (Payload 101) - Tuşlama (IVR) için zorunlu
        self = self.add_codec(101, "telephone-event", 8000, Some("0-16"));
        
        self
    }

    pub fn build(self) -> String {
        let mut sdp = String::new();
        
        // Version
        let _ = write!(sdp, "v=0\r\n");
        
        // Origin (RFC 3261: Username, SessionID, Version, NetType, AddrType, Address)
        // SessionID ve Version dinamik olmalı ki cihazlar değişikliği algılasın.
        let _ = write!(sdp, "o=- {} {} IN IP4 {}\r\n", self.session_id, self.session_id, self.ip_address);
        
        // Session Name
        let _ = write!(sdp, "s=Sentiric\r\n");
        
        // Connection Data (Global)
        let _ = write!(sdp, "c=IN IP4 {}\r\n", self.ip_address);
        
        // Timing (Start/Stop - 0 0 limitsiz demek)
        let _ = write!(sdp, "t=0 0\r\n");
        
        // Media Description
        let codec_ids: Vec<String> = self.codecs.iter().map(|c| c.id.to_string()).collect();
        let _ = write!(sdp, "m=audio {} RTP/AVP {}\r\n", self.port, codec_ids.join(" "));

        // [FIX]: a=rtcp Attribute (RTP port + 1)
        // Bazı NAT cihazları ve SBC'ler RTCP portunu açıkça görmek ister.
        let _ = write!(sdp, "a=rtcp:{} IN IP4 {}\r\n", self.port + 1, self.ip_address);

        // Attributes (Codecs & FMTP)
        for codec in &self.codecs {
            let _ = write!(sdp, "a=rtpmap:{} {}/{}\r\n", codec.id, codec.name, codec.rate);
            if let Some(params) = &codec.params {
                let _ = write!(sdp, "a=fmtp:{} {}\r\n", codec.id, params);
            }
        }
        
        // Packet Time (20ms standarttır)
        let _ = write!(sdp, "a=ptime:{}\r\n", self.ptime);
        
        // Direction
        let _ = write!(sdp, "a=sendrecv\r\n");
        
        sdp
    }
}