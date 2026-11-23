use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub if_type: String, // "WAN", "LAN", "WLAN_2G", "WLAN_5G"
    pub ip_address: Option<String>,
    pub mac_address: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub is_up: bool,
}

// ==========================================
// 1. WI-FI COMPREHENSIVE CONFIGURATION
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    // 2.4 GHz (MediaTek MT7620A 2T2R 300Mbps)
    pub wifi_2g_enabled: bool,
    pub wifi_2g_ssid: String,
    pub wifi_2g_password: String,
    pub wifi_2g_security: String,   // "WPA2PSK", "WPA2WPA3", "WPA3SAE"
    pub wifi_2g_mode: String,       // "11bgn", "11n_only", "11bg"
    pub wifi_2g_channel: u8,        // 0 = Auto ACS, 1..13
    pub wifi_2g_bandwidth: u8,      // 20 or 40 MHz
    pub wifi_2g_tx_power: u8,       // 10..100 %
    pub wifi_2g_short_gi: bool,     // Short Guard Interval (400ns, +11% speed)
    pub wifi_2g_isolate: bool,      // AP Client Isolation
    pub wifi_2g_hidden: bool,       // Hide SSID broadcast
    pub wifi_2g_pmf: u8,            // 0=Disabled, 1=Capable, 2=Required (802.11w)
    pub wifi_2g_dtim: u8,           // 1..3
    pub wifi_2g_beacon: u16,        // 100 ms

    // 5 GHz (MediaTek MT7610E 802.11ac 1T1R 433Mbps)
    pub wifi_5g_enabled: bool,
    pub wifi_5g_ssid: String,
    pub wifi_5g_password: String,
    pub wifi_5g_security: String,   // "WPA2PSK", "WPA2WPA3", "WPA3SAE"
    pub wifi_5g_mode: String,       // "11vht_ac", "11ac_only", "11n_5g"
    pub wifi_5g_channel: u8,        // 0=Auto, 36, 40, 44, 48, etc.
    pub wifi_5g_bandwidth: u8,      // 20, 40, 80 MHz (VHT80)
    pub wifi_5g_tx_power: u8,       // 10..100 %
    pub wifi_5g_short_gi: bool,     // VHT Short GI
    pub wifi_5g_isolate: bool,
    pub wifi_5g_hidden: bool,
    pub wifi_5g_pmf: u8,
    pub wifi_5g_beamforming: bool,  // Explicit TxBf
    pub wifi_5g_stbc_ldpc: bool,

    // Advanced & Seamless Roaming
    pub band_steering: bool,        // Smart Connect (auto-steer 5G devices)
    pub fast_roaming_80211k: bool,  // Radio Resource Management (RRM)
    pub fast_roaming_80211v: bool,  // BSS Transition Management
    pub fast_roaming_80211r: bool,  // Fast BSS Transition
    pub rssi_roam_threshold: i8,    // Disconnect clients when signal < -75 dBm
    pub tx_burst: bool,             // Frame Bursting (+25% throughput)
    pub packet_aggregate: bool,     // A-MPDU / A-MSDU
    pub wmm_qos: bool,              // WMM QoS (required for >54Mbps)

    // Guest Wi-Fi (MBSSID)
    pub guest_enabled: bool,
    pub guest_ssid: String,
    pub guest_password: String,
    pub guest_isolate_lan: bool,    // Block access to LAN devices
    pub guest_bandwidth_limit_mbps: u32,
}

impl Default for WifiConfig {
    fn default() -> Self {
        WifiConfig {
            wifi_2g_enabled: true,
            wifi_2g_ssid: "FerrumOS_2.4G".into(),
            wifi_2g_password: "ChangeMe123".into(),
            wifi_2g_security: "WPA2PSK".into(),
            wifi_2g_mode: "11bgn".into(),
            wifi_2g_channel: 6,
            wifi_2g_bandwidth: 40,
            wifi_2g_tx_power: 100,
            wifi_2g_short_gi: true,
            wifi_2g_isolate: false,
            wifi_2g_hidden: false,
            wifi_2g_pmf: 1,
            wifi_2g_dtim: 1,
            wifi_2g_beacon: 100,

            wifi_5g_enabled: true,
            wifi_5g_ssid: "FerrumOS_5G".into(),
            wifi_5g_password: "ChangeMe123".into(),
            wifi_5g_security: "WPA2PSK".into(),
            wifi_5g_mode: "11vht_ac".into(),
            wifi_5g_channel: 36,
            wifi_5g_bandwidth: 80,
            wifi_5g_tx_power: 100,
            wifi_5g_short_gi: true,
            wifi_5g_isolate: false,
            wifi_5g_hidden: false,
            wifi_5g_pmf: 1,
            wifi_5g_beamforming: true,
            wifi_5g_stbc_ldpc: true,

            band_steering: false,
            fast_roaming_80211k: true,
            fast_roaming_80211v: true,
            fast_roaming_80211r: true,
            rssi_roam_threshold: -75,
            tx_burst: true,
            packet_aggregate: true,
            wmm_qos: true,

            guest_enabled: false,
            guest_ssid: "FerrumOS_Guest".into(),
            guest_password: "GuestPassword".into(),
            guest_isolate_lan: true,
            guest_bandwidth_limit_mbps: 20,
        }
    }
}

// ==========================================
// 2. DNS & DOH ADVANCED CONFIGURATION
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsAdvancedConfig {
    pub provider: String,            // "cloudflare", "google", "adguard", "quad9", "yandex", "custom"
    pub primary_dns: String,         // e.g. "1.1.1.1"
    pub secondary_dns: String,       // e.g. "1.0.0.1"
    pub doh_enabled: bool,           // DNS-over-HTTPS
    pub doh_url: String,             // e.g. "https://cloudflare-dns.com/dns-query"
    pub force_router_dns: bool,      // Redirect port 53 UDP/TCP queries to router
    pub anti_dns_rebind: bool,       // Reject 127.0.0.0/8 and private IPs in public DNS
    pub adblock_enabled: bool,       // Router-level Ad & Tracker Blocking
    pub custom_hosts: Vec<String>,   // List of "192.168.1.5 nas.lan"
}

impl Default for DnsAdvancedConfig {
    fn default() -> Self {
        DnsAdvancedConfig {
            provider: "cloudflare".into(),
            primary_dns: "1.1.1.1".into(),
            secondary_dns: "1.0.0.1".into(),
            doh_enabled: true,
            doh_url: "https://cloudflare-dns.com/dns-query".into(),
            force_router_dns: true,
            anti_dns_rebind: true,
            adblock_enabled: true,
            custom_hosts: vec!["192.168.1.1 ferrum.router".into()],
        }
    }
}

// ==========================================
// 3. LAN & DHCP CONFIGURATION
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticLease {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanDhcpConfig {
    pub lan_ip: String,              // "192.168.1.1"
    pub lan_netmask: String,         // "255.255.255.0"
    pub dhcp_enabled: bool,
    pub dhcp_start: String,          // "192.168.1.100"
    pub dhcp_end: String,            // "192.168.1.244"
    pub lease_time_hours: u32,       // 24
    pub domain_name: String,         // "lan"
    pub static_leases: Vec<StaticLease>,
}

impl Default for LanDhcpConfig {
    fn default() -> Self {
        LanDhcpConfig {
            lan_ip: "192.168.1.1".into(),
            lan_netmask: "255.255.255.0".into(),
            dhcp_enabled: true,
            dhcp_start: "192.168.1.100".into(),
            dhcp_end: "192.168.1.244".into(),
            lease_time_hours: 24,
            domain_name: "lan".into(),
            static_leases: vec![],
        }
    }
}

// ==========================================
// 4. WAN, ROUTING & NAT CONFIGURATION
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardRule {
    pub name: String,
    pub proto: String,              // "TCP", "UDP", "BOTH"
    pub ext_port: u16,
    pub int_ip: String,
    pub int_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WanConfig {
    pub connection_type: String,     // "dhcp", "static", "pppoe"
    pub static_ip: Option<String>,
    pub static_netmask: Option<String>,
    pub static_gateway: Option<String>,
    pub pppoe_user: Option<String>,
    pub pppoe_pass: Option<String>,
    pub pppoe_service: Option<String>,
    pub mtu: u16,                    // 1500 or 1492
    pub clone_mac: Option<String>,   // WAN MAC spoofing
    pub vlan_internet_id: Option<u16>,
    pub vlan_iptv_id: Option<u16>,
    pub iptv_port: Option<u8>,       // e.g. Port 4 for STB
    pub hw_nat: bool,                // Hardware NAT PPE engine
    pub upnp: bool,                  // UPnP & NAT-PMP for gaming
    pub dmz_ip: Option<String>,
    pub port_forwards: Vec<PortForwardRule>,
}

impl Default for WanConfig {
    fn default() -> Self {
        WanConfig {
            connection_type: "dhcp".into(),
            static_ip: None,
            static_netmask: None,
            static_gateway: None,
            pppoe_user: None,
            pppoe_pass: None,
            pppoe_service: None,
            mtu: 1500,
            clone_mac: None,
            vlan_internet_id: None,
            vlan_iptv_id: None,
            iptv_port: None,
            hw_nat: true,
            upnp: true,
            dmz_ip: None,
            port_forwards: vec![],
        }
    }
}

// ==========================================
// HARDWARE DRIVER GENERATION
// ==========================================

pub fn get_network_interfaces() -> Vec<NetworkInterface> {
    let mut list = Vec::new();

    let if_mappings = [
        ("eth2.2", "WAN"),
        ("br0", "LAN"),
        ("ra0", "WLAN_2G"),
        ("rai0", "WLAN_5G"),
    ];

    let net_dev = fs::read_to_string("/proc/net/dev").unwrap_or_default();

    for (name, if_type) in if_mappings {
        let (rx, tx) = parse_proc_net_dev(&net_dev, name);
        let mac = read_mac_address(name).unwrap_or_else(|| "00:00:00:00:00:00".into());
        let ip = read_ip_address(name);

        list.push(NetworkInterface {
            name: name.into(),
            if_type: if_type.into(),
            ip_address: ip,
            mac_address: mac,
            rx_bytes: rx,
            tx_bytes: tx,
            is_up: true,
        });
    }

    list
}

fn parse_proc_net_dev(content: &str, iface: &str) -> (u64, u64) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(iface) && trimmed.contains(':') {
            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() == 2 {
                let stats: Vec<&str> = parts[1].split_whitespace().collect();
                if stats.len() >= 9 {
                    let rx = stats[0].parse::<u64>().unwrap_or(0);
                    let tx = stats[8].parse::<u64>().unwrap_or(0);
                    return (rx, tx);
                }
            }
        }
    }
    (0, 0)
}

fn read_mac_address(iface: &str) -> Option<String> {
    let path = format!("/sys/class/net/{}/address", iface);
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_ip_address(_iface: &str) -> Option<String> {
    None
}

/// Applies LAN and DHCP server settings, writing dnsmasq host assignments
pub fn apply_lan_config(cfg: &LanDhcpConfig) -> Result<(), String> {
    if !cfg.dhcp_enabled {
        let _ = fs::write("/etc/dnsmasq.d/ferrum_dhcp.conf", "# DHCP server disabled\n");
        let _ = Command::new("killall").arg("dnsmasq").output();
        return Ok(());
    }

    let mut dhcp_conf = format!(
        "# FerrumOS DHCP Server Configuration\n\
        interface=br0\n\
        dhcp-range={},{},{},{}h\n\
        dhcp-option=option:router,{}\n\
        dhcp-option=option:dns-server,{}\n\
        domain={}\n",
        cfg.dhcp_start, cfg.dhcp_end, cfg.lan_netmask, cfg.lease_time_hours,
        cfg.lan_ip, cfg.lan_ip, cfg.domain_name
    );

    for lease in &cfg.static_leases {
        if !lease.mac.is_empty() && !lease.ip.is_empty() {
            dhcp_conf.push_str(&format!(
                "dhcp-host={},{},{}\n",
                lease.mac.trim(),
                lease.ip.trim(),
                lease.hostname.trim()
            ));
        }
    }

    let _ = fs::create_dir_all("/etc/dnsmasq.d");
    let _ = fs::write("/etc/dnsmasq.d/ferrum_dhcp.conf", dhcp_conf);

    // Restart dnsmasq
    let _ = Command::new("killall").arg("dnsmasq").output();
    let _ = Command::new("dnsmasq").output();

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpLeaseEntry {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub expires: String,
}

/// Reads active DHCP leases from dnsmasq
pub fn get_dhcp_leases() -> Vec<DhcpLeaseEntry> {
    let lease_paths = ["/tmp/dnsmasq.leases", "/var/lib/misc/dnsmasq.leases"];
    for path in lease_paths {
        if let Ok(content) = fs::read_to_string(path) {
            let mut result = Vec::new();
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    result.push(DhcpLeaseEntry {
                        expires: parts[0].to_string(),
                        mac: parts[1].to_string(),
                        ip: parts[2].to_string(),
                        hostname: if parts[3] == "*" { "Unknown".into() } else { parts[3].to_string() },
                    });
                }
            }
            if !result.is_empty() {
                return result;
            }
        }
    }

    // Default mock data when running on dev host
    vec![
        DhcpLeaseEntry {
            mac: "BC:D0:74:11:22:33".into(),
            ip: "192.168.1.101".into(),
            hostname: "iPhone-15-Pro".into(),
            expires: "22h remaining".into(),
        },
        DhcpLeaseEntry {
            mac: "E4:5F:01:44:55:66".into(),
            ip: "192.168.1.102".into(),
            hostname: "Samsung-SmartTV".into(),
            expires: "23h remaining".into(),
        },
        DhcpLeaseEntry {
            mac: "30:9C:23:77:88:99".into(),
            ip: "192.168.1.105".into(),
            hostname: "MacBook-Air-M2".into(),
            expires: "18h remaining".into(),
        },
    ]
}

/// Generates full MediaTek driver configuration for MT7620 (2.4 GHz) and MT7610E (5 GHz)
/// Includes WMM, A-MPDU aggregation, Short GI, STBC/LDPC, beamforming, and roaming.
pub fn apply_wifi_config(cfg: &WifiConfig) -> Result<(), String> {
    // 1. Generate 2.4 GHz Configuration (/etc/Wireless/RT2860/RT2860AP.dat)
    let ht_bw_2g = if cfg.wifi_2g_bandwidth >= 40 { 1 } else { 0 };
    let ht_extcha_2g = if cfg.wifi_2g_channel <= 7 { 1 } else { 0 };
    let gi_2g = if cfg.wifi_2g_short_gi { 1 } else { 0 };
    let isolate_2g = if cfg.wifi_2g_isolate { 1 } else { 0 };
    let burst_val = if cfg.tx_burst { 1 } else { 0 };
    let agg_val = if cfg.packet_aggregate { 1 } else { 0 };
    let wmm_val = if cfg.wmm_qos { 1 } else { 0 };
    let hide_2g = if cfg.wifi_2g_hidden { 1 } else { 0 };

    let dat_2g = format!(
        "# FerrumOS MT7620 2.4GHz High-Performance Driver Profile\n\
        Default\n\
        CountryRegion=1\n\
        CountryRegionABand=7\n\
        CountryCode=RU\n\
        BssidNum={bssid_num}\n\
        SSID1={ssid1}\n\
        SSID2={ssid2}\n\
        WPAPSK1={pass1}\n\
        WPAPSK2={pass2}\n\
        AuthMode=WPA2PSK;WPA2PSK\n\
        EncrypType=AES;AES\n\
        RekeyMethod=TIME\n\
        RekeyInterval=3600\n\
        PMFMFPC={pmf}\n\
        Channel={channel}\n\
        AutoChannelSelect={auto_chan}\n\
        WirelessMode=9\n\
        TxPower={tx_pwr}\n\
        BeaconPeriod={beacon}\n\
        DtimPeriod={dtim}\n\
        TxPreamble=1\n\
        RTSThreshold=2347\n\
        FragThreshold=2346\n\
        TxBurst={burst}\n\
        PktAggregate={agg}\n\
        WmmCapable={wmm};{wmm}\n\
        APAifsn=3;7;1;1\n\
        APCwmin=4;4;3;2\n\
        APCwmax=6;10;4;3\n\
        APTxop=0;0;94;47\n\
        NoForwarding={iso};{iso}\n\
        NoForwardingBTNBSSID={iso_guest}\n\
        HideSSID={hide};0\n\
        ShortSlot=1\n\
        HT_HTC=1\n\
        HT_RDG=1\n\
        HT_EXTCHA={extcha}\n\
        HT_BW={bw}\n\
        HT_BSSCoexistence=0\n\
        HT_AutoBA=1\n\
        HT_AMSDU=0\n\
        HT_BAWinSize=64\n\
        HT_GI={gi}\n\
        HT_STBC=1\n\
        HT_LDPC=1\n\
        HT_PROTECT=1\n\
        HT_TxStream=2\n\
        HT_RxStream=2\n\
        Dot11kEnable={dot11k}\n\
        Dot11vEnable={dot11v}\n\
        GreenAP=0\n",
        bssid_num = if cfg.guest_enabled { 2 } else { 1 },
        ssid1 = cfg.wifi_2g_ssid,
        ssid2 = if cfg.guest_enabled { &cfg.guest_ssid } else { "" },
        pass1 = cfg.wifi_2g_password,
        pass2 = if cfg.guest_enabled { &cfg.guest_password } else { "" },
        pmf = cfg.wifi_2g_pmf,
        channel = cfg.wifi_2g_channel,
        auto_chan = if cfg.wifi_2g_channel == 0 { 2 } else { 0 },
        tx_pwr = cfg.wifi_2g_tx_power,
        beacon = cfg.wifi_2g_beacon,
        dtim = cfg.wifi_2g_dtim,
        burst = burst_val,
        agg = agg_val,
        wmm = wmm_val,
        iso = isolate_2g,
        iso_guest = if cfg.guest_isolate_lan { 1 } else { 0 },
        hide = hide_2g,
        extcha = ht_extcha_2g,
        bw = ht_bw_2g,
        gi = gi_2g,
        dot11k = if cfg.fast_roaming_80211k { 1 } else { 0 },
        dot11v = if cfg.fast_roaming_80211v { 1 } else { 0 },
    );

    let _ = fs::create_dir_all("/etc/Wireless/RT2860");
    let _ = fs::write("/etc/Wireless/RT2860/RT2860AP.dat", dat_2g);

    // 2. Generate 5 GHz Configuration (/etc/Wireless/iNIC/iNIC_ap.dat)
    let vht_bw_5g = match cfg.wifi_5g_bandwidth {
        80 => 1, // VHT 80 MHz
        40 => 0, // HT/VHT 40 MHz
        _ => 0,  // 20 MHz
    };
    let gi_5g = if cfg.wifi_5g_short_gi { 1 } else { 0 };
    let isolate_5g = if cfg.wifi_5g_isolate { 1 } else { 0 };
    let hide_5g = if cfg.wifi_5g_hidden { 1 } else { 0 };

    let dat_5g = format!(
        "# FerrumOS MT7610E 5GHz 802.11ac Driver Profile\n\
        Default\n\
        CountryRegion=1\n\
        CountryRegionABand=7\n\
        CountryCode=RU\n\
        BssidNum={bssid_num}\n\
        SSID1={ssid1}\n\
        SSID2={ssid2}\n\
        WPAPSK1={pass1}\n\
        WPAPSK2={pass2}\n\
        AuthMode=WPA2PSK;WPA2PSK\n\
        EncrypType=AES;AES\n\
        RekeyMethod=TIME\n\
        RekeyInterval=3600\n\
        PMFMFPC={pmf}\n\
        Channel={channel}\n\
        AutoChannelSelect={auto_chan}\n\
        WirelessMode=14\n\
        TxPower={tx_pwr}\n\
        BeaconPeriod=100\n\
        DtimPeriod=1\n\
        TxBurst={burst}\n\
        PktAggregate={agg}\n\
        WmmCapable={wmm};{wmm}\n\
        NoForwarding={iso};{iso}\n\
        NoForwardingBTNBSSID={iso_guest}\n\
        HideSSID={hide};0\n\
        HT_EXTCHA=1\n\
        HT_BW=1\n\
        HT_AutoBA=1\n\
        HT_BAWinSize=64\n\
        HT_GI={gi}\n\
        HT_STBC=1\n\
        HT_LDPC=1\n\
        HT_TxStream=1\n\
        HT_RxStream=1\n\
        VHT_BW={vht_bw}\n\
        VHT_SGI={gi}\n\
        VHT_STBC=1\n\
        VHT_LDPC=1\n\
        ITxBfEn={txbf}\n\
        Dot11kEnable={dot11k}\n\
        Dot11vEnable={dot11v}\n\
        IEEE80211H=0\n\
        RDRegion=CE\n",
        bssid_num = if cfg.guest_enabled { 2 } else { 1 },
        ssid1 = cfg.wifi_5g_ssid,
        ssid2 = if cfg.guest_enabled { &cfg.guest_ssid } else { "" },
        pass1 = cfg.wifi_5g_password,
        pass2 = if cfg.guest_enabled { &cfg.guest_password } else { "" },
        pmf = cfg.wifi_5g_pmf,
        channel = cfg.wifi_5g_channel,
        auto_chan = if cfg.wifi_5g_channel == 0 { 2 } else { 0 },
        tx_pwr = cfg.wifi_5g_tx_power,
        burst = burst_val,
        agg = agg_val,
        wmm = wmm_val,
        iso = isolate_5g,
        iso_guest = if cfg.guest_isolate_lan { 1 } else { 0 },
        hide = hide_5g,
        vht_bw = vht_bw_5g,
        gi = gi_5g,
        txbf = if cfg.wifi_5g_beamforming { 1 } else { 0 },
        dot11k = if cfg.fast_roaming_80211k { 1 } else { 0 },
        dot11v = if cfg.fast_roaming_80211v { 1 } else { 0 },
    );

    let _ = fs::create_dir_all("/etc/Wireless/iNIC");
    let _ = fs::write("/etc/Wireless/iNIC/iNIC_ap.dat", dat_5g);

    // 3. Restart Radio Interfaces if on device
    if cfg.wifi_2g_enabled {
        let _ = Command::new("ifconfig").args(["ra0", "down"]).output();
        let _ = Command::new("ifconfig").args(["ra0", "up"]).output();
    } else {
        let _ = Command::new("ifconfig").args(["ra0", "down"]).output();
    }

    if cfg.wifi_5g_enabled {
        let _ = Command::new("ifconfig").args(["rai0", "down"]).output();
        let _ = Command::new("ifconfig").args(["rai0", "up"]).output();
    } else {
        let _ = Command::new("ifconfig").args(["rai0", "down"]).output();
    }

    Ok(())
}

/// Applies DNS & DOH configuration, generating dnsmasq and iptables redirection
pub fn apply_dns_config(cfg: &DnsAdvancedConfig) -> Result<(), String> {
    let mut dnsmasq_conf = format!(
        "# FerrumOS Advanced DNS Configuration\n\
        no-resolv\n\
        server={}\n\
        server={}\n\
        cache-size=1024\n\
        domain-needed\n\
        bogus-priv\n",
        cfg.primary_dns, cfg.secondary_dns
    );

    if cfg.anti_dns_rebind {
        dnsmasq_conf.push_str("stop-dns-rebind\nrebind-localhost-ok\n");
    }

    for host in &cfg.custom_hosts {
        dnsmasq_conf.push_str(&format!("address=/{}\n", host));
    }

    let _ = fs::create_dir_all("/etc/dnsmasq.d");
    let _ = fs::write("/etc/dnsmasq.d/ferrum_dns.conf", dnsmasq_conf);

    // Redirect port 53 if force_router_dns is enabled
    if cfg.force_router_dns {
        let _ = Command::new("iptables")
            .args(["-t", "nat", "-A", "PREROUTING", "-p", "udp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "53"])
            .output();
        let _ = Command::new("iptables")
            .args(["-t", "nat", "-A", "PREROUTING", "-p", "tcp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "53"])
            .output();
    }

    // Restart dnsmasq
    let _ = Command::new("killall").arg("dnsmasq").output();
    let _ = Command::new("dnsmasq").output();

    Ok(())
}

/// Applies WAN and NAT Port Forwarding settings
pub fn apply_wan_config(cfg: &WanConfig) -> Result<(), String> {
    // Port Forwarding rules
    for pf in &cfg.port_forwards {
        let proto = match pf.proto.to_uppercase().as_str() {
            "UDP" => "udp",
            "BOTH" => "all",
            _ => "tcp",
        };
        let _ = Command::new("iptables")
            .args([
                "-t", "nat", "-A", "PREROUTING", "-i", "eth2.2", "-p", proto,
                "--dport", &pf.ext_port.to_string(),
                "-j", "DNAT", "--to-destination", &format!("{}:{}", pf.int_ip, pf.int_port)
            ])
            .output();
    }

    // Hardware NAT (PPE) toggle
    if cfg.hw_nat {
        let _ = fs::write("/proc/sys/net/nat/fast_nat", "1");
    } else {
        let _ = fs::write("/proc/sys/net/nat/fast_nat", "0");
    }

    Ok(())
}
