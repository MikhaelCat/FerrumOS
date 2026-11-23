use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Vless,
    Hysteria2,
    Trojan,
    Shadowsocks,
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyType::Vless => write!(f, "vless"),
            ProxyType::Hysteria2 => write!(f, "hysteria2"),
            ProxyType::Trojan => write!(f, "trojan"),
            ProxyType::Shadowsocks => write!(f, "shadowsocks"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub tag: String,
    pub server: String,
    pub server_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>, // "reality", "tls", "none"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,     // "xtls-rprx-vision"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sni: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>, // Reality pbk
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,   // Reality sid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,    // "tcp", "ws", "grpc"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(default)]
    pub insecure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_mbps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down_mbps: Option<u32>,
    pub raw_uri: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_params: HashMap<String, String>,
}

pub fn parse_proxy_uri(raw: &str) -> Result<ProxyConfig, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Empty URI".into());
    }

    let lower = trimmed.to_lowercase();
    if lower.starts_with("vless://") {
        parse_vless(trimmed)
    } else if lower.starts_with("hysteria2://") || lower.starts_with("hy2://") {
        parse_hysteria2(trimmed)
    } else if lower.starts_with("trojan://") {
        parse_trojan(trimmed)
    } else if lower.starts_with("ss://") {
        parse_shadowsocks(trimmed)
    } else {
        Err(format!("Unsupported proxy protocol: {}", trimmed))
    }
}

fn parse_vless(raw: &str) -> Result<ProxyConfig, String> {
    let parsed = Url::parse(raw).map_err(|e| format!("Invalid VLESS URL: {}", e))?;

    let uuid = parsed.username().to_string();
    if uuid.is_empty() {
        return Err("VLESS URI missing UUID".into());
    }

    let server = parsed
        .host_str()
        .ok_or_else(|| "VLESS URI missing server host".to_string())?
        .to_string();

    let server_port = parsed.port().unwrap_or(443);

    let mut query_map = HashMap::new();
    for (k, v) in parsed.query_pairs() {
        query_map.insert(k.to_string(), v.to_string());
    }

    let tag = if let Some(fragment) = parsed.fragment() {
        urlencoding_decode(fragment)
    } else {
        format!("vless-{}-{}", server, server_port)
    };

    let security = query_map.get("security").cloned().unwrap_or_else(|| "none".into());
    let flow = query_map.get("flow").cloned();
    let sni = query_map.get("sni").cloned();
    let fp = query_map
        .get("fp")
        .cloned()
        .or_else(|| if security == "reality" || security == "tls" { Some("chrome".into()) } else { None });
    let pbk = query_map.get("pbk").cloned();
    let sid = query_map.get("sid").cloned();
    let network = query_map.get("type").cloned().unwrap_or_else(|| "tcp".into());
    let path = query_map.get("path").cloned();
    let service_name = query_map.get("serviceName").cloned();
    let insecure = query_map.get("insecure").map(|v| v == "1").unwrap_or(false)
        || query_map.get("allowInsecure").map(|v| v == "1").unwrap_or(false);

    Ok(ProxyConfig {
        proxy_type: ProxyType::Vless,
        tag,
        server,
        server_port,
        uuid: Some(uuid),
        password: None,
        security: Some(security),
        flow,
        sni,
        fingerprint: fp,
        public_key: pbk,
        short_id: sid,
        network: Some(network),
        path,
        service_name,
        insecure,
        up_mbps: None,
        down_mbps: None,
        raw_uri: raw.to_string(),
        extra_params: query_map,
    })
}

fn parse_hysteria2(raw: &str) -> Result<ProxyConfig, String> {
    // Normalizing hy2:// to hysteria2://
    let normalized = if raw.to_lowercase().starts_with("hy2://") {
        format!("hysteria2://{}", &raw[6..])
    } else {
        raw.to_string()
    };

    let parsed = Url::parse(&normalized).map_err(|e| format!("Invalid Hysteria2 URL: {}", e))?;

    let password = if !parsed.username().is_empty() {
        parsed.username().to_string()
    } else if let Some(p) = parsed.password() {
        p.to_string()
    } else {
        String::new()
    };

    let server = parsed
        .host_str()
        .ok_or_else(|| "Hysteria2 URI missing server host".to_string())?
        .to_string();

    let server_port = parsed.port().unwrap_or(443);

    let mut query_map = HashMap::new();
    for (k, v) in parsed.query_pairs() {
        query_map.insert(k.to_string(), v.to_string());
    }

    let tag = if let Some(fragment) = parsed.fragment() {
        urlencoding_decode(fragment)
    } else {
        format!("hy2-{}-{}", server, server_port)
    };

    let sni = query_map.get("sni").cloned().or_else(|| Some(server.clone()));
    let insecure = query_map.get("insecure").map(|v| v == "1").unwrap_or(false)
        || query_map.get("allowInsecure").map(|v| v == "1").unwrap_or(false);

    let up_mbps = query_map.get("up").and_then(|u| {
        u.trim_end_matches("mbps").trim_end_matches("Mbps").parse::<u32>().ok()
    });
    let down_mbps = query_map.get("down").and_then(|d| {
        d.trim_end_matches("mbps").trim_end_matches("Mbps").parse::<u32>().ok()
    });

    Ok(ProxyConfig {
        proxy_type: ProxyType::Hysteria2,
        tag,
        server,
        server_port,
        uuid: None,
        password: Some(password),
        security: Some("tls".into()),
        flow: None,
        sni,
        fingerprint: None,
        public_key: None,
        short_id: None,
        network: Some("udp".into()),
        path: None,
        service_name: None,
        insecure,
        up_mbps,
        down_mbps,
        raw_uri: raw.to_string(),
        extra_params: query_map,
    })
}

fn parse_trojan(raw: &str) -> Result<ProxyConfig, String> {
    let parsed = Url::parse(raw).map_err(|e| format!("Invalid Trojan URL: {}", e))?;
    let pass = parsed.username().to_string();
    let server = parsed.host_str().ok_or_else(|| "Missing host".to_string())?.to_string();
    let server_port = parsed.port().unwrap_or(443);
    let tag = parsed.fragment().map(urlencoding_decode).unwrap_or_else(|| format!("trojan-{}-{}", server, server_port));

    Ok(ProxyConfig {
        proxy_type: ProxyType::Trojan,
        tag,
        server: server.clone(),
        server_port,
        uuid: None,
        password: Some(pass),
        security: Some("tls".into()),
        flow: None,
        sni: Some(server),
        fingerprint: None,
        public_key: None,
        short_id: None,
        network: Some("tcp".into()),
        path: None,
        service_name: None,
        insecure: false,
        up_mbps: None,
        down_mbps: None,
        raw_uri: raw.to_string(),
        extra_params: HashMap::new(),
    })
}

fn parse_shadowsocks(raw: &str) -> Result<ProxyConfig, String> {
    let parsed = Url::parse(raw).map_err(|e| format!("Invalid SS URL: {}", e))?;
    let userinfo = parsed.username();
    let tag = parsed.fragment().map(urlencoding_decode).unwrap_or_else(|| "shadowsocks".into());

    let decoded = STANDARD.decode(userinfo).map_err(|e| format!("SS base64 decode failed: {}", e))?;
    let decoded_str = String::from_utf8(decoded).map_err(|e| format!("SS utf8 decode failed: {}", e))?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Invalid SS userinfo payload".into());
    }

    let method = parts[0].to_string();
    let pass = parts[1].to_string();
    let server = parsed.host_str().ok_or_else(|| "Missing SS host".to_string())?.to_string();
    let server_port = parsed.port().unwrap_or(8388);

    let mut extra = HashMap::new();
    extra.insert("method".into(), method);

    Ok(ProxyConfig {
        proxy_type: ProxyType::Shadowsocks,
        tag,
        server,
        server_port,
        uuid: None,
        password: Some(pass),
        security: None,
        flow: None,
        sni: None,
        fingerprint: None,
        public_key: None,
        short_id: None,
        network: Some("tcp".into()),
        path: None,
        service_name: None,
        insecure: false,
        up_mbps: None,
        down_mbps: None,
        raw_uri: raw.to_string(),
        extra_params: extra,
    })
}

fn urlencoding_decode(val: &str) -> String {
    percent_encoding::percent_decode_str(val)
        .decode_utf8_lossy()
}

mod percent_encoding {
    pub struct PercentDecode<'a> {
        bytes: &'a [u8],
    }

    pub fn percent_decode_str(input: &str) -> PercentDecode<'_> {
        PercentDecode {
            bytes: input.as_bytes(),
        }
    }

    impl<'a> PercentDecode<'a> {
        pub fn decode_utf8_lossy(&self) -> String {
            let mut out = Vec::with_capacity(self.bytes.len());
            let mut i = 0;
            while i < self.bytes.len() {
                if self.bytes[i] == b'%' && i + 2 < self.bytes.len() {
                    if let Ok(val) = u8::from_str_radix(
                        std::str::from_utf8(&self.bytes[i + 1..i + 3]).unwrap_or(""),
                        16,
                    ) {
                        out.push(val);
                        i += 3;
                        continue;
                    }
                }
                out.push(self.bytes[i]);
                i += 1;
            }
            String::from_utf8_lossy(&out).into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vless_reality() {
        let uri = "vless://b831381d-6324-4d53-ad4f-8cda48b30811@198.51.100.1:443?security=reality&sni=dl.google.com&fp=chrome&pbk=Z3b4-RealityPublicKey-12345&sid=1a2b3c4d&type=tcp&flow=xtls-rprx-vision#Ferrum-VLESS-Reality";
        let cfg = parse_proxy_uri(uri).expect("Should parse VLESS");
        assert_eq!(cfg.proxy_type, ProxyType::Vless);
        assert_eq!(cfg.server, "198.51.100.1");
        assert_eq!(cfg.server_port, 443);
        assert_eq!(cfg.uuid.as_deref(), Some("b831381d-6324-4d53-ad4f-8cda48b30811"));
        assert_eq!(cfg.security.as_deref(), Some("reality"));
        assert_eq!(cfg.flow.as_deref(), Some("xtls-rprx-vision"));
        assert_eq!(cfg.sni.as_deref(), Some("dl.google.com"));
        assert_eq!(cfg.public_key.as_deref(), Some("Z3b4-RealityPublicKey-12345"));
        assert_eq!(cfg.short_id.as_deref(), Some("1a2b3c4d"));
        assert_eq!(cfg.tag, "Ferrum-VLESS-Reality");
    }

    #[test]
    fn test_hysteria2() {
        let uri = "hysteria2://Secr3tPass@203.0.113.88:8443?sni=fast.cdn.net&insecure=0&up=100mbps&down=200mbps#Ferrum-Hy2";
        let cfg = parse_proxy_uri(uri).expect("Should parse Hysteria2");
        assert_eq!(cfg.proxy_type, ProxyType::Hysteria2);
        assert_eq!(cfg.server, "203.0.113.88");
        assert_eq!(cfg.server_port, 8443);
        assert_eq!(cfg.password.as_deref(), Some("Secr3tPass"));
        assert_eq!(cfg.sni.as_deref(), Some("fast.cdn.net"));
        assert_eq!(cfg.up_mbps, Some(100));
        assert_eq!(cfg.down_mbps, Some(200));
        assert_eq!(cfg.tag, "Ferrum-Hy2");
    }

    #[test]
    fn test_hy2_short() {
        let uri = "hy2://pwd123@1.1.1.1:443#ShortHy2";
        let cfg = parse_proxy_uri(uri).expect("Should parse hy2 short");
        assert_eq!(cfg.proxy_type, ProxyType::Hysteria2);
        assert_eq!(cfg.server, "1.1.1.1");
        assert_eq!(cfg.password.as_deref(), Some("pwd123"));
        assert_eq!(cfg.tag, "ShortHy2");
    }

    #[test]
    fn test_empty_and_invalid_uris() {
        assert!(parse_proxy_uri("").is_err());
        assert!(parse_proxy_uri("   ").is_err());
        assert!(parse_proxy_uri("http://google.com").is_err());
        assert!(parse_proxy_uri("vless://@1.1.1.1:443").is_err());
    }

    #[test]
    fn test_url_encoded_tag_decoding() {
        let uri = "vless://b831381d-6324-4d53-ad4f-8cda48b30811@1.1.1.1:443#%D0%A0%D0%BE%D1%83%D1%82%D0%B5%D1%80";
        let cfg = parse_proxy_uri(uri).expect("Should parse with cyrillic fragment");
        assert_eq!(cfg.tag, "Роутер");
    }
}
