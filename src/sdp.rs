// sentiric-sip-core/src/sdp.rs (Tam Dosya Revizyonu)
use std::fmt::Write;

pub struct SdpManipulator;

impl SdpManipulator {
    /// Gelen SDP gövdesini satır satır gezer, IP (c=) ve Port (m=audio) bilgilerini
    /// beyaz boşluk (whitespace) hassasiyeti olmadan değiştirir.
    pub fn rewrite_connection_info(sdp_body: &[u8], new_ip: &str, new_port: u16) -> Option<Vec<u8>> {
        let sdp_str = std::str::from_utf8(sdp_body).ok()?;
        let mut new_sdp = String::with_capacity(sdp_str.len());
        let mut modified = false;

        for line in sdp_str.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("c=IN IP4") {
                // IP değiştirme
                let _ = writeln!(new_sdp, "c=IN IP4 {}", new_ip);
                modified = true;
            } else if trimmed.starts_with("m=audio") {
                // Port değiştirme: m=audio <port> <proto> <payloads>
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 4 {
                    // parts[0] = "m=audio", parts[1] = "<eski_port>", parts[2] = "<proto>", rest = payloads
                    let proto = parts[2];
                    let payloads = parts[3..].join(" ");
                    let _ = writeln!(new_sdp, "m=audio {} {} {}", new_port, proto, payloads);
                    modified = true;
                } else {
                    new_sdp.push_str(line);
                    new_sdp.push_str("\r\n");
                }
            } else {
                new_sdp.push_str(line);
                new_sdp.push_str("\r\n");
            }
        }

        if modified { Some(new_sdp.into_bytes()) } else { None }
    }
}