// sentiric-sip-core/src/sdp.rs
// ✅ YENİ: SDP Codec Negotiation

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Codec {
    pub id: u8,
    pub name: String,
    pub rate: u32,
}

use once_cell::sync::Lazy;

/// Platform tarafından desteklenen codec'ler
pub static SUPPORTED_CODECS: Lazy<Vec<Codec>> = Lazy::new(|| vec![
    Codec { id: 0, name: "PCMU".to_string(), rate: 8000 },
    Codec { id: 8, name: "PCMA".to_string(), rate: 8000 },
]);

/// İstemci tarafından önerilen codec listesinden platform destekleyeni seç
///
/// # Example
/// ```
/// use sentiric_sip_core::sdp::{Codec, negotiate_codec};
///
/// let offered = vec![
///     Codec { id: 9, name: "G722".to_string(), rate: 8000 },
///     Codec { id: 0, name: "PCMU".to_string(), rate: 8000 },
/// ];
///
/// let selected = negotiate_codec(&offered).expect("Codec bulunamadı");
/// assert_eq!(selected.id, 0); // PCMU seçildi
/// ```
pub fn negotiate_codec(offered: &[Codec]) -> Option<Codec> {
    offered.iter()
        .find(|c| SUPPORTED_CODECS.contains(c))
        .cloned()
}

/// SDP formatında codec listesi oluştur
///
/// # Example Output
/// ```text
/// m=audio 10002 RTP/AVP 0 8
/// a=rtpmap:0 PCMU/8000
/// a=rtpmap:8 PCMA/8000
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_codec_success() {
        let offered = vec![
            Codec { id: 9, name: "G722".to_string(), rate: 8000 },
            Codec { id: 0, name: "PCMU".to_string(), rate: 8000 },
        ];
        
        let result = negotiate_codec(&offered);
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, 0);
    }

    #[test]
    fn test_negotiate_codec_no_match() {
        let offered = vec![
            Codec { id: 9, name: "G722".to_string(), rate: 8000 },
        ];
        
        let result = negotiate_codec(&offered);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_sdp_media_line() {
        let line = build_sdp_media_line(10002);
        assert_eq!(line, "m=audio 10002 RTP/AVP 0 8");
    }
}
