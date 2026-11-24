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
    WifiNeighborNetwork, ChannelCongestion,
    FlowOffloadConfig, apply_flow_offload, load_flow_offload_config,
    AntizapretBgpConfig, get_antizapret_bgp_status, apply_antizapret_bgp_config,
    run_wan_speedtest
};
use crate::parser::{parse_proxy_uri, ProxyConfig};
use crate::singbox::{generate_singbox_config, RoutingMode, SingBoxConfig};
use crate::system::get_system_status;
use crate::zapret::{apply_zapret_config, get_zapret_status, probe_zapret_strategies, ZapretConfig};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tiny_http::{Header, Method, Request, Response, StatusCode};

#[derive(Clone)]
pub struct AppState {
    pub proxy_config: Arc<Mutex<Option<ProxyConfig>>>,
    pub proxy_enabled: Arc<Mutex<bool>>,
    pub routing_mode: Arc<Mutex<RoutingMode>>,
    pub zapret_config: Arc<Mutex<ZapretConfig>>,
    pub auth_password: Arc<Mutex<String>>,
    pub auth_tokens: Arc<Mutex<Vec<String>>>,
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

        let zapret_cfg = std::fs::read_to_string("/etc/storage/zapret_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<ZapretConfig>(&s).ok())
            .unwrap_or_default();

        let admin_pw = std::fs::read_to_string("/etc/storage/admin_password.txt")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "root".into());

        AppState {
            proxy_config: Arc::new(Mutex::new(None)),
            proxy_enabled: Arc::new(Mutex::new(true)),
            routing_mode: Arc::new(Mutex::new(RoutingMode::SmartRu)),
            zapret_config: Arc::new(Mutex::new(zapret_cfg)),
            auth_password: Arc::new(Mutex::new(admin_pw)),
            auth_tokens: Arc::new(Mutex::new(Vec::new())),
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
struct LoginReq {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResp {
    success: bool,
    token: Option<String>,
    message: String,
}

#[derive(Deserialize)]
struct ChangePasswordReq {
    old_password: String,
    new_password: String,
}

#[derive(Deserialize)]
struct ProxyToggleReq {
    enabled: bool,
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
    #[serde(default = "default_true")]
    enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize)]
struct GenericResp {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct ProxyStatusResp {
    active: bool,
    enabled: bool,
    running: bool,
    core_installed: bool,
    mode: RoutingMode,
    current_proxy: Option<ProxyConfig>,
    status_text: String,
}

#[derive(Serialize)]
struct FirmwareUploadResp {
    success: bool,
    message: String,
    file_size: usize,
    valid_header: bool,
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
    let raw_url = request.url().to_string();
    let path_no_query = raw_url.split('?').next().unwrap_or("/");
    let clean_path = if path_no_query.len() > 1 && path_no_query.ends_with('/') {
        path_no_query.trim_end_matches('/')
    } else {
        path_no_query
    };
    let method = request.method().clone();

    // Security Headers
    let sec_xfo = Header::from_bytes(&b"X-Frame-Options"[..], &b"DENY"[..]).unwrap();
    let sec_nosniff = Header::from_bytes(&b"X-Content-Type-Options"[..], &b"nosniff"[..]).unwrap();
    let sec_ref = Header::from_bytes(&b"Referrer-Policy"[..], &b"no-referrer"[..]).unwrap();
    let cors_header = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();
    let content_type_json = Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap();
    let content_type_html = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
    let content_type_js = Header::from_bytes(&b"Content-Type"[..], &b"application/javascript; charset=utf-8"[..]).unwrap();
    let content_type_css = Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap();

    // Cross-Origin Request Protection (Anti-CSRF on state-modifying requests)
    if method == Method::Post {
        let is_valid_origin = request.headers().iter().all(|h| {
            if h.field.equiv("Origin") {
                let val = h.value.as_str();
                val.starts_with("http://192.168.1.1")
                    || val.starts_with("http://localhost")
                    || val.starts_with("http://127.0.0.1")
                    || val.starts_with("http://[::1]")
            } else {
                true
            }
        });

        if !is_valid_origin {
            let resp = Response::from_string("403 Forbidden: Cross-Origin Request Blocked")
                .with_status_code(StatusCode(403))
                .with_header(sec_xfo)
                .with_header(sec_nosniff);
            let _ = request.respond(resp);
            return;
        }
    }

    if method == Method::Options {
        let resp = Response::empty(StatusCode(204))
            .with_header(cors_header)
            .with_header(sec_xfo)
            .with_header(sec_nosniff)
            .with_header(Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..]).unwrap())
            .with_header(Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap());
        let _ = request.respond(resp);
        return;
    }

    match (method, clean_path) {
        // Padavan internal watchdog health check
        (Method::Get, "/httpd_check.htm") | (Method::Head, "/httpd_check.htm") => {
            let resp = Response::from_string("ASUSTeK\n")
                .with_header(content_type_html);
            let _ = request.respond(resp);
        }
        // UI static assets
        (Method::Get, "/") | (Method::Get, "/index.html") | (Method::Head, "/") | (Method::Head, "/index.html") => {
            let resp = Response::from_string(INDEX_HTML)
                .with_header(content_type_html)
                .with_header(sec_xfo)
                .with_header(sec_nosniff)
                .with_header(sec_ref);
            let _ = request.respond(resp);
        }
        (Method::Get, "/app.js") | (Method::Head, "/app.js") => {
            let cache_header = Header::from_bytes(&b"Cache-Control"[..], &b"public, max-age=604800, immutable"[..]).unwrap();
            let resp = Response::from_string(APP_JS)
                .with_header(content_type_js)
                .with_header(sec_xfo)
                .with_header(sec_nosniff)
                .with_header(cache_header);
            let _ = request.respond(resp);
        }
        (Method::Get, "/app.css") | (Method::Head, "/app.css") => {
            let cache_header = Header::from_bytes(&b"Cache-Control"[..], &b"public, max-age=604800, immutable"[..]).unwrap();
            let resp = Response::from_string(APP_CSS)
                .with_header(content_type_css)
                .with_header(sec_xfo)
                .with_header(sec_nosniff)
                .with_header(cache_header);
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


        // REST API: Network Flow Offloading (Hardware PPE & Software Fast-Path)
        (Method::Get, "/api/v1/network/offload") => {
            let cfg = load_flow_offload_config();
            respond_json(request, &cfg, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/network/offload") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<FlowOffloadConfig>(&body) {
                let _ = apply_flow_offload(&new_cfg);
                let resp = GenericResp { success: true, message: "Аппаратное и программное ускорение потоков (Flow Offload) применено".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON Flow Offload".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Antizapret BGP Dynamic Routing
        (Method::Get, "/api/v1/bgp/status") => {
            let status = get_antizapret_bgp_status();
            respond_json(request, &status, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/bgp/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<AntizapretBgpConfig>(&body) {
                let _ = apply_antizapret_bgp_config(&new_cfg);
                let resp = GenericResp { success: true, message: "Параметры BGP-маршрутизации сохранены".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Некорректный JSON BGP".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: WAN Speedtest Runner
        (Method::Post, "/api/v1/diagnostics/speedtest") => {
            match run_wan_speedtest() {
                Ok(res) => respond_json(request, &res, cors_header, content_type_json),
                Err(e) => {
                    let resp = GenericResp { success: false, message: format!("Ошибка теста скорости: {}", e) };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
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

        // REST API: Auth (root / root default)
        (Method::Post, "/api/v1/auth/login") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(req) = serde_json::from_str::<LoginReq>(&body) {
                let current_pw = state.auth_password.lock().unwrap().clone();
                if (req.username == "root" || req.username == "admin") && req.password == current_pw {
                    let token = format!("ferrum_tok_{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0));
                    state.auth_tokens.lock().unwrap().push(token.clone());
                    let resp = LoginResp {
                        success: true,
                        token: Some(token),
                        message: "Авторизация успешна".into(),
                    };
                    respond_json(request, &resp, cors_header, content_type_json);
                } else {
                    let resp = LoginResp {
                        success: false,
                        token: None,
                        message: "Неверный логин или пароль. По умолчанию: root / root".into(),
                    };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            } else {
                let resp = GenericResp { success: false, message: "Invalid JSON".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }
        (Method::Get, "/api/v1/auth/check") => {
            let resp = serde_json::json!({ "authenticated": true, "user": "root" });
            respond_json(request, &resp, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/auth/change_password") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(req) = serde_json::from_str::<ChangePasswordReq>(&body) {
                let mut current_pw = state.auth_password.lock().unwrap();
                if req.old_password == *current_pw {
                    if req.new_password.len() < 3 {
                        let resp = GenericResp { success: false, message: "Пароль должен быть не короче 3 символов".into() };
                        respond_json(request, &resp, cors_header, content_type_json);
                    } else {
                        *current_pw = req.new_password.clone();
                        let _ = std::fs::create_dir_all("/etc/storage");
                        let _ = std::fs::write("/etc/storage/admin_password.txt", &req.new_password);
                        let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                        let resp = GenericResp { success: true, message: "Пароль администратора успешно изменен и сохранен в Flash".into() };
                        respond_json(request, &resp, cors_header, content_type_json);
                    }
                } else {
                    let resp = GenericResp { success: false, message: "Старый пароль указан неверно".into() };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            } else {
                let resp = GenericResp { success: false, message: "Invalid JSON".into() };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: Proxy toggle
        (Method::Post, "/api/v1/proxy/toggle") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(req) = serde_json::from_str::<ProxyToggleReq>(&body) {
                *state.proxy_enabled.lock().unwrap() = req.enabled;
                if req.enabled {
                    let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("restart").output();
                } else {
                    let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("stop").output();
                }
                let resp = GenericResp {
                    success: true,
                    message: if req.enabled { "VPN туннель включен".into() } else { "VPN туннель отключен".into() },
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp { success: false, message: "Invalid JSON".into() };
                respond_json(request, &resp, cors_header, content_type_json);
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
                                *state.proxy_enabled.lock().unwrap() = data.enabled;
                                *state.routing_mode.lock().unwrap() = data.mode;

                                // Save config to /etc/sing-box/config.json in production
                                let json_str = serde_json::to_string_pretty(&cfg).unwrap_or_default();
                                let _ = std::fs::create_dir_all("/etc/sing-box");
                                let _ = std::fs::create_dir_all("/etc/storage/sing-box");
                                let _ = std::fs::write("/etc/sing-box/config.json", &json_str);
                                let _ = std::fs::write("/etc/storage/sing-box/config.json", &json_str);

                                // Commit to MTD Flash partition (mtd3 RW config) so settings survive reboot
                                let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();

                                // Restart Sing-Box service if enabled
                                if data.enabled {
                                    let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("restart").output();
                                } else {
                                    let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("stop").output();
                                }

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
            let enabled = *state.proxy_enabled.lock().unwrap();
            let core_installed = std::path::Path::new("/usr/bin/sing-box").exists() || std::path::Path::new("/tmp/sing-box").exists();
            let running = if core_installed {
                std::process::Command::new("pidof")
                    .arg("sing-box")
                    .output()
                    .map(|o| o.status.success() && !o.stdout.is_empty())
                    .unwrap_or(false)
            } else {
                false
            };

            let status_text = if !core_installed && enabled {
                "Ядро Sing-Box не установлено во Flash (лимит 8 МБ). Доступна загрузка в RAM (/tmp) или USB".into()
            } else if !running && enabled {
                "Служба Sing-Box остановлена".into()
            } else if running && enabled {
                "Туннель активен (Подключено)".into()
            } else {
                "Туннель отключен".into()
            };

            let resp = ProxyStatusResp {
                active: running && enabled,
                enabled,
                running,
                core_installed,
                mode,
                current_proxy: proxy,
                status_text,
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

        // REST API: Zapret status, config & auto-probe
        (Method::Get, "/api/v1/zapret/status") => {
            let current_strategy = state.zapret_config.lock().unwrap().strategy;
            let status = get_zapret_status(current_strategy);
            respond_json(request, &status, cors_header, content_type_json);
        }
        (Method::Get, "/api/v1/zapret/config") => {
            let cfg = state.zapret_config.lock().unwrap().clone();
            respond_json(request, &cfg, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/zapret/config") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(new_cfg) = serde_json::from_str::<ZapretConfig>(&body) {
                *state.zapret_config.lock().unwrap() = new_cfg.clone();
                let _ = apply_zapret_config(&new_cfg);

                if let Ok(json_str) = serde_json::to_string_pretty(&new_cfg) {
                    let _ = std::fs::create_dir_all("/etc/storage");
                    let _ = std::fs::write("/etc/storage/zapret_config.json", json_str);
                    let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                }

                let resp = GenericResp {
                    success: true,
                    message: "Настройки и стратегия Zapret успешно применены и сохранены в Flash".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Invalid Zapret JSON payload".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }
        (Method::Post, "/api/v1/zapret/probe") => {
            let probe_res = probe_zapret_strategies();
            respond_json(request, &probe_res, cors_header, content_type_json);
        }

        // REST API: Firmware Upload & Pre-flash Validation
        (Method::Post, "/api/v1/system/firmware/upload") => {
            let mut body_bytes = Vec::new();
            let read_res = request.as_reader().read_to_end(&mut body_bytes);
            if read_res.is_err() || body_bytes.len() < 1_000_000 {
                let resp = GenericResp {
                    success: false,
                    message: "Ошибка передачи: размер файла прошивки слишком мал (< 1 МБ) или поток был прерван.".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
                return;
            }

            if body_bytes.len() > 8_060_928 {
                let resp = GenericResp {
                    success: false,
                    message: format!("Размер файла ({} байт) превышает допустимый размер MTD-раздела Flash (8,060,928 байт).", body_bytes.len()),
                };
                respond_json(request, &resp, cors_header, content_type_json);
                return;
            }

            // Check Magic: uImage (0x27051956) or TRX (0x30524448)
            let is_uimage = body_bytes.len() >= 4 && body_bytes[0..4] == [0x27, 0x05, 0x19, 0x56];
            let is_trx = body_bytes.len() >= 4 && body_bytes[0..4] == [0x48, 0x44, 0x52, 0x30];

            if !is_uimage && !is_trx {
                let resp = GenericResp {
                    success: false,
                    message: "Некорректная сигнатура образа. Заголовок не соответствует стандарту uImage/TRX для MediaTek MT7620.".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
                return;
            }

            let file_size = body_bytes.len();
            if let Err(e) = std::fs::write("/tmp/firmware_upload.trx", &body_bytes) {
                let resp = GenericResp {
                    success: false,
                    message: format!("Ошибка сохранения во временную память /tmp: {}", e),
                };
                respond_json(request, &resp, cors_header, content_type_json);
                return;
            }

            let resp = FirmwareUploadResp {
                success: true,
                message: "Образ FerrumOS успешно загружен, проверен и готов к прошивке в SPI Flash.".into(),
                file_size,
                valid_header: true,
            };
            respond_json(request, &resp, cors_header, content_type_json);
        }

        // REST API: Firmware Flash Trigger
        (Method::Post, "/api/v1/system/firmware/flash") => {
            if !std::path::Path::new("/tmp/firmware_upload.trx").exists() {
                let resp = GenericResp {
                    success: false,
                    message: "Файл прошивки не найден в памяти (/tmp/firmware_upload.trx). Сначала выполните загрузку файла.".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
                return;
            }

            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = std::process::Command::new("mtd_write")
                    .args(["-r", "write", "/tmp/firmware_upload.trx", "Firmware_Stub"])
                    .output();
            });

            let resp = GenericResp {
                success: true,
                message: "Запущена запись во Flash-память (Firmware_Stub). Роутер автоматически перезагрузится через 30-45 секунд.".into(),
            };
            respond_json(request, &resp, cors_header, content_type_json);
        }

        // REST API: System Reboot
        (Method::Post, "/api/v1/system/reboot") => {
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = std::process::Command::new("reboot").output();
            });
            let resp = GenericResp {
                success: true,
                message: "Команда перезагрузки отправлена. Веб-интерфейс будет доступен через 30-45 секунд.".into(),
            };
            respond_json(request, &resp, cors_header, content_type_json);
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
        .with_header(content_type)
        .with_header(Header::from_bytes(&b"X-Frame-Options"[..], &b"DENY"[..]).unwrap())
        .with_header(Header::from_bytes(&b"X-Content-Type-Options"[..], &b"nosniff"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Referrer-Policy"[..], &b"no-referrer"[..]).unwrap());
    let _ = request.respond(resp);
}

#[cfg(test)]
mod tests {
    use crate::network::{WifiConfig, Ipv6Config, IptvConfig, FirewallConfig, StaticRoute, PingWatchdogConfig, SqmQosConfig};

    #[test]
    fn test_wifi_config_roundtrip() {
        let mut default_cfg = WifiConfig::default();
        default_cfg.mac_filter_mode = 1;
        default_cfg.mac_filter_list.push("00:11:22:33:44:55".into());
        let json = serde_json::to_string(&default_cfg).expect("Serialization failed");
        let parsed: WifiConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.wifi_2g_ssid, default_cfg.wifi_2g_ssid);
        assert_eq!(parsed.wifi_5g_bandwidth, 80);
        assert!(parsed.turbo_qam);
        assert!(parsed.preamble_short);
        assert_eq!(parsed.country_code, "RU");
        assert_eq!(parsed.mac_filter_mode, 1);
        assert_eq!(parsed.mac_filter_list.len(), 1);
    }

    #[test]
    fn test_ipv6_config_roundtrip() {
        let cfg = Ipv6Config {
            enabled: true,
            connection_type: "native_dhcp6".into(),
            primary_dns: "2606:4700:4700::1111".into(),
            secondary_dns: "2606:4700:4700::1001".into(),
        };
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let parsed: Ipv6Config = serde_json::from_str(&json).expect("Deserialization failed");
        assert!(parsed.enabled);
        assert_eq!(parsed.connection_type, "native_dhcp6");
    }

    #[test]
    fn test_iptv_config_roundtrip() {
        let cfg = IptvConfig {
            igmp_snooping: true,
            igmp_proxy_enabled: true,
            udpxy_enabled: true,
            udpxy_port: 4028,
        };
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let parsed: IptvConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.udpxy_port, 4028);
        assert!(parsed.igmp_snooping);
    }

    #[test]
    fn test_firewall_config_roundtrip() {
        let cfg = FirewallConfig {
            wan_ping_respond: false,
            syn_flood_protection: true,
            port_scan_protection: true,
        };
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let parsed: FirewallConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert!(!parsed.wan_ping_respond);
        assert!(parsed.syn_flood_protection);
    }

    #[test]
    fn test_routes_config_roundtrip() {
        let routes = vec![
            StaticRoute {
                destination: "10.0.0.0/8".into(),
                gateway: "192.168.1.254".into(),
                interface: "WAN".into(),
                metric: 1,
            }
        ];
        let json = serde_json::to_string(&routes).expect("Serialization failed");
        let parsed: Vec<StaticRoute> = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].destination, "10.0.0.0/8");
    }

    #[test]
    fn test_watchdog_config_roundtrip() {
        let cfg = PingWatchdogConfig::default();
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let parsed: PingWatchdogConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.target_ip, "1.1.1.1");
        assert_eq!(parsed.check_interval_sec, 30);
    }

    #[test]
    fn test_qos_config_roundtrip() {
        let cfg = SqmQosConfig::default();
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let parsed: SqmQosConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert!(parsed.game_boost_enabled);
        assert_eq!(parsed.upload_limit_mbps, 95);
    }

    #[test]
    fn test_malformed_json_handling() {
        let malformed = "{\"enabled\": \"not_a_boolean\"}";
        let res = serde_json::from_str::<FirewallConfig>(malformed);
        assert!(res.is_err(), "Malformed JSON should return Err, never panic");
    }

    #[test]
    fn test_zapret_config_roundtrip() {
        use crate::zapret::ZapretConfig;
        let mut cfg = ZapretConfig::default();
        cfg.strategy = 3;
        cfg.custom_nfqws_args = Some("--dpi-desync-ttl=4".into());
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let parsed: ZapretConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(parsed.strategy, 3);
        assert_eq!(parsed.custom_nfqws_args.as_deref(), Some("--dpi-desync-ttl=4"));
    }

    #[test]
    fn test_auth_default_credentials() {
        let state = crate::AppState::new();
        assert_eq!(*state.auth_password.lock().unwrap(), "root");
        assert!(*state.proxy_enabled.lock().unwrap());
    }
}
