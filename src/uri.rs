// sentiric-sip-core/src/uri.rs

use crate::error::SipError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SipUri {
    pub scheme: String, // sip or sips
    pub user: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub params: Vec<(String, String)>,
}

impl FromStr for SipUri {
    type Err = SipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.trim().trim_start_matches('<').trim_end_matches('>');

        let (scheme, rest) = if clean.starts_with("sips:") {
            ("sips", &clean[5..])
        } else if clean.starts_with("sip:") {
            ("sip", &clean[4..])
        } else {
            return Err(SipError::ParseError("Invalid scheme".into()));
        };

        let (user_host, params_part) = match rest.find(';') {
            Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
            None => (rest, None),
        };

        let (user, host_port) = match user_host.find('@') {
            Some(idx) => (Some(user_host[..idx].to_string()), &user_host[idx + 1..]),
            None => (None, user_host),
        };

        let (host, port) = match host_port.find(':') {
            Some(idx) => {
                let h = host_port[..idx].to_string();
                let p = host_port[idx + 1..]
                    .parse::<u16>()
                    .map_err(|_| SipError::ParseError("Invalid port".into()))?;
                (h, Some(p))
            }
            None => (host_port.to_string(), None),
        };

        let mut params = Vec::new();
        if let Some(p_str) = params_part {
            for pair in p_str.split(';') {
                if let Some((k, v)) = pair.split_once('=') {
                    params.push((k.to_string(), v.to_string()));
                } else {
                    params.push((pair.to_string(), String::new()));
                }
            }
        }

        Ok(SipUri {
            scheme: scheme.to_string(),
            user,
            host,
            port,
            params,
        })
    }
}

impl SipUri {
    pub fn get_param(&self, name: &str) -> Option<&String> {
        self.params.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
}
