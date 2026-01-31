// sentiric-sip-core/src/sdp.rs

use regex::Regex;
use once_cell::sync::Lazy;
use std::fmt::Write;

static SDP_CONNECTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"c=IN IP4 \d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap());
static SDP_AUDIO_MEDIA_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"m=audio (\d+)").unwrap());

pub struct SdpManipulator;

impl SdpManipulator {
    pub fn rewrite_connection_info(sdp_body: &[u8], new_ip: &str, new_port: u16) -> Option<Vec<u8>> {
        let sdp_str = match std::str::from_utf8(sdp_body) {
            Ok(s) => s,
            Err(_) => return None,
        };
        let sdp_ip_replaced = SDP_CONNECTION_REGEX.replace_all(sdp_str, format!("c=IN IP4 {}", new_ip));
        let sdp_final = SDP_AUDIO_MEDIA_REGEX.replace(&sdp_ip_replaced, format!("m=audio {}", new_port));

        if sdp_str != sdp_final { Some(sdp_final.as_bytes().to_vec()) } else { None }
    }
}

// --- YENİ EKLENEN: SdpBuilder ---
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
}

impl SdpBuilder {
    pub fn new(ip_address: String, port: u16) -> Self {
        Self { ip_address, port, codecs: Vec::new(), ptime: 20 }
    }

    pub fn add_codec(mut self, id: u8, name: &str, rate: u32, params: Option<&str>) -> Self {
        self.codecs.push(Codec {
            id, name: name.to_string(), rate, params: params.map(|s| s.to_string()),
        });
        self
    }

    pub fn with_standard_codecs(mut self) -> Self {
        self = self.add_codec(18, "G729", 8000, Some("annexb=no"));
        self = self.add_codec(8, "PCMA", 8000, None);
        self = self.add_codec(0, "PCMU", 8000, None);
        self = self.add_codec(101, "telephone-event", 8000, Some("0-16"));
        self
    }

    pub fn build(self) -> String {
        let mut sdp = String::new();
        let _ = write!(sdp, "v=0\r\n");
        let _ = write!(sdp, "o=- 123456 123456 IN IP4 {}\r\n", self.ip_address);
        let _ = write!(sdp, "s=Sentiric\r\n");
        let _ = write!(sdp, "c=IN IP4 {}\r\n", self.ip_address);
        let _ = write!(sdp, "t=0 0\r\n");
        
        let codec_ids: Vec<String> = self.codecs.iter().map(|c| c.id.to_string()).collect();
        let _ = write!(sdp, "m=audio {} RTP/AVP {}\r\n", self.port, codec_ids.join(" "));

        for codec in &self.codecs {
            let _ = write!(sdp, "a=rtpmap:{} {}/{}\r\n", codec.id, codec.name, codec.rate);
            if let Some(params) = &codec.params {
                let _ = write!(sdp, "a=fmtp:{} {}\r\n", codec.id, params);
            }
        }
        let _ = write!(sdp, "a=ptime:{}\r\n", self.ptime);
        let _ = write!(sdp, "a=sendrecv\r\n");
        sdp
    }
}