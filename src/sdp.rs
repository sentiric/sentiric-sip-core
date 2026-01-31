// sentiric-sip-core/src/sdp.rs

use regex::Regex;
use once_cell::sync::Lazy;

// Performans için regexler compile-time (lazy) hazırlanır.
static SDP_CONNECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"c=IN IP4 \d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap());
static SDP_AUDIO_MEDIA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"m=audio (\d+)").unwrap());

/// SDP işlemlerini yöneten yardımcı yapı.
pub struct SdpManipulator;

impl SdpManipulator {
    /// SDP body'si içindeki Connection IP (c=) ve Audio Port (m=audio) bilgilerini değiştirir.
    /// SBC'nin "Topology Hiding" özelliği için kritiktir.
    pub fn rewrite_connection_info(sdp_body: &[u8], new_ip: &str, new_port: u16) -> Option<Vec<u8>> {
        let sdp_str = match std::str::from_utf8(sdp_body) {
            Ok(s) => s,
            Err(_) => return None,
        };

        // c=IN IP4 x.x.x.x -> c=IN IP4 <new_ip>
        let sdp_ip_replaced = SDP_CONNECTION_REGEX.replace_all(sdp_str, format!("c=IN IP4 {}", new_ip));
        
        // m=audio <port> ... -> m=audio <new_port> ...
        let sdp_final = SDP_AUDIO_MEDIA_REGEX.replace(&sdp_ip_replaced, format!("m=audio {}", new_port));

        if sdp_str != sdp_final {
            Some(sdp_final.as_bytes().to_vec())
        } else {
            None
        }
    }
}

// --- Mevcut Codec Negotiation kodları aşağıda korunur ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Codec {
    pub id: u8,
    pub name: String,
    pub rate: u32,
}

pub static SUPPORTED_CODECS: Lazy<Vec<Codec>> = Lazy::new(|| vec![
    Codec { id: 0, name: "PCMU".to_string(), rate: 8000 },
    Codec { id: 8, name: "PCMA".to_string(), rate: 8000 },
]);

pub fn negotiate_codec(offered: &[Codec]) -> Option<Codec> {
    offered.iter()
        .find(|c| SUPPORTED_CODECS.contains(c))
        .cloned()
}

pub fn build_sdp_media_line(port: u16) -> String {
    let codec_ids: Vec<String> = SUPPORTED_CODECS.iter()
        .map(|c| c.id.to_string())
        .collect();
    
    format!("m=audio {} RTP/AVP {}", port, codec_ids.join(" "))
}

pub fn build_rtpmap_attributes() -> Vec<String> {
    SUPPORTED_CODECS.iter()
        .map(|c| format!("a=rtpmap:{} {}/{}", c.id, c.name, c.rate))
        .collect()
}