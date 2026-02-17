// sentiric-sip-core/src/sdp.rs
use std::fmt::Write;

pub struct SdpManipulator;

impl SdpManipulator {
    pub fn rewrite_connection_info(sdp_body: &[u8], new_ip: &str, new_port: u16) -> Option<Vec<u8>> {
        let sdp_str = std::str::from_utf8(sdp_body).ok()?;
        let mut new_sdp = String::with_capacity(sdp_str.len());
        let mut modified = false;

        for line in sdp_str.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }

            if trimmed.starts_with("c=IN IP4") {
                let _ = writeln!(new_sdp, "c=IN IP4 {}", new_ip);
                modified = true;
            } else if trimmed.starts_with("m=audio") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 4 {
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