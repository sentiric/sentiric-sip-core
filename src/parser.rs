// src/parser.rs

use crate::packet::{SipPacket, Method};
use crate::header::{Header, HeaderName};
use std::str;

#[derive(Debug)]
pub enum ParseError {
    Utf8Error,
    EmptyPacket,
    InvalidStartLine,
}

pub fn parse(data: &[u8]) -> Result<SipPacket, ParseError> {
    let text = str::from_utf8(data).map_err(|_| ParseError::Utf8Error)?;
    
    // Header ve Body ayrımı (Çift CRLF)
    let mut parts = text.splitn(2, "\r\n\r\n");
    let head_part = parts.next().ok_or(ParseError::EmptyPacket)?;
    let body_part = parts.next(); // Opsiyonel

    let mut lines = head_part.lines();
    let start_line = lines.next().ok_or(ParseError::EmptyPacket)?;

    // Request mi Response mu?
    let mut packet = if start_line.starts_with("SIP/2.0") {
        // RESPONSE: "SIP/2.0 200 OK"
        let mut sl_parts = start_line.splitn(3, ' ');
        let _ver = sl_parts.next();
        let code = sl_parts.next().ok_or(ParseError::InvalidStartLine)?.parse::<u16>().unwrap_or(0);
        let reason = sl_parts.next().unwrap_or("").to_string();
        SipPacket::new_response(code, reason)
    } else {
        // REQUEST: "INVITE sip:x@y SIP/2.0"
        let mut sl_parts = start_line.splitn(3, ' ');
        let method_str = sl_parts.next().ok_or(ParseError::InvalidStartLine)?;
        let uri = sl_parts.next().ok_or(ParseError::InvalidStartLine)?.to_string();
        
        let method = match method_str {
            "INVITE" => Method::Invite,
            "ACK" => Method::Ack,
            "BYE" => Method::Bye,
            "CANCEL" => Method::Cancel,
            "OPTIONS" => Method::Options,
            "REGISTER" => Method::Register,
            _ => Method::Other(method_str.to_string()),
        };
        SipPacket::new_request(method, uri)
    };

    // Headerları Parse Et
    for line in lines {
        if line.trim().is_empty() { continue; }
        
        if let Some((key, value)) = line.split_once(':') {
            let header_name = Header::from_str(key);
            let header_val = value.trim().to_string();
            packet.headers.push(Header::new(header_name, header_val));
        }
    }

    // Body Varsa Ekle
    if let Some(body) = body_part {
        // null byte'ları temizle (bazen UDP paketinin sonunda gelebilir)
        let trimmed_body = body.trim_end_matches(char::from(0));
        packet.body = trimmed_body.as_bytes().to_vec();
    }

    Ok(packet)
}