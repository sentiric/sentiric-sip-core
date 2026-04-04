// sentiric-sip-core/src/sdp.rs
use std::fmt::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn}; // GÖZLEMLENEBİLİRLİK EKLENDİ

pub struct SdpManipulator;

impl SdpManipulator {
    /// Boşluk ve format bağımsız SDP IP/Port değiştirici (v1.5.6 - Observable)
    pub fn rewrite_connection_info(
        sdp_body: &[u8],
        new_ip: &str,
        new_port: u16,
    ) -> Option<Vec<u8>> {
        let sdp_str = match std::str::from_utf8(sdp_body) {
            Ok(s) => s,
            Err(_) => {
                warn!(event = "SDP_UTF8_ERROR", "SDP body is not valid UTF-8");
                return None;
            }
        };

        let mut new_sdp = String::with_capacity(sdp_str.len() + 50);
        let mut modified = false;
        let mut old_ip = String::new();
        let mut old_port = String::new();

        for line in sdp_str.lines() {
            let trimmed = line.trim();

            // 1. Connection (c=) satırını değiştir
            if trimmed.starts_with("c=IN IP4") {
                old_ip = trimmed.split_whitespace().last().unwrap_or("").to_string();
                let _ = writeln!(new_sdp, "c=IN IP4 {}", new_ip);
                modified = true;
                continue;
            }

            // 2. Media (m=) satırını değiştir
            if trimmed.starts_with("m=audio") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 4 {
                    old_port = parts[1].to_string();
                    let proto = parts[2];
                    let payloads = parts[3..].join(" ");
                    let _ = writeln!(new_sdp, "m=audio {} {} {}", new_port, proto, payloads);
                    modified = true;
                    continue;
                }
            }

            // 3. Origin (o=) satırını değiştir (Strict client uyumluluğu için)
            if trimmed.starts_with("o=") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 6 && parts[4] == "IP4" {
                    let _ = writeln!(
                        new_sdp,
                        "{} {} {} {} {} {}",
                        parts[0], parts[1], parts[2], parts[3], parts[4], new_ip
                    );
                    modified = true;
                    continue;
                }
            }

            new_sdp.push_str(line);
            new_sdp.push_str("\r\n");
        }

        if modified {
            debug!(
                event="SDP_MUTATION_DETAIL",
                old_ip=%old_ip, new_ip=%new_ip,
                old_port=%old_port, new_port=%new_port,
                "SDP içeriği dönüştürüldü"
            );
            Some(new_sdp.into_bytes())
        } else {
            None
        }
    }
}

// --- B2BUA İÇİN BUILDER (DEĞİŞMEDİ) ---
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
    enable_rtcp_attribute: bool,
}

impl SdpBuilder {
    pub fn new(ip_address: String, port: u16) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            ip_address,
            port,
            codecs: Vec::new(),
            ptime: 20,
            session_id: now,
            enable_rtcp_attribute: true,
        }
    }

    pub fn with_ptime(mut self, ptime: u8) -> Self {
        self.ptime = ptime;
        self
    }
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

    pub fn build(self) -> String {
        let mut sdp = String::new();
        let _ = write!(sdp, "v=0\r\n");
        let _ = write!(
            sdp,
            "o=- {} {} IN IP4 {}\r\n",
            self.session_id, self.session_id, self.ip_address
        );
        let _ = write!(sdp, "s=Sentiric Media\r\n");
        let _ = write!(sdp, "c=IN IP4 {}\r\n", self.ip_address);
        let _ = write!(sdp, "t=0 0\r\n");
        let codec_ids: Vec<String> = self.codecs.iter().map(|c| c.id.to_string()).collect();
        let _ = write!(
            sdp,
            "m=audio {} RTP/AVP {}\r\n",
            self.port,
            codec_ids.join(" ")
        );
        if self.enable_rtcp_attribute {
            let _ = write!(
                sdp,
                "a=rtcp:{} IN IP4 {}\r\n",
                self.port + 1,
                self.ip_address
            );
        }
        for codec in &self.codecs {
            let _ = write!(
                sdp,
                "a=rtpmap:{} {}/{}\r\n",
                codec.id, codec.name, codec.rate
            );
            if let Some(params) = &codec.params {
                let _ = write!(sdp, "a=fmtp:{} {}\r\n", codec.id, params);
            }
        }
        let _ = write!(sdp, "a=ptime:{}\r\n", self.ptime);
        let _ = write!(sdp, "a=sendrecv\r\n");
        sdp
    }
}
