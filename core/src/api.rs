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
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tiny_http::{Header, Method, Request, Response, StatusCode};

#[derive(Clone)]
pub struct AppState {
    pub proxy_config: Arc<Mutex<Option<ProxyConfig>>>,
    pub proxy_enabled: Arc<Mutex<bool>>,
    pub routing_mode: Arc<Mutex<RoutingMode>>,
    pub proxy_user_state: Arc<Mutex<Option<ProxyUserState>>>,
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

        if !zapret_cfg.enabled {
            let _ = std::process::Command::new("/etc/init.d/S30zapret").arg("stop").output();
        } else {
            let _ = apply_zapret_config(&zapret_cfg);
        }

        let admin_pw = std::fs::read_to_string("/etc/storage/admin_password.txt")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "root".into());

        let saved_proxy_user = std::fs::read_to_string("/etc/storage/proxy_user_config.json")
            .ok()
            .and_then(|s| serde_json::from_str::<ProxyUserState>(&s).ok());

        let (init_proxy_cfg, init_proxy_enabled, init_routing_mode) = if let Some(ref pus) = saved_proxy_user {
            let parsed = parse_proxy_uri(&pus.uri).ok();
            (parsed, pus.enabled, pus.mode.clone())
        } else {
            (None, false, RoutingMode::SmartRu)
        };

        if init_proxy_enabled {
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(3));
                if std::path::Path::new("/usr/bin/sing-box").exists()
                    || std::path::Path::new("/tmp/sing-box").exists()
                    || std::path::Path::new("/etc/storage/sing-box.gz").exists()
                {
                    let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("start").output();
                }
            });
        }

        let proxy_enabled_arc = Arc::new(Mutex::new(init_proxy_enabled));
        let watchdog_proxy = Arc::clone(&proxy_enabled_arc);
        std::thread::spawn(move || {
            let mut last_restart = std::time::Instant::now() - std::time::Duration::from_secs(60);
            loop {
                std::thread::sleep(std::time::Duration::from_secs(10));
                let enabled = *watchdog_proxy.lock().unwrap();
                if enabled {
                    let alive = std::process::Command::new("pidof")
                        .arg("sing-box")
                        .output()
                        .map(|o| o.status.success() && !o.stdout.is_empty())
                        .unwrap_or(false);
                    let core_available = is_valid_elf_file("/tmp/sing-box") || is_valid_elf_file("/usr/bin/sing-box");

                    if !alive {
                        if core_available {
                            if last_restart.elapsed() >= std::time::Duration::from_secs(30) {
                                eprintln!("[FerrumOS Watchdog] Sing-Box enabled but not running. Restarting S40singbox...");
                                let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("restart").output();
                                last_restart = std::time::Instant::now();
                            }
                        } else if std::path::Path::new("/sys/class/net/tun0").exists() {
                            // If sing-box is not running and core is missing, flush rules to prevent network blackhole
                            let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("stop").output();
                        }
                    }
                }
            }
        });

        AppState {
            proxy_config: Arc::new(Mutex::new(init_proxy_cfg)),
            proxy_enabled: proxy_enabled_arc,
            routing_mode: Arc::new(Mutex::new(init_routing_mode)),
            proxy_user_state: Arc::new(Mutex::new(saved_proxy_user)),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyUserState {
    pub uri: String,
    pub mode: RoutingMode,
    #[serde(default)]
    pub custom_domains: Vec<String>,
    #[serde(default)]
    pub custom_ips: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBackup {
    pub version: String,
    pub timestamp: u64,
    pub proxy_user_state: Option<ProxyUserState>,
    pub zapret_config: Option<ZapretConfig>,
    pub wifi_config: Option<WifiConfig>,
    pub lan_config: Option<LanDhcpConfig>,
    pub wan_config: Option<WanConfig>,
    pub dns_config: Option<DnsAdvancedConfig>,
    pub firewall_config: Option<FirewallConfig>,
    pub qos_config: Option<SqmQosConfig>,
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
pub struct ProxyStatusResp {
    pub active: bool,
    pub enabled: bool,
    pub running: bool,
    pub core_installed: bool,
    pub core_path: Option<String>,
    pub lan_routing: bool,
    pub mode: RoutingMode,
    pub current_proxy: Option<ProxyConfig>,
    pub raw_uri: Option<String>,
    pub custom_domains: Vec<String>,
    pub custom_ips: Vec<String>,
    pub status_text: String,
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

#[derive(Deserialize)]
struct InstallCoreReq {
    url: Option<String>,
}

#[derive(Serialize)]
struct CoreUploadResp {
    success: bool,
    message: String,
    file_size: usize,
}

fn is_valid_elf_file(path: &str) -> bool {
    use std::io::Read;
    if let Ok(mut f) = std::fs::File::open(path) {
        let mut header = [0u8; 4];
        if f.read_exact(&mut header).is_ok() {
            return header == [0x7f, b'E', b'L', b'F'];
        }
    }
    false
}

fn find_elf_recursively(dir: &str) -> Option<std::path::PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(found) = find_elf_recursively(p.to_str().unwrap_or_default()) {
                    return Some(found);
                }
            } else if p.is_file() {
                let p_str = p.to_str().unwrap_or_default();
                if is_valid_elf_file(p_str) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn extract_singbox_payload(archive_path: &str) -> Result<usize, String> {
    // 1. Direct ELF binary:
    if is_valid_elf_file(archive_path) {
        let _ = std::fs::rename(archive_path, "/tmp/sing-box");
        let _ = std::process::Command::new("chmod").args(["755", "/tmp/sing-box"]).output();
        let size = std::fs::metadata("/tmp/sing-box").map(|m| m.len() as usize).unwrap_or(0);
        return Ok(size);
    }

    // 2. Gzip stream (.gz or .tar.gz):
    let _ = std::fs::create_dir_all("/tmp/sb_unpack");
    let _ = std::process::Command::new("sh")
        .args(["-c", &format!("gzip -dc {} > /tmp/sb_unpack/decompressed.tmp 2>/dev/null", archive_path)])
        .output();

    if is_valid_elf_file("/tmp/sb_unpack/decompressed.tmp") {
        let _ = std::fs::rename("/tmp/sb_unpack/decompressed.tmp", "/tmp/sing-box");
        let _ = std::fs::remove_dir_all("/tmp/sb_unpack");
        let _ = std::fs::remove_file(archive_path);
        let _ = std::process::Command::new("chmod").args(["755", "/tmp/sing-box"]).output();
        let size = std::fs::metadata("/tmp/sing-box").map(|m| m.len() as usize).unwrap_or(0);
        return Ok(size);
    }

    // 3. Tar archive:
    let _ = std::process::Command::new("sh")
        .args(["-c", &format!(
            "tar -xf /tmp/sb_unpack/decompressed.tmp -C /tmp/sb_unpack 2>/dev/null || tar -xzf {} -C /tmp/sb_unpack 2>/dev/null",
            archive_path
        )])
        .output();
    let _ = std::fs::remove_file("/tmp/sb_unpack/decompressed.tmp");
    let _ = std::fs::remove_file(archive_path);

    // 4. Find ELF inside unpacked dir:
    if let Some(elf_path) = find_elf_recursively("/tmp/sb_unpack") {
        let _ = std::fs::rename(&elf_path, "/tmp/sing-box");
        let _ = std::fs::remove_dir_all("/tmp/sb_unpack");
        let _ = std::process::Command::new("chmod").args(["755", "/tmp/sing-box"]).output();
        let size = std::fs::metadata("/tmp/sing-box").map(|m| m.len() as usize).unwrap_or(0);
        return Ok(size);
    }
    let _ = std::fs::remove_dir_all("/tmp/sb_unpack");

    if is_valid_elf_file("/tmp/sing-box") {
        let _ = std::process::Command::new("chmod").args(["755", "/tmp/sing-box"]).output();
        let size = std::fs::metadata("/tmp/sing-box").map(|m| m.len() as usize).unwrap_or(0);
        return Ok(size);
    }

    let _ = std::fs::remove_file("/tmp/sing-box");
    Err("Не удалось обнаружить валидный исполняемый файл ELF (sing-box) в загруженном файле или архиве".into())
}

fn install_singbox_core(url: Option<&str>) -> Result<String, String> {
    // 1. Clean up old temporary files to maximize free RAM in /tmp (tmpfs)
    let _ = std::fs::remove_file("/tmp/sing-box.download");
    let _ = std::fs::remove_file("/tmp/sing-box.tar.gz");
    let _ = std::fs::remove_file("/tmp/sing-box.gz");

    // Fast CDN & GitHub mirrors:
    const JSDELIVR_GZ_URL: &str = "https://cdn.jsdelivr.net/gh/MikhaelCat/FerrumOS@main/bin/sing-box.gz";
    const GITHUB_RAW_GZ_URL: &str = "https://raw.githubusercontent.com/MikhaelCat/FerrumOS/main/bin/sing-box.gz";
    const GITHUB_RELEASE_URL: &str = "https://github.com/SagerNet/sing-box/releases/download/v1.10.7/sing-box-1.10.7-linux-mipsle.tar.gz";

    let urls_to_try: Vec<&str> = if let Some(u) = url {
        vec![u]
    } else {
        vec![JSDELIVR_GZ_URL, GITHUB_RAW_GZ_URL, GITHUB_RELEASE_URL]
    };

    let mut download_success = false;
    for target_url in urls_to_try {
        let output = std::process::Command::new("wget")
            .args(["--no-check-certificate", "-q", "-T", "35", "-O", "/tmp/sing-box.download", target_url])
            .output();

        if let Ok(out) = output {
            if out.status.success() && std::path::Path::new("/tmp/sing-box.download").exists() {
                if let Ok(meta) = std::fs::metadata("/tmp/sing-box.download") {
                    if meta.len() >= 100_000 {
                        download_success = true;
                        break;
                    }
                }
            }
        }
    }

    if !download_success {
        let _ = std::fs::remove_file("/tmp/sing-box.download");
        return Err("Не удалось загрузить бинарник ядра Sing-Box через wget. Проверьте подключение к сети или загрузите файл ядра sing-box вручную через веб-интерфейс.".into());
    }

    let sz = extract_singbox_payload("/tmp/sing-box.download")?;
    Ok(format!("Ядро Sing-Box (MIPS) успешно распаковано в RAM (/tmp/sing-box, {} байт) и готово к работе", sz))
}

fn upload_singbox_core(bytes: &[u8]) -> Result<usize, String> {
    if bytes.len() < 100_000 {
        return Err("Размер файла слишком мал для исполняемого ядра Sing-Box".into());
    }

    let tmp_path = "/tmp/singbox_uploaded.bin";
    std::fs::write(tmp_path, bytes).map_err(|e| format!("Ошибка записи во временный файл: {}", e))?;

    extract_singbox_payload(tmp_path)
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
            let cache_header = Header::from_bytes(&b"Cache-Control"[..], &b"no-cache, no-store, must-revalidate"[..]).unwrap();
            let resp = Response::from_string(APP_JS)
                .with_header(content_type_js)
                .with_header(sec_xfo)
                .with_header(sec_nosniff)
                .with_header(cache_header);
            let _ = request.respond(resp);
        }
        (Method::Get, "/app.css") | (Method::Head, "/app.css") => {
            let cache_header = Header::from_bytes(&b"Cache-Control"[..], &b"no-cache, no-store, must-revalidate"[..]).unwrap();
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
                if let Some(ref mut pus) = *state.proxy_user_state.lock().unwrap() {
                    pus.enabled = req.enabled;
                    if let Ok(user_json) = serde_json::to_string_pretty(pus) {
                        let _ = std::fs::write("/etc/storage/proxy_user_config.json", user_json);
                        let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();
                    }
                }
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
                                *state.proxy_config.lock().unwrap() = Some(parsed.clone());
                                *state.proxy_enabled.lock().unwrap() = data.enabled;
                                *state.routing_mode.lock().unwrap() = data.mode.clone();

                                let user_state = ProxyUserState {
                                    uri: data.uri.clone(),
                                    mode: data.mode.clone(),
                                    custom_domains: data.custom_domains.clone(),
                                    custom_ips: data.custom_ips.clone(),
                                    enabled: data.enabled,
                                };
                                *state.proxy_user_state.lock().unwrap() = Some(user_state.clone());
                                if let Ok(user_json) = serde_json::to_string_pretty(&user_state) {
                                    let _ = std::fs::create_dir_all("/etc/storage");
                                    let _ = std::fs::write("/etc/storage/proxy_user_config.json", user_json);
                                }

                                // Save config to /etc/sing-box/config.json in production
                                let json_str = serde_json::to_string_pretty(&cfg).unwrap_or_default();
                                let _ = std::fs::create_dir_all("/etc/sing-box");
                                let _ = std::fs::create_dir_all("/etc/storage/sing-box");
                                let _ = std::fs::write("/etc/sing-box/config.json", &json_str);
                                let _ = std::fs::write("/etc/storage/sing-box/config.json", &json_str);

                                // Save proxy server address for firewall and Zapret bypass
                                // Resolve hostname to numerical IPv4 to ensure iptables rules never fail
                                let resolved_ip = format!("{}:{}", parsed.server, parsed.server_port)
                                    .to_socket_addrs()
                                    .ok()
                                    .and_then(|mut addrs| addrs.next())
                                    .map(|a| a.ip().to_string())
                                    .unwrap_or_else(|| parsed.server.clone());

                                let _ = std::fs::write("/etc/sing-box/proxy_server.txt", &resolved_ip);
                                let _ = std::fs::write("/etc/storage/sing-box/proxy_server.txt", &resolved_ip);

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
                                    message: "Конфигурация туннеля успешно сохранена в Flash и применена".into(),
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
            let (core_installed, core_path) = if is_valid_elf_file("/tmp/sing-box") {
                (true, Some("/tmp/sing-box".to_string()))
            } else if is_valid_elf_file("/usr/bin/sing-box") {
                (true, Some("/usr/bin/sing-box".to_string()))
            } else {
                (false, None)
            };
            let lan_routing = std::path::Path::new("/sys/class/net/tun0").exists();
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
                "Ядро Sing-Box не установлено в RAM (/tmp/sing-box). Загрузите ядро кнопкой ниже.".into()
            } else if !running && enabled {
                "Служба Sing-Box остановлена".into()
            } else if running && enabled && lan_routing {
                "Туннель активен. Маршрутизация LAN (br0 -> tun0) работает в штатном режиме.".into()
            } else if running && enabled {
                "Служба Sing-Box запущена, туннель инициализируется...".into()
            } else {
                "Туннель отключен".into()
            };

            let user_state = state.proxy_user_state.lock().unwrap().clone();
            let raw_uri = user_state.as_ref().map(|u| u.uri.clone()).or_else(|| proxy.as_ref().map(|p| p.raw_uri.clone()));
            let custom_domains = user_state.as_ref().map(|u| u.custom_domains.clone()).unwrap_or_default();
            let custom_ips = user_state.as_ref().map(|u| u.custom_ips.clone()).unwrap_or_default();

            let resp = ProxyStatusResp {
                active: running && enabled,
                enabled,
                running,
                core_installed,
                core_path,
                lan_routing,
                mode,
                current_proxy: proxy,
                raw_uri,
                custom_domains,
                custom_ips,
                status_text,
            };
            respond_json(request, &resp, cors_header, content_type_json);
        }

        // REST API: Proxy core install (auto-download into /tmp/sing-box)
        (Method::Post, "/api/v1/proxy/core/install") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let req: Result<InstallCoreReq, _> = serde_json::from_str(&body);
            let url_opt = req.ok().and_then(|r| r.url);
            match install_singbox_core(url_opt.as_deref()) {
                Ok(msg) => {
                    if *state.proxy_enabled.lock().unwrap() {
                        let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("restart").output();
                    }
                    let resp = GenericResp { success: true, message: msg };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
                Err(e) => {
                    let resp = GenericResp { success: false, message: e };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            }
        }

        // REST API: Proxy core upload (manual binary/gzip upload into /tmp/sing-box)
        (Method::Post, "/api/v1/proxy/core/upload") => {
            let mut body_bytes = Vec::new();
            let read_res = request.as_reader().read_to_end(&mut body_bytes);
            if read_res.is_err() || body_bytes.len() < 100_000 {
                let resp = GenericResp {
                    success: false,
                    message: "Ошибка передачи: размер файла ядра слишком мал (< 100 КБ) или поток был прерван".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
                return;
            }

            match upload_singbox_core(&body_bytes) {
                Ok(sz) => {
                    if *state.proxy_enabled.lock().unwrap() {
                        let _ = std::process::Command::new("/etc/init.d/S40singbox").arg("restart").output();
                    }
                    let resp = CoreUploadResp {
                        success: true,
                        message: format!("Исполняемое ядро Sing-Box успешно сохранено в RAM (/tmp/sing-box, {} байт)", sz),
                        file_size: sz,
                    };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
                Err(e) => {
                    let resp = GenericResp {
                        success: false,
                        message: format!("Ошибка установки ядра: {}", e),
                    };
                    respond_json(request, &resp, cors_header, content_type_json);
                }
            }
        }

        // REST API: Ping test
        (Method::Post, "/api/v1/proxy/test") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            let req: Result<TestPingReq, _> = serde_json::from_str(&body);

            match req {
                Ok(data) => {
                    let addr_str = format!("{}:{}", data.host, data.port);
                    let start = Instant::now();
                    let target_addr = match addr_str.to_socket_addrs() {
                        Ok(mut addrs) => addrs.next(),
                        Err(_) => None,
                    };

                    match target_addr {
                        Some(sa) => {
                            match std::net::TcpStream::connect_timeout(&sa, std::time::Duration::from_millis(2500)) {
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
                                        error: Some(format!("Таймаут подключения ({}): {}", addr_str, e)),
                                    };
                                    respond_json(request, &resp, cors_header, content_type_json);
                                }
                            }
                        }
                        None => {
                            let resp = TestPingResp {
                                success: false,
                                latency_ms: None,
                                error: Some(format!("Не удалось разрешить DNS-имя хоста: {}", data.host)),
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
        (Method::Post, "/api/v1/zapret/restart") => {
            let cfg = state.zapret_config.lock().unwrap().clone();
            let _ = apply_zapret_config(&cfg);
            let status = get_zapret_status(cfg.strategy);
            let resp = GenericResp {
                success: status.running,
                message: if status.running {
                    format!("Служба Zapret успешно перезапущена (PID {})", status.pid.unwrap_or(0))
                } else {
                    "Служба Zapret перезапущена".into()
                },
            };
            respond_json(request, &resp, cors_header, content_type_json);
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
                // Anti-brick protection: copy mtd_write to RAM (/tmp) so writing squashfs never page-faults mid-erase
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("cp /sbin/mtd_write /tmp/mtd_write 2>/dev/null || true; chmod 755 /tmp/mtd_write 2>/dev/null || true; sync; echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || true; /tmp/mtd_write -r write /tmp/firmware_upload.trx Firmware_Stub 2>&1 | logger -t ferrum_flash")
                    .output();
            });

            let resp = GenericResp {
                success: true,
                message: "Запущена запись во Flash-память (Firmware_Stub). Роутер автоматически перезагрузится через 30-45 секунд.".into(),
            };
            respond_json(request, &resp, cors_header, content_type_json);
        }

        // REST API: System Configuration Backup & Restore
        (Method::Get, "/api/v1/system/backup") => {
            let backup = SystemBackup {
                version: "1.0.0".into(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                proxy_user_state: state.proxy_user_state.lock().unwrap().clone(),
                zapret_config: Some(state.zapret_config.lock().unwrap().clone()),
                wifi_config: Some(state.wifi_config.lock().unwrap().clone()),
                lan_config: Some(state.lan_config.lock().unwrap().clone()),
                wan_config: Some(state.wan_config.lock().unwrap().clone()),
                dns_config: Some(state.dns_config.lock().unwrap().clone()),
                firewall_config: Some(state.firewall_config.lock().unwrap().clone()),
                qos_config: Some(state.qos_config.lock().unwrap().clone()),
            };
            respond_json(request, &backup, cors_header, content_type_json);
        }
        (Method::Post, "/api/v1/system/restore") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(backup) = serde_json::from_str::<SystemBackup>(&body) {
                if let Some(pus) = backup.proxy_user_state {
                    *state.proxy_user_state.lock().unwrap() = Some(pus.clone());
                    if let Ok(js) = serde_json::to_string_pretty(&pus) {
                        let _ = std::fs::create_dir_all("/etc/storage");
                        let _ = std::fs::write("/etc/storage/proxy_user_config.json", js);
                    }
                }
                if let Some(zc) = backup.zapret_config {
                    *state.zapret_config.lock().unwrap() = zc.clone();
                    let _ = apply_zapret_config(&zc);
                }
                if let Some(wc) = backup.wifi_config {
                    *state.wifi_config.lock().unwrap() = wc.clone();
                    let _ = apply_wifi_config(&wc);
                    if let Ok(js) = serde_json::to_string_pretty(&wc) {
                        let _ = std::fs::write("/etc/storage/wifi_config.json", js);
                    }
                }
                if let Some(lc) = backup.lan_config {
                    *state.lan_config.lock().unwrap() = lc.clone();
                    let _ = apply_lan_config(&lc);
                    if let Ok(js) = serde_json::to_string_pretty(&lc) {
                        let _ = std::fs::write("/etc/storage/lan_config.json", js);
                    }
                }
                if let Some(wanc) = backup.wan_config {
                    *state.wan_config.lock().unwrap() = wanc.clone();
                    let _ = apply_wan_config(&wanc);
                    if let Ok(js) = serde_json::to_string_pretty(&wanc) {
                        let _ = std::fs::write("/etc/storage/wan_config.json", js);
                    }
                }
                if let Some(dc) = backup.dns_config {
                    *state.dns_config.lock().unwrap() = dc.clone();
                    let _ = apply_dns_config(&dc);
                    if let Ok(js) = serde_json::to_string_pretty(&dc) {
                        let _ = std::fs::write("/etc/storage/dns_config.json", js);
                    }
                }
                if let Some(fc) = backup.firewall_config {
                    *state.firewall_config.lock().unwrap() = fc.clone();
                    let _ = apply_firewall_config(&fc);
                    if let Ok(js) = serde_json::to_string_pretty(&fc) {
                        let _ = std::fs::write("/etc/storage/firewall_config.json", js);
                    }
                }
                if let Some(qc) = backup.qos_config {
                    *state.qos_config.lock().unwrap() = qc.clone();
                    let _ = apply_sqm_qos(&qc);
                    if let Ok(js) = serde_json::to_string_pretty(&qc) {
                        let _ = std::fs::write("/etc/storage/qos_config.json", js);
                    }
                }
                let _ = std::process::Command::new("mtd_storage.sh").arg("save").output();

                let resp = GenericResp {
                    success: true,
                    message: "Конфигурация успешно восстановлена и сохранена в Flash".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            } else {
                let resp = GenericResp {
                    success: false,
                    message: "Ошибка разбора файла резервной копии. Проверьте формат JSON.".into(),
                };
                respond_json(request, &resp, cors_header, content_type_json);
            }
        }

        // REST API: System Reboot
        (Method::Post, "/api/v1/system/reboot") => {
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("mtd_storage.sh save 2>/dev/null || true; sync; reboot")
                    .output();
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
        assert_eq!(parsed.wifi_5g_bandwidth, 0);
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
        assert!(!*state.proxy_enabled.lock().unwrap());
    }
}
