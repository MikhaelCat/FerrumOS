use serde::{Deserialize, Serialize};
use std::fs;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    pub wifi_2g_ssid: String,
    pub wifi_2g_channel: u8,
    pub wifi_2g_tx_power: u8,
    pub wifi_2g_enabled: bool,
    pub wifi_5g_ssid: String,
    pub wifi_5g_channel: u8,
    pub wifi_5g_bandwidth_mhz: u8,
    pub wifi_5g_enabled: bool,
}

impl Default for WifiConfig {
    fn default() -> Self {
        WifiConfig {
            wifi_2g_ssid: "FerrumOS_2.4G".into(),
            wifi_2g_channel: 6,
            wifi_2g_tx_power: 100,
            wifi_2g_enabled: true,
            wifi_5g_ssid: "FerrumOS_5G".into(),
            wifi_5g_channel: 36,
            wifi_5g_bandwidth_mhz: 80,
            wifi_5g_enabled: true,
        }
    }
}

pub fn get_network_interfaces() -> Vec<NetworkInterface> {
    let mut list = Vec::new();

    // In FerrumOS:
    // eth2.2 -> WAN
    // br0 -> LAN (eth2.1 + ra0 + rai0)
    // ra0 -> 2.4 GHz MT7620
    // rai0 -> 5 GHz MT7610E
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

pub fn apply_wifi_config(cfg: &WifiConfig) -> Result<(), String> {
    // Write 2.4G config
    let dat_2g = format!(
        "# FerrumOS MT7620 2.4G Configuration\n\
        Default\n\
        CountryRegion=1\n\
        CountryRegionABand=7\n\
        CountryCode=RU\n\
        Channel={}\n\
        SSID1={}\n\
        WPAPSK1=12345678\n\
        AuthMode=WPA2PSK\n\
        EncrypType=AES\n\
        TxPower={}\n\
        HT_BW=1\n\
        HT_BSSCoexistence=1\n",
        cfg.wifi_2g_channel,
        cfg.wifi_2g_ssid,
        cfg.wifi_2g_tx_power
    );

    let _ = fs::create_dir_all("/etc/Wireless/RT2860");
    let _ = fs::write("/etc/Wireless/RT2860/RT2860AP.dat", dat_2g);

    // Apply to radio if running on device
    let _ = std::process::Command::new("ifconfig").args(["ra0", "down"]).output();
    let _ = std::process::Command::new("ifconfig").args(["ra0", "up"]).output();

    Ok(())
}
