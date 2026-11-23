use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZapretConfig {
    pub enabled: bool,
    pub youtube_fix: bool,
    pub discord_fix: bool,
    pub general_dpi: bool,
    pub custom_nfqws_args: Option<String>,
}

impl Default for ZapretConfig {
    fn default() -> Self {
        ZapretConfig {
            enabled: true,
            youtube_fix: true,
            discord_fix: true,
            general_dpi: true,
            custom_nfqws_args: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZapretStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub youtube_fix_active: bool,
    pub discord_fix_active: bool,
}

pub fn get_zapret_status() -> ZapretStatus {
    // Check if nfqws process is running
    let output = Command::new("pidof")
        .arg("nfqws")
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let pid_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let pid = pid_str.split_whitespace().next().and_then(|p| p.parse::<u32>().ok());
            return ZapretStatus {
                running: true,
                pid,
                youtube_fix_active: true,
                discord_fix_active: true,
            };
        }
    }

    ZapretStatus {
        running: false,
        pid: None,
        youtube_fix_active: false,
        discord_fix_active: false,
    }
}

#[allow(dead_code)]
pub fn build_nfqws_command(config: &ZapretConfig) -> Vec<String> {
    let mut args = vec![
        "--qnum=200".into(),
        "--dpi-desync=split2".into(),
        "--dpi-desync-split-pos=1".into(),
        "--dpi-desync-fooling=badseq".into(),
    ];

    if config.youtube_fix {
        // High-performance YouTube 4K stream fix
        args.extend(vec![
            "--dpi-desync-cutoff=d4".into(),
            "--dpi-desync-repeats=2".into(),
        ]);
    }

    if config.discord_fix {
        // Discord voice and media connection fix
        args.extend(vec![
            "--dpi-desync-any-protocol".into(),
        ]);
    }

    if let Some(custom) = &config.custom_nfqws_args {
        for arg in custom.split_whitespace() {
            args.push(arg.to_string());
        }
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_nfqws_command_defaults() {
        let cfg = ZapretConfig::default();
        let args = build_nfqws_command(&cfg);
        assert!(args.contains(&"--dpi-desync=split2".to_string()));
        assert!(args.contains(&"--dpi-desync-cutoff=d4".to_string()));
        assert!(args.contains(&"--dpi-desync-any-protocol".to_string()));
    }

    #[test]
    fn test_build_nfqws_command_custom_args() {
        let mut cfg = ZapretConfig::default();
        cfg.custom_nfqws_args = Some("--dpi-desync-ttl=5 --wf-l3=ipv4".into());
        let args = build_nfqws_command(&cfg);
        assert!(args.contains(&"--dpi-desync-ttl=5".to_string()));
        assert!(args.contains(&"--wf-l3=ipv4".to_string()));
    }
}
