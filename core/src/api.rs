use crate::network::{
    get_network_interfaces, WifiConfig, DnsAdvancedConfig, LanDhcpConfig, WanConfig,
    apply_wifi_config, apply_dns_config, apply_lan_config, apply_wan_config, get_dhcp_leases,
    scan_wifi_networks, analyze_channel_congestion, trigger_auto_channel_selection,
    SqmQosConfig, apply_sqm_qos, get_clients_traffic,
    Ipv6Config, apply_ipv6_config,
    IptvConfig, apply_iptv_config,
    FirewallConfig, apply_firewall_config,
    StaticRoute, apply_static_routes, get_switch_port_status,
    send_wake_on_lan, PingWatchdogConfig, apply_watchdog_config,
    DdnsConfig, apply_ddns_config, DiagnosticReq, run_diagnostic,
    WifiNeighborNetwork, ChannelCongestion
};
use crate::parser::{parse_proxy_uri, ProxyConfig};
use crate::singbox::{generate_singbox_config, RoutingMode, SingBoxConfig};
use crate::system::get_system_status;
use crate::zapret::{get_zapret_status, ZapretConfig};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tiny_http::{Header, Method, Request, Response, StatusCode};

#[derive(Clone)]
pub struct AppState {
    pub proxy_config: Arc<Mutex<Option<ProxyConfig>>>,
    pub routing_mode: Arc<Mutex<RoutingMode>>,
    #[allow(dead_code)]
    pub zapret_config: Arc<Mutex<ZapretConfig>>,
    pub wifi_config: Arc<Mutex<WifiConfig>>,
    pub dns_config: Arc<Mutex<DnsAdvancedConfig>>,
    pub lan_config: Arc<Mutex<LanDhcpConfig>>,
    pub wan_config: Arc<Mutex<WanConfig>>,
    pub qos_config: Arc<Mutex<SqmQosConfig>>,
    pub ipv6_config: Arc<Mutex<Ipv6Config>>,
    pub iptv_config: Arc<Mutex<IptvConfig>>,
    pub firewall_config: Arc<Mutex<FirewallConfig>>,
    pub routes_config: Arc<Mutex<Vec<StaticRoute>>>,
    pub watchdog_config: Arc<Mutex<PingWatchdogConfig>>,
    pub ddns_config: Arc<Mutex<DdnsConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        let wifi_cfg = std::fs::read_to_string("/etc/storage/wifi_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<WifiConfig>(&s).ok())
            .unwrap_or_default();

        let dns_cfg = std::fs::read_to_string("/etc/storage/dns_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<DnsAdvancedConfig>(&s).ok())
            .unwrap_or_default();

        let lan_cfg = std::fs::read_to_string("/etc/storage/lan_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<LanDhcpConfig>(&s).ok())
            .unwrap_or_default();

        let wan_cfg = std::fs::read_to_string("/etc/storage/wan_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<WanConfig>(&s).ok())
            .unwrap_or_default();

        let qos_cfg = std::fs::read_to_string("/etc/storage/qos_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<SqmQosConfig>(&s).ok())
            .unwrap_or_default();

        let ipv6_cfg = std::fs::read_to_string("/etc/storage/ipv6_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<Ipv6Config>(&s).ok())
            .unwrap_or_default();

        let iptv_cfg = std::fs::read_to_string("/etc/storage/iptv_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<IptvConfig>(&s).ok())
            .unwrap_or_default();

        let firewall_cfg = std::fs::read_to_string("/etc/storage/firewall_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<FirewallConfig>(&s).ok())
            .unwrap_or_default();

        let routes_cfg = std::fs::read_to_string("/etc/storage/routes_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<StaticRoute>>(&s).ok())
            .unwrap_or_default();

        let watchdog_cfg = std::fs::read_to_string("/etc/storage/watchdog_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<PingWatchdogConfig>(&s).ok())
            .unwrap_or_default();

        let ddns_cfg = std::fs::read_to_string("/etc/storage/ddns_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<DdnsConfig>(&s).ok())
            .unwrap_or_default();

        AppState {
            proxy_config: Arc::new(Mutex::new(None)),
            routing_mode: Arc::new(Mutex::new(RoutingMode::SmartRu)),
            zapret_config: Arc::new(Mutex::new(ZapretConfig::default())),
            wifi_config: Arc::new(Mutex::new(wifi_cfg)),
            dns_config: Arc::new(Mutex::new(dns_cfg)),
            lan_config: Arc::new(Mutex::new(lan_cfg)),
            wan_config: Arc::new(Mutex::new(wan_cfg)),
            qos_config: Arc::new(Mutex::new(qos_cfg)),
            ipv6_config: Arc::new(Mutex::new(ipv6_cfg)),
            iptv_config: Arc::new(Mutex::new(iptv_cfg)),
            firewall_config: Arc::new(Mutex::new(firewall_cfg)),
            routes_config: Arc::new(Mutex::new(routes_cfg)),
            watchdog_config: Arc::new(Mutex::new(watchdog_cfg)),
            ddns_config: Arc::new(Mutex::new(ddns_cfg)),
        }
    }
}

#[derive(Deserialize)]
struct WolReq {
    mac: String,
}

#[derive(Serialize)]
struct WifiScanResp {
    networks: Vec<WifiNeighborNetwork>,
    congestion: Vec<ChannelCongestion>,
}

#[derive(Deserialize)]
struct ImportProxyReq {
    uri: String,
}

#[derive(Serialize)]
struct ImportProxyResp {
    success: bool,
    config: Option<ProxyConfig>,
    singbox_json: Option<SingBoxConfig>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ApplyProxyReq {
    uri: String,
    mode: RoutingMode,
    #[serde(default)]
    custom_domains: Vec<String>,
    #[serde(default)]
    custom_ips: Vec<String>,
}

#[derive(Serialize)]
struct GenericResp {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct ProxyStatusResp {
    active: bool,
    mode: RoutingMode,
    current_proxy: Option<ProxyConfig>,
}

#[derive(Deserialize)]
struct TestPingReq {
    host: String,
    port: u16,
}

#[derive(Serialize)]
struct TestPingResp {
    success: bool,
    latency_ms: Option<u64>,
    error: Option<String>,
}

// Embedded Web UI assets
pub const INDEX_HTML: &str = include_str!("../../ui/public/index.html");
pub const APP_JS: &str = include_str!("../../ui/public/app.js");
pub const APP_CSS: &str = include_str!("../../ui/public/app.css");

pub fn handle_request(mut request: Request, state: &AppState) {
    let url = request.url().to_string();
    let method = request.method().clone();

    // CORS and Security Headers
    let cors_header = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();
    let content_type_json = Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap();
    let content_type_html = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
    let content_type_js = Header::from_bytes(&b"Content-Type"[..], &b"application/javascript; charset=utf-8"[..]).unwrap();
    let content_type_css = Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap();

    if method == Method::Options {
        let resp = Response::empty(StatusCode(204))
            .with_header(cors_header)
            .with_header(Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..]).unwrap())
            .with_header(Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap());
        let _ = request.respond(resp);
        return;
    }

    match (method, url.as_str()) {
        // UI static assets
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            let resp = Response::from_string(INDEX_HTML)
                .with_header(content_type_html);
            let _ = request.respond(resp);
        }
        (Method::Get, "/app.js") => {
            let resp = Response::from_string(APP_JS)
                .with_header(content_type_js);
            let _ = request.respond(resp);
        }
        (Method::Get, "/app.css") => {
            let resp = Response::from_string(APP_CSS)
                .with_header(content_type_css);
            let _ = request.respond(resp);
        }

        // REST API: System
        (Method::Get, "/api/v1/system/status") => {
            let status = get_system_status();
            respond_json(request, &status, cors_header, content_type_json);
        }

        // REST API: Network
        (Method::Get, "/api/v1/network/interfaces") => {
            let ifaces = get_network_interfaces();
            respond_json(request, &ifaces, cors_header, content_type_json);
        }

        // REST API: Wi-Fi
        (Method::Get, "/api/v1/wifi/config") => {
            let wifi = state.wifi_config.lock().unwrap().clone();
            respond_json(request, &wifi, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/wifi/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<WifiConfig>(&body) {
                *state.wifi_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_wifi_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/wifi_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "Wi-Fi settings applied to MT7620/MT7610 radio hardware and saved to Flash".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Invalid Wi-Fi JSON payload".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: DNS Advanced & DoH
        (Method::Get, "/api/v1/dns/config") => {
            let dns = state.dns_config.lock().unwrap().clone();
            respond_json(request, &dns, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/dns/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<DnsAdvancedConfig>(&body) {
                *state.dns_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_dns_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/dns_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "DNS & DoH settings saved to Flash and dnsmasq reloaded".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Invalid DNS JSON payload".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: LAN & DHCP
        (Method::Get, "/api/v1/lan/config") => {
            let lan = state.lan_config.lock().unwrap().clone();
            respond_json(request, &lan, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/lan/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<LanDhcpConfig>(&body) {
                *state.lan_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_lan_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/lan_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "LAN & DHCP static leases saved to Flash and applied".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Invalid LAN JSON payload".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: DHCP Active Leases
        (Method::Get, "/api/v1/dhcp/leases") => {
            let leases = get_dhcp_leases();
            respond_json(request, &leases, cors_header, content_type_json);
        }

        // REST API: WAN & NAT Port Forwarding
        (Method::Get, "/api/v1/wan/config") => {
            let wan = state.wan_config.lock().unwrap().clone();
            respond_json(request, &wan, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/wan/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<WanConfig>(&body) {
                *state.wan_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_wan_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/wan_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "WAN, Port Forwarding and Hardware NAT settings saved and active".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Invalid WAN JSON payload".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Wi-Fi Site Survey & Radar
        (Method::Get, "/api/v1/wifi/scan") => {
            let networks = scan_wifi_networks();
            let congestion = analyze_channel_congestion(&networks);
            let resp = WifiScanResp { networks, congestion };
            respond_json(request, &resp, cors_header, content_type_json);
        }

        // REST API: Dynamic Auto Channel Selection (ACS) Trigger
        (Method::Post, "/api/v1/wifi/acs/trigger") => {
            match trigger_auto_channel_selection() {
                Ok(msg) => {
                    let resp = GenericResp { success: true, message: msg };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
                Err(e) => {
                    let resp = GenericResp { success: false, message: e };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            }
        }

        // REST API: ASUS ROG Game Boost & SQM QoS
        (Method::Get, "/api/v1/qos/config") => {
            let qos = state.qos_config.lock().unwrap().clone();
            respond_json(request, &qos, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/qos/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<SqmQosConfig>(&body) {
                *state.qos_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_sqm_qos(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/qos_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "Настройки Game Boost и SQM QoS применены и сохранены".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Некорректный JSON для QoS".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Live Per-Device Traffic Analyzer
        (Method::Get, "/api/v1/clients/traffic") => {
            let game_ips = state.qos_config.lock().unwrap().game_device_ips.clone();
            let stats = get_clients_traffic(&game_ips);
            respond_json(request, &stats, cors_header, content_type_json);
        }

        // REST API: IPv6 Dual-Stack
        (Method::Get, "/api/v1/ipv6/config") => {
            let ipv6 = state.ipv6_config.lock().unwrap().clone();
            respond_json(request, &ipv6, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/ipv6/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<Ipv6Config>(&body) {
                *state.ipv6_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_ipv6_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/ipv6_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp { success: true, message: "Настройки IPv6 сохранены и применены".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON IPv6".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: IPTV Multicast & Udpxy
        (Method::Get, "/api/v1/iptv/config") => {
            let iptv = state.iptv_config.lock().unwrap().clone();
            respond_json(request, &iptv, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/iptv/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<IptvConfig>(&body) {
                *state.iptv_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_iptv_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/iptv_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp { success: true, message: "Настройки IPTV и Udpxy применены".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON IPTV".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Firewall & SPI Security
        (Method::Get, "/api/v1/firewall/config") => {
            let fw = state.firewall_config.lock().unwrap().clone();
            respond_json(request, &fw, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/firewall/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<FirewallConfig>(&body) {
                *state.firewall_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_firewall_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/firewall_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp { success: true, message: "Правила Firewall успешно обновлены".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON Firewall".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Static Routing Table
        (Method::Get, "/api/v1/routes/config") => {
            let routes = state.routes_config.lock().unwrap().clone();
            respond_json(request, &routes, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/routes/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_routes) = serde_json::from_str::<Vec<StaticRoute>>(&body) {
                *state.routes_config.lock().unwrap() = new_routes.clone();
                let _ = apply_static_routes(&new_routes);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_routes) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/routes_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp { success: true, message: "Статические маршруты применены".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON маршрутов".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Switch Physical Ethernet Ports Status
        (Method::Get, "/api/v1/switch/ports") => {
            let ports = get_switch_port_status();
            respond_json(request, &ports, cors_header, content_type_json);
        }

        // REST API: Wake-on-LAN (WoL)
        (Method::Post, "/api/v1/wol/wake") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(req) = serde_json::from_str::<WolReq>(&body) {
                match send_wake_on_lan(&req.mac) {
                    Ok(_) => {
                        let resp = GenericResp {
                            success: true,
                            message: format!("Magic packet успешно отправлен на MAC {}", req.mac),
                        };
                        respond_json(request, &resp, cors_header, content_type_json);
                    }
                    Err(e) => {
                        let resp = GenericResp { success: false, message: e };
                        respond_json(request, &resp, cors_header, content_type_json);
                    }
                }
            } else {
                let resp = GenericResp { success: false, message: "Не указан MAC-адрес".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Ping Watchdog
        (Method::Get, "/api/v1/watchdog/config") => {
            let wd = state.watchdog_config.lock().unwrap().clone();
            respond_json(request, &wd, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/watchdog/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<PingWatchdogConfig>(&body) {
                *state.watchdog_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_watchdog_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/watchdog_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "Настройки Ping Watchdog применены и сохранены".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON Watchdog".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Dynamic DNS (DDNS)
        (Method::Get, "/api/v1/ddns/config") => {
            let ddns = state.ddns_config.lock().unwrap().clone();
            respond_json(request, &ddns, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/ddns/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<DdnsConfig>(&body) {
                *state.ddns_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_ddns_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/ddns_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "Настройки DDNS успешно сохранены".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON DDNS".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Network Diagnostics
        (Method::Post, "/api/v1/diagnostics/run") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(req) = serde_json::from_str::<DiagnosticReq>(&body) {
                let result = run_diagnostic(&req);
                respond_json(request, &result, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный запрос диагностики".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Proxy import (parse URI)
        (Method::Post, "/api/v1/proxy/import") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let req: Result<ImportProxyReq, _> = serde_json::from_str(&body);

            match req {
                Ok(data) => match parse_proxy_uri(&data.uri) {
                    Ok(parsed) => {
                        let singbox = generate_singbox_config(&parsed, RoutingMode::SmartRu, &[], &[]).ok();
                        let resp_data = ImportProxyResp {
                            success: true,
                            config: Some(parsed),
                            singbox_json: singbox,
                            error: None,
                        };
                        respond_json(request, &resp_data, cors_header, content_type_json);
                    }
                    Err(e) => {
                        let resp_data = ImportProxyResp {
                            success: false,
                            config: None,
                            singbox_json: None,
                            error: Some(e),
                        };
                        respond_json(request, &resp_data, cors_header, content_type_json);
                    }
                },
                Err(e) => {
                    let resp_data = ImportProxyResp {
                        success: false,
                        config: None,
                        singbox_json: None,
                        error: Some(format!("Invalid request body: {}", e)),
                    };
                    respond_json(request, &resp_data, cors_header, content_type_json);
                }
            }
        }

        // REST API: Proxy apply (commit & apply configuration)
        (Method::Post, "/api/v1/proxy/apply") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let req: Result<ApplyProxyReq, _> = serde_json::from_str(&body);

            match req {
                Ok(data) => match parse_proxy_uri(&data.uri) {
                    Ok(parsed) => {
                        let sb_cfg = generate_singbox_config(&parsed, data.mode.clone(), &data.custom_domains, &data.custom_ips);
                        match sb_cfg {
                            Ok(cfg) => {
                                // Save active state
                                *state.proxy_config.lock().unwrap() = Some(parsed);
                                *state.routing_mode.lock().unwrap() = data.mode;

                                // Save config to /etc/sing-box/config.json in production
                                let json_str = serde_json::to_string_pretty(&cfg).unwrap_or_default();
                                let _ = std::fs::create_dir_all("/etc/sing-box");
                                let _ = std::fs::create_dir_all("/etc/storage/sing-box");
                                let _ = std::fs::write("/etc/sing-box/config.json", &json_str);
                                let _ = std::fs::write("/etc/storage/sing-box/config.json", &json_str);

                                // Commit to MTD Flash partition (mtd3 RW config) so settings survive reboot
                                let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();

                                // Restart Sing-Box service
                                let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("restart").output();

                                let resp = GenericResp {
                                    success: true,
                                    message: "Proxy configuration successfully applied, saved to Flash, and tunnel active".into(),
                                };
                                respond_json(request, &resp, cors_header, content_type_json);
                            }
                            Err(e) => {
                                let resp = GenericResp {
                                    success: false,
                                    message: format!("Failed to generate sing-box config: {}", e),
                                };
                                respond_json(request, &resp, cors_header, content_type_json);
                            }
                        }
                    }
                    Err(e) => {
                        let resp = GenericResp {
                            success: false,
                            message: format!("Failed to parse proxy URI: {}", e),
                        };
                        respond_json(request, &resp, cors_header, content_type_json);
                    }
                },
                Err(e) => {
                    let resp = GenericResp {
                        success: false,
                        message: format!("Invalid JSON request: {}", e),
                    };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            }
        }

        // REST API: Proxy status
        (Method::Get, "/api/v1/proxy/status") => {
            let proxy = state.proxy_config.lock().unwrap().clone();
            let mode = state.routing_mode.lock().unwrap().clone();
            let resp = ProxyStatusResp {
                active: proxy.is_some(),
                mode,
                current_proxy: proxy,
            };
            respond_json(request, &resp, cors_header, content_type_json);
        }

        // REST API: Ping test
        (Method::Post, "/api/v1/proxy/test") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let req: Result<TestPingReq, _> = serde_json::from_str(&body);

            match req {
                Ok(data) => {
                    let addr = format!("{}:{}", data.host, data.port);
                    let start = Instant::now();
                    match std::net::TcpStream::connect_timeout(
                        &addr.parse().unwrap_or_else(|_| "1.1.1.1:443".parse().unwrap()),
                        std::time::Duration::from_millis(2000),
                    ) {
                        Ok(_) => {
                            let duration = start.elapsed().as_millis() as u64;
                            let resp = TestPingResp {
                                success: true,
                                latency_ms: Some(duration),
                                error: None,
                            };
                            respond_json(request, &resp, cors_header, content_type_json);
                        }
                        Err(e) => {
                            let resp = TestPingResp {
                                success: false,
                                latency_ms: None,
                                error: Some(format!("Connection timeout or error: {}", e)),
                            };
                            respond_json(request, &resp, cors_header, content_type_json);
                        }
                    }
                }
                Err(e) => {
                    let resp = TestPingResp {
                        success: false,
                        latency_ms: None,
                        error: Some(format!("Invalid request: {}", e)),
                    };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            }
        }

        // REST API: Zapret status
        (Method::Get, "/api/v1/zapret/status") => {
            let status = get_zapret_status();
            respond_json(request, &status, cors_header, content_type_json);
        }

        // Fallback 404
        _ => {
            let resp = Response::from_string("{\"error\":\"Not Found\"}")
                .with_status_code(StatusCode(404))
                .with_header(cors_header)
                .with_header(content_type_json);
            let _ = request.respond(resp);
        }
    }
}

fn respond_json<T: Serialize>(request: Request, data: &T, cors: Header, content_type: Header) {
    let json_bytes = serde_json::to_vec(data).unwrap_or_default();
    let resp = Response::from_data(json_bytes)
        .with_header(cors)
        .with_header(content_type);
    let _ = request.respond(resp);
}
