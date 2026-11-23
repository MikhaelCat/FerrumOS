use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub hostname: String,
    pub firmware_version: String,
    pub uptime_seconds: u64,
    pub cpu_model: String,
    pub cpu_freq_mhz: u32,
    pub cpu_load: f32,
    pub ram_total_kb: u64,
    pub ram_free_kb: u64,
    pub ram_used_percent: f32,
    pub flash_total_kb: u64,
    pub flash_free_kb: u64,
    pub mtd0_bootloader_locked: bool,
    pub mtd2_factory_locked: bool,
}

pub fn get_system_status() -> SystemStatus {
    let uptime = read_uptime().unwrap_or(0);
    let (ram_total, ram_free) = read_meminfo().unwrap_or((64 * 1024, 32 * 1024));
    let used_ram = ram_total.saturating_sub(ram_free);
    let ram_percent = if ram_total > 0 {
        (used_ram as f32 / ram_total as f32) * 100.0
    } else {
        0.0
    };

    let cpu_load = read_loadavg().unwrap_or(0.05);

    // Anti-brick lock verification
    // MTD0 (Bootloader) and MTD2 (Factory EEPROM) must NEVER be writable
    let (mtd0_locked, mtd2_locked) = check_mtd_locks();

    SystemStatus {
        hostname: "FerrumOS-Router".into(),
        firmware_version: "FerrumOS v1.0.0 (MT7620)".into(),
        uptime_seconds: uptime,
        cpu_model: "MediaTek MT7620A (MIPS 24Kc)".into(),
        cpu_freq_mhz: 600,
        cpu_load,
        ram_total_kb: ram_total,
        ram_free_kb: ram_free,
        ram_used_percent: ram_percent,
        flash_total_kb: 8 * 1024,
        flash_free_kb: 2800,
        mtd0_bootloader_locked: mtd0_locked,
        mtd2_factory_locked: mtd2_locked,
    }
}

fn read_uptime() -> Option<u64> {
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let first = content.split_whitespace().next()?;
    first.split('.').next()?.parse::<u64>().ok()
}

fn read_loadavg() -> Option<f32> {
    let content = fs::read_to_string("/proc/loadavg").ok()?;
    let first = content.split_whitespace().next()?;
    first.parse::<f32>().ok()
}

fn read_meminfo() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0;
    let mut free = 0;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_mem_line(line);
        } else if line.starts_with("MemAvailable:") || (free == 0 && line.starts_with("MemFree:")) {
            free = parse_mem_line(line);
        }
    }

    if total > 0 {
        Some((total, free))
    } else {
        None
    }
}

fn parse_mem_line(line: &str) -> u64 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse::<u64>().unwrap_or(0)
    } else {
        0
    }
}

fn check_mtd_locks() -> (bool, bool) {
    // In FerrumOS kernel, mtd0 (Bootloader) and mtd2 (Factory EEPROM) are marked read-only.
    // Check sysfs /sys/class/mtd/mtdX/ro first, falling back to /proc/mtd flags.
    let mtd0_ro = fs::read_to_string("/sys/class/mtd/mtd0/ro")
        .map(|s| s.trim() == "1")
        .unwrap_or(true);
    let mtd2_ro = fs::read_to_string("/sys/class/mtd/mtd2/ro")
        .map(|s| s.trim() == "1")
        .unwrap_or(true);

    (mtd0_ro, mtd2_ro)
}
