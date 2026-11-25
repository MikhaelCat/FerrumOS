use crate::parser::{ProxyConfig, ProxyType};
use crate::network::is_valid_ipv4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    SmartRu, // Unblock Telegram, Discord, YouTube, blocked services; RU direct
    Global,  // All non-LAN traffic via proxy
    Custom,  // User-defined rules only
}

impl Default for RoutingMode {
    fn default() -> Self {
        RoutingMode::SmartRu
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingBoxConfig {
    pub log: LogConfig,
    pub dns: DnsConfig,
    pub inbounds: Vec<InboundConfig>,
    pub outbounds: Vec<OutboundConfig>,
    pub route: RouteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<DnsServer>,
    pub rules: Vec<DnsRule>,
    #[serde(rename = "final")]
    pub final_server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServer {
    pub tag: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detour: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRule {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub domain_suffix: Vec<String>,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    #[serde(rename = "type")]
    pub inbound_type: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub address: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inet4_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    pub auto_route: bool,
    pub strict_route: bool,
    #[serde(default)]
    pub auto_redirect: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    pub sniff: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub sniff_override_destination: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hysteria2Obfs {
    #[serde(rename = "type")]
    pub obfs_type: String,
    pub password: String,
}


impl Default for OutboundConfig {
    fn default() -> Self {
        OutboundConfig {
            outbound_type: "direct".into(),
            tag: "direct".into(),
            server: None,
            server_port: None,
            uuid: None,
            password: None,
            flow: None,
            tls: None,
            transport: None,
            network: None,
            packet_encoding: None,
            obfs: None,
            server_ports: None,
            up_mbps: None,
            down_mbps: None,
            congestion_control: None,
            udp_relay_mode: None,
            version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    #[serde(rename = "type")]
    pub outbound_type: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<TransportConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet_encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfs: Option<Hysteria2Obfs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_ports: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_mbps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down_mbps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub congestion_control: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_relay_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(default)]
    pub insecure: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utls: Option<UtlsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality: Option<RealityConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtlsConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityConfig {
    pub enabled: bool,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub rules: Vec<RouteRule>,
    pub auto_detect_interface: bool,
    #[serde(rename = "final")]
    pub final_outbound: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inbound: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub port: Vec<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protocol: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ip_is_private: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domain_suffix: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ip_cidr: Vec<String>,
    pub outbound: String,
}

pub const TELEGRAM_IPS: &[&str] = &[
    "91.108.0.0/16",
    "91.108.4.0/22",
    "91.108.8.0/22",
    "91.108.12.0/22",
    "91.108.16.0/22",
    "91.108.20.0/22",
    "91.108.56.0/22",
    "149.154.160.0/20",
    "149.154.164.0/22",
    "149.154.168.0/22",
    "149.154.172.0/22",
    "185.76.151.0/24",
];

pub const RU_BLOCKED_DOMAINS: &[&str] = &[
    // Telegram
    "t.me",
    "telegram.org",
    "telegram.me",
    "telesco.pe",
    "tdesktop.com",
    "telegra.ph",
    "stel.com",
    // Discord
    "discord.com",
    "discord.gg",
    "discordapp.com",
    "discordapp.net",
    "discord.media",
    "discord.co",
    "discordcdn.com",
    "discordstatus.com",
    // YouTube & Google CDN
    "youtube.com",
    "youtu.be",
    "googlevideo.com",
    "ytimg.com",
    "youtube-nocookie.com",
    "ggpht.com",
    "gvt1.com",
    "gvt2.com",
    "googleapis.com",
    "googleusercontent.com",
    "wide-youtube.l.google.com",
    "yt.be",
    // Meta / Instagram / Facebook
    "instagram.com",
    "cdninstagram.com",
    "facebook.com",
    "fbcdn.net",
    "fb.com",
    "messenger.com",
    // Twitter / X
    "twitter.com",
    "x.com",
    "twimg.com",
    "t.co",
    // AI Services
    "openai.com",
    "chatgpt.com",
    "anthropic.com",
    "claude.ai",
    // Encrypted Messengers & Email
    "signal.org",
    "proton.me",
    "protonmail.com",
    // Tech & Community
    "rutracker.org",
    "ntc.party",
    "flibusta.is",
    "notion.so",
    "medium.com",
];

pub fn generate_singbox_config(
    proxy: &ProxyConfig,
    mode: RoutingMode,
    custom_domains: &[String],
    custom_ips: &[String],
) -> Result<SingBoxConfig, String> {
    let proxy_outbound = build_proxy_outbound(proxy)?;

    let mut rules = Vec::new();

    // Rule 1: DNS Inbound from Dnsmasq on 127.0.0.1:5353 to dns-out
    rules.push(RouteRule {
        inbound: vec!["dns-in".into()],
        port: Vec::new(),
        protocol: Vec::new(),
        ip_is_private: false,
        domain_suffix: Vec::new(),
        ip_cidr: Vec::new(),
        outbound: "dns-out".into(),
    });

    // Rule 2: Sniffed DNS or destination port 53 to dns-out
    rules.push(RouteRule {
        inbound: Vec::new(),
        port: vec![53],
        protocol: vec!["dns".into()],
        ip_is_private: false,
        domain_suffix: Vec::new(),
        ip_cidr: Vec::new(),
        outbound: "dns-out".into(),
    });

    // Rule 3: Private LAN IPs stay direct
    rules.push(RouteRule {
        inbound: Vec::new(),
        port: Vec::new(),
        protocol: Vec::new(),
        ip_is_private: true,
        domain_suffix: Vec::new(),
        ip_cidr: Vec::new(),
        outbound: "direct".into(),
    });

    let (final_outbound, final_dns, dns_rules) = match mode {
        RoutingMode::Global => {
            (proxy.tag.clone(), "dns-remote".into(), Vec::new())
        }
        RoutingMode::Custom => {
            if !custom_domains.is_empty() {
                rules.push(RouteRule {
                    inbound: Vec::new(),
                    port: Vec::new(),
                    protocol: Vec::new(),
                    ip_is_private: false,
                    domain_suffix: custom_domains.to_vec(),
                    ip_cidr: Vec::new(),
                    outbound: proxy.tag.clone(),
                });
            }
            if !custom_ips.is_empty() {
                rules.push(RouteRule {
                    inbound: Vec::new(),
                    port: Vec::new(),
                    protocol: Vec::new(),
                    ip_is_private: false,
                    domain_suffix: Vec::new(),
                    ip_cidr: custom_ips.to_vec(),
                    outbound: proxy.tag.clone(),
                });
            }
            let dr = if !custom_domains.is_empty() {
                vec![DnsRule {
                    domain_suffix: custom_domains.to_vec(),
                    server: "dns-remote".into(),
                }]
            } else {
                Vec::new()
            };
            ("direct".into(), "dns-direct".into(), dr)
        }
        RoutingMode::SmartRu => {
            let mut all_domains: Vec<String> = RU_BLOCKED_DOMAINS.iter().map(|s| s.to_string()).collect();
            all_domains.extend_from_slice(custom_domains);

            rules.push(RouteRule {
                inbound: Vec::new(),
                port: Vec::new(),
                protocol: Vec::new(),
                ip_is_private: false,
                domain_suffix: all_domains.clone(),
                ip_cidr: Vec::new(),
                outbound: proxy.tag.clone(),
            });

            let mut all_ips: Vec<String> = TELEGRAM_IPS.iter().map(|s| s.to_string()).collect();
            all_ips.extend_from_slice(custom_ips);

            rules.push(RouteRule {
                inbound: Vec::new(),
                port: Vec::new(),
                protocol: Vec::new(),
                ip_is_private: false,
                domain_suffix: Vec::new(),
                ip_cidr: all_ips,
                outbound: proxy.tag.clone(),
            });

            let dr = vec![DnsRule {
                domain_suffix: all_domains,
                server: "dns-remote".into(),
            }];

            ("direct".into(), "dns-direct".into(), dr)
        }
    };

    let mut final_dns_rules = dns_rules;
    // Always resolve the proxy server domain directly to break circular bootstrap dependency in all modes
    if !is_valid_ipv4(&proxy.server) {
        final_dns_rules.insert(0, DnsRule {
            domain_suffix: vec![proxy.server.clone()],
            server: "dns-direct".into(),
        });
    }

    Ok(SingBoxConfig {
        log: LogConfig {
            level: "warn".into(),
            timestamp: false,
        },
        dns: DnsConfig {
            servers: vec![
                DnsServer {
                    tag: "dns-direct".into(),
                    address: "77.88.8.8".into(),
                    detour: Some("direct".into()),
                },
                DnsServer {
                    tag: "dns-remote".into(),
                    address: "8.8.8.8".into(),
                    detour: Some(proxy.tag.clone()),
                },
                DnsServer {
                    tag: "dns-remote-backup".into(),
                    address: "1.1.1.1".into(),
                    detour: Some(proxy.tag.clone()),
                },
            ],
            rules: final_dns_rules,
            final_server: final_dns,
        },
        inbounds: vec![
            InboundConfig {
                inbound_type: "direct".into(),
                tag: "dns-in".into(),
                listen: Some("127.0.0.1".into()),
                listen_port: Some(5353),
                network: Some("udp".into()),
                interface_name: None,
                address: Vec::new(),
                inet4_address: None,
                mtu: None,
                auto_route: false,
                strict_route: false,
                auto_redirect: false,
                stack: None,
                sniff: false,
                sniff_override_destination: false,
            },
            InboundConfig {
                inbound_type: "tun".into(),
                tag: "tun-in".into(),
                listen: None,
                listen_port: None,
                network: None,
                interface_name: Some("tun0".into()),
                address: vec!["172.19.0.1/30".into()],
                inet4_address: None,
                mtu: Some(1420),
                auto_route: false,
                strict_route: false,
                auto_redirect: false,
                stack: Some("gvisor".into()),
                sniff: true,
                sniff_override_destination: true,
            },
        ],
        outbounds: vec![
            proxy_outbound,
            OutboundConfig {
                outbound_type: "direct".into(),
                tag: "direct".into(),
                server: None,
                server_port: None,
                uuid: None,
                password: None,
                flow: None,
                tls: None,
                transport: None,
                network: None,
                packet_encoding: None,
                obfs: None,
                server_ports: None,
                up_mbps: None,
                down_mbps: None,
                congestion_control: None,
                udp_relay_mode: None,
                version: None,
            },
            OutboundConfig {
                outbound_type: "block".into(),
                tag: "block".into(),
                server: None,
                server_port: None,
                uuid: None,
                password: None,
                flow: None,
                tls: None,
                transport: None,
                network: None,
                packet_encoding: None,
                obfs: None,
                server_ports: None,
                up_mbps: None,
                down_mbps: None,
                congestion_control: None,
                udp_relay_mode: None,
                version: None,
            },
            OutboundConfig {
                outbound_type: "dns".into(),
                tag: "dns-out".into(),
                server: None,
                server_port: None,
                uuid: None,
                password: None,
                flow: None,
                tls: None,
                transport: None,
                network: None,
                packet_encoding: None,
                obfs: None,
                server_ports: None,
                up_mbps: None,
                down_mbps: None,
                congestion_control: None,
                udp_relay_mode: None,
                version: None,
            },
        ],
        route: RouteConfig {
            rules,
            auto_detect_interface: true,
            final_outbound,
        },
    })
}

fn build_proxy_outbound(proxy: &ProxyConfig) -> Result<OutboundConfig, String> {
    match proxy.proxy_type {
        ProxyType::Vless => {
            let alpn = proxy.extra_params.get("alpn").map(|s| s.split(',').map(|p| p.trim().to_string()).collect::<Vec<_>>());
            let tls = if proxy.security.as_deref() == Some("reality") {
                let pbk = proxy.public_key.clone().ok_or("Reality missing public key (pbk)")?;
                Some(TlsConfig {
                    enabled: true,
                    server_name: proxy.sni.clone(),
                    insecure: proxy.insecure,
                    alpn: alpn.clone(),
                    utls: Some(UtlsConfig {
                        enabled: true,
                        fingerprint: proxy.fingerprint.clone(),
                    }),
                    reality: Some(RealityConfig {
                        enabled: true,
                        public_key: pbk,
                        short_id: proxy.short_id.clone(),
                    }),
                })
            } else if proxy.security.as_deref() == Some("tls") {
                Some(TlsConfig {
                    enabled: true,
                    server_name: proxy.sni.clone(),
                    insecure: proxy.insecure,
                    alpn: alpn.clone(),
                    utls: Some(UtlsConfig {
                        enabled: true,
                        fingerprint: proxy.fingerprint.clone(),
                    }),
                    reality: None,
                })
            } else {
                None
            };

            let transport = if proxy.network.as_deref() == Some("ws") {
                Some(TransportConfig {
                    transport_type: "ws".into(),
                    path: proxy.path.clone(),
                    service_name: None,
                })
            } else if proxy.network.as_deref() == Some("grpc") {
                Some(TransportConfig {
                    transport_type: "grpc".into(),
                    path: None,
                    service_name: proxy.service_name.clone(),
                })
            } else {
                None
            };

            let packet_encoding = proxy.extra_params.get("packetEncoding").cloned().or_else(|| Some("xudp".into()));

            Ok(OutboundConfig {
                outbound_type: "vless".into(),
                tag: proxy.tag.clone(),
                server: Some(proxy.server.clone()),
                server_port: Some(proxy.server_port),
                uuid: proxy.uuid.clone(),
                password: None,
                flow: proxy.flow.clone(),
                tls,
                transport,
                network: None,
                packet_encoding,
                obfs: None,
                server_ports: None,
                up_mbps: None,
                down_mbps: None,
                congestion_control: None,
                udp_relay_mode: None,
                version: None,
            })
        }
        ProxyType::Hysteria2 => {
            let alpn = proxy.extra_params.get("alpn").map(|s| s.split(',').map(|p| p.trim().to_string()).collect::<Vec<_>>());
            let obfs = if let Some(obfs_type) = proxy.extra_params.get("obfs") {
                let obfs_pw = proxy.extra_params.get("obfs-password").cloned().unwrap_or_default();
                Some(Hysteria2Obfs {
                    obfs_type: obfs_type.clone(),
                    password: obfs_pw,
                })
            } else {
                None
            };
            let server_ports = proxy.extra_params.get("mport").cloned()
                .or_else(|| proxy.extra_params.get("ports").cloned());

            Ok(OutboundConfig {
                outbound_type: "hysteria2".into(),
                tag: proxy.tag.clone(),
                server: Some(proxy.server.clone()),
                server_port: Some(proxy.server_port),
                uuid: None,
                password: proxy.password.clone(),
                flow: None,
                tls: Some(TlsConfig {
                    enabled: true,
                    server_name: proxy.sni.clone(),
                    insecure: proxy.insecure,
                    alpn,
                    utls: None,
                    reality: None,
                }),
                transport: None,
                network: None,
                packet_encoding: None,
                obfs,
                server_ports,
                up_mbps: proxy.up_mbps,
                down_mbps: proxy.down_mbps,
                congestion_control: None,
                udp_relay_mode: None,
                version: None,
            })
        }
        ProxyType::Trojan => {
            Ok(OutboundConfig {
                outbound_type: "trojan".into(),
                tag: proxy.tag.clone(),
                server: Some(proxy.server.clone()),
                server_port: Some(proxy.server_port),
                uuid: None,
                password: proxy.password.clone(),
                flow: None,
                tls: Some(TlsConfig {
                    enabled: true,
                    server_name: proxy.sni.clone(),
                    insecure: proxy.insecure,
                    alpn: None,
                    utls: None,
                    reality: None,
                }),
                transport: None,
                network: None,
                packet_encoding: None,
                obfs: None,
                server_ports: None,
                up_mbps: None,
                down_mbps: None,
                congestion_control: None,
                udp_relay_mode: None,
                version: None,
            })
        }
        ProxyType::Shadowsocks => {
            Err("Shadowsocks direct outbound not configured for sing-box".into())
        }
        ProxyType::Tuic => {
            let alpn = proxy.extra_params.get("alpn").map(|s| s.split(',').map(|p| p.trim().to_string()).collect::<Vec<_>>())
                .or_else(|| Some(vec!["h3".to_string()]));
            let cc = proxy.extra_params.get("congestion_control").cloned().or_else(|| Some("bbr".into()));
            let udp_relay = proxy.extra_params.get("udp_relay_mode").cloned().or_else(|| Some("native".into()));

            Ok(OutboundConfig {
                outbound_type: "tuic".into(),
                tag: proxy.tag.clone(),
                server: Some(proxy.server.clone()),
                server_port: Some(proxy.server_port),
                uuid: proxy.uuid.clone(),
                password: proxy.password.clone(),
                flow: None,
                tls: Some(TlsConfig {
                    enabled: true,
                    server_name: proxy.sni.clone().or_else(|| Some(proxy.server.clone())),
                    insecure: proxy.insecure,
                    alpn,
                    utls: None,
                    reality: None,
                }),
                transport: None,
                network: None,
                packet_encoding: None,
                obfs: None,
                server_ports: None,
                up_mbps: proxy.up_mbps,
                down_mbps: proxy.down_mbps,
                congestion_control: cc,
                udp_relay_mode: udp_relay,
                version: None,
            })
        }
        ProxyType::Shadowtls => {
            let version = proxy.extra_params.get("version").and_then(|v| v.parse::<u32>().ok()).unwrap_or(3);
            let sni = proxy.sni.clone().or_else(|| Some("apple.com".to_string()));

            Ok(OutboundConfig {
                outbound_type: "shadowtls".into(),
                tag: proxy.tag.clone(),
                server: Some(proxy.server.clone()),
                server_port: Some(proxy.server_port),
                uuid: None,
                password: proxy.password.clone(),
                flow: None,
                tls: Some(TlsConfig {
                    enabled: true,
                    server_name: sni,
                    insecure: false,
                    alpn: None,
                    utls: None,
                    reality: None,
                }),
                transport: None,
                network: None,
                packet_encoding: None,
                obfs: None,
                server_ports: None,
                up_mbps: None,
                down_mbps: None,
                congestion_control: None,
                udp_relay_mode: None,
                version: Some(version),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_proxy_uri;

    
    #[test]
    fn test_generate_tuic_outbound() {
        let uri = "tuic://b831381d-6324-4d53-ad4f-8cda48b30811:MyPass@198.51.100.99:8443?congestion_control=bbr&sni=fast.cdn.net#Tuic-Node";
        let proxy = crate::parser::parse_proxy_uri(uri).expect("Parse TUIC");
        let cfg = generate_singbox_config(&proxy, RoutingMode::Global, &[], &[]).expect("Generate singbox config");
        let outbound = cfg.outbounds.iter().find(|o| o.tag == "Tuic-Node").expect("Tuic outbound");
        assert_eq!(outbound.outbound_type, "tuic");
        assert_eq!(outbound.congestion_control.as_deref(), Some("bbr"));
        assert_eq!(outbound.server.as_deref(), Some("198.51.100.99"));
    }

    #[test]
    fn test_generate_shadowtls_outbound() {
        let uri = "shadowtls://ShadowSecret@198.51.100.50:443?sni=gateway.icloud.com&version=3#Shadow-Node";
        let proxy = crate::parser::parse_proxy_uri(uri).expect("Parse ShadowTLS");
        let cfg = generate_singbox_config(&proxy, RoutingMode::SmartRu, &[], &[]).expect("Generate singbox config");
        let outbound = cfg.outbounds.iter().find(|o| o.tag == "Shadow-Node").expect("ShadowTLS outbound");
        assert_eq!(outbound.outbound_type, "shadowtls");
        assert_eq!(outbound.version, Some(3));
    }

    #[test]
    fn test_generate_vless_smart_ru() {
        let uri = "vless://b831381d-6324-4d53-ad4f-8cda48b30811@198.51.100.1:443?security=reality&sni=dl.google.com&fp=chrome&pbk=Z3b4-RealityPublicKey-12345&sid=1a2b3c4d&type=tcp&flow=xtls-rprx-vision#Ferrum-VLESS";
        let parsed = parse_proxy_uri(uri).expect("Parse error");
        let cfg = generate_singbox_config(&parsed, RoutingMode::SmartRu, &[], &[]).expect("Gen error");

        assert_eq!(cfg.outbounds[0].outbound_type, "vless");
        assert_eq!(cfg.outbounds[0].tag, "Ferrum-VLESS");
        assert_eq!(cfg.route.final_outbound, "direct");

        // Validate that Telegram and Discord domain suffixes are routed to proxy
        let rule = cfg.route.rules.iter().find(|r| r.domain_suffix.contains(&"t.me".to_string())).expect("Domain rule missing");
        assert_eq!(rule.outbound, "Ferrum-VLESS");

        let serialized = serde_json::to_string_pretty(&cfg).expect("JSON serialize error");
        assert!(serialized.contains("dl.google.com"));
        assert!(serialized.contains("xtls-rprx-vision"));
    }

    #[test]
    fn test_generate_hy2_global() {
        let uri = "hysteria2://MyPassword@198.51.100.2:8443?sni=fast.cdn.net#Ferrum-Hy2";
        let parsed = parse_proxy_uri(uri).expect("Parse error");
        let cfg = generate_singbox_config(&parsed, RoutingMode::Global, &[], &[]).expect("Gen error");

        assert_eq!(cfg.outbounds[0].outbound_type, "hysteria2");
        assert_eq!(cfg.outbounds[0].tag, "Ferrum-Hy2");
        assert_eq!(cfg.route.final_outbound, "Ferrum-Hy2");
    }
}
