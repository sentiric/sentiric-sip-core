// sentiric-sip-core/src/sdp.rs

use regex::Regex;
use once_cell::sync::Lazy;
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};

// [FIX]: Regex güncellemeleri
// (?m) bayrağı "multiline" modunu açar, böylece ^ satır başını yakalar.
static SDP_CONNECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^c=IN IP4 [^\r\n]+").unwrap());

// [CRITICAL FIX]: m=audio satırı.
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
        let sdp_ip_replaced = SDP_CONNECTION_REGEX.replace_all(sdp_str, format!("c=IN IP4 {}", new_ip));
        
        // 2. Portu Değiştir (m=audio satırı)
        let sdp_final = SDP_AUDIO_MEDIA_REGEX.replace(&sdp_ip_replaced, format!("m=audio {} $2", new_port));

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
    session_id: u64,
    enable_rtcp_attribute: bool, // [NEW] RTCP satırını kontrol etmek için
}

impl SdpBuilder {
    pub fn new(ip_address: String, port: u16) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        Self {
            ip_address,
            port,
            codecs: Vec::new(),
            ptime: 20, // Telekom standardı varsayılan: 20ms
            session_id: now,
            enable_rtcp_attribute: true,
        }
    }

    /// Paketleme süresini (ptime) ayarlar.
    /// Örn: G.729 için 20ms (2 frame) standarttır.
    pub fn with_ptime(mut self, ptime: u8) -> Self {
        self.ptime = ptime;
        self
    }

    /// RTCP attribute (a=rtcp:...) satırını ekler veya kaldırır.
    /// NAT arkasındaki karmaşık senaryolarda bu satırı kapatmak gerekebilir.
    pub fn with_rtcp(mut self, enabled: bool) -> Self {
        self.enable_rtcp_attribute = enabled;
        self
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
    /// Artık parametreleri explicit olarak belirliyoruz.
    pub fn with_standard_codecs(mut self) -> Self {
        // 1. G.729 (Payload 18) - Düşük Bant Genişliği
        self = self.add_codec(18, "G729", 8000, Some("annexb=no"));
        
        // 2. PCMA (Payload 8) - Avrupa Standardı (PCMU yerine PCMA öncelikli olabilir)
        self = self.add_codec(8, "PCMA", 8000, None);

        // 3. PCMU (Payload 0) - ABD Standardı
        self = self.add_codec(0, "PCMU", 8000, None);
        
        // 4. DTMF (Payload 101) - Tuşlama (IVR) için zorunlu
        self = self.add_codec(101, "telephone-event", 8000, Some("0-16"));
        
        self
    }

    pub fn build(self) -> String {
        let mut sdp = String::new();
        
        // v=0
        let _ = write!(sdp, "v=0\r\n");
        
        // o=- <SessionID> <Version> IN IP4 <Address>
        // Version alanını da session_id ile aynı yaparak her yeni SDP'de unique olmasını sağlıyoruz.
        let _ = write!(sdp, "o=- {} {} IN IP4 {}\r\n", self.session_id, self.session_id, self.ip_address);
        
        // s=
        let _ = write!(sdp, "s=Sentiric Media\r\n");
        
        // c=IN IP4 <Address>
        let _ = write!(sdp, "c=IN IP4 {}\r\n", self.ip_address);
        
        // t=0 0
        let _ = write!(sdp, "t=0 0\r\n");
        
        // m=audio <Port> RTP/AVP <CodecIDs>
        let codec_ids: Vec<String> = self.codecs.iter().map(|c| c.id.to_string()).collect();
        let _ = write!(sdp, "m=audio {} RTP/AVP {}\r\n", self.port, codec_ids.join(" "));

        // a=rtcp:<Port+1> IN IP4 <Address>
        if self.enable_rtcp_attribute {
            let _ = write!(sdp, "a=rtcp:{} IN IP4 {}\r\n", self.port + 1, self.ip_address);
        }

        // Codec Attributes
        for codec in &self.codecs {
            let _ = write!(sdp, "a=rtpmap:{} {}/{}\r\n", codec.id, codec.name, codec.rate);
            if let Some(params) = &codec.params {
                let _ = write!(sdp, "a=fmtp:{} {}\r\n", codec.id, params);
            }
        }
        
        // a=ptime:<ms>
        let _ = write!(sdp, "a=ptime:{}\r\n", self.ptime);
        
        // a=sendrecv
        let _ = write!(sdp, "a=sendrecv\r\n");
        
        sdp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdp_builder_standard() {
        let sdp = SdpBuilder::new("127.0.0.1".to_string(), 10000)
            .with_standard_codecs()
            .with_ptime(20)
            .build();
        
        assert!(sdp.contains("m=audio 10000 RTP/AVP 18 8 0 101"));
        assert!(sdp.contains("a=rtpmap:18 G729/8000"));
        assert!(sdp.contains("a=ptime:20"));
        assert!(sdp.contains("a=sendrecv"));
        // RTCP varsayılan olarak açık olmalı
        assert!(sdp.contains("a=rtcp:10001 IN IP4 127.0.0.1"));
    }

    #[test]
    fn test_sdp_builder_no_rtcp() {
        let sdp = SdpBuilder::new("127.0.0.1".to_string(), 10000)
            .with_standard_codecs()
            .with_rtcp(false)
            .build();
        
        assert!(!sdp.contains("a=rtcp:"));
    }
}