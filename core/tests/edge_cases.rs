use ferrum_core::parser::{parse_proxy_uri, ProxyType};
use ferrum_core::singbox::{generate_singbox_config, RoutingMode};
use ferrum_core::zapret::{build_nfqws_command, ZapretConfig};
use ferrum_core::network::{is_valid_ipv4, is_valid_cidr, is_valid_mac};

#[test]
fn test_vless_complex_url_encoded_tags() {
    let uri = "vless://a1b2c3d4-e5f6-7890-abcd-ef1234567890@1.2.3.4:443?security=reality&sni=dl.google.com&fp=chrome&pbk=191QzR9mR22s6g4d98X-F5J7_Nn8QfHq9zY18_7k6jM&sid=1a2b&flow=xtls-rprx-vision#%D0%9C%D0%BE%D1%8F%20%D0%A0%D0%B0%D0%BA%D0%B5%D1%82%D0%B0%20%F0%9F%9A%80";
    let res = parse_proxy_uri(uri).expect("Should parse valid complex VLESS");
    assert_eq!(res.proxy_type, ProxyType::Vless);
    assert_eq!(res.tag, "Моя Ракета 🚀");
    assert_eq!(res.server, "1.2.3.4");
    assert_eq!(res.server_port, 443);
    assert_eq!(res.flow.as_deref(), Some("xtls-rprx-vision"));
    assert_eq!(res.fingerprint.as_deref(), Some("chrome"));
}

#[test]
fn test_vless_grpc_reality_convoluted() {
    let uri = "vless://b831381d-6324-4d53-ad4f-8cda48b30811@my-cdn.cloudflare.net:8443?type=grpc&serviceName=my-stealth-grpc&security=reality&pbk=Z3b4-RealityPublicKey-12345&sid=abcd1234&alpn=h2%2Chttp%2F1.1&packetEncoding=xudp&sni=dl.google.com&fp=safari#Convoluted-VLESS-gRPC";
    let res = parse_proxy_uri(uri).expect("Should parse convoluted gRPC VLESS");
    assert_eq!(res.network.as_deref(), Some("grpc"));
    assert_eq!(res.service_name.as_deref(), Some("my-stealth-grpc"));
    assert_eq!(res.fingerprint.as_deref(), Some("safari"));

    let singbox_cfg = generate_singbox_config(&res, RoutingMode::SmartRu, &[], &[]).expect("Singbox gen");
    let json_str = serde_json::to_string(&singbox_cfg).unwrap();
    assert!(json_str.contains("my-stealth-grpc"));
    assert!(json_str.contains("xudp"));
    assert!(json_str.contains("h2"));
}

#[test]
fn test_hysteria2_salamander_obfs_port_hopping() {
    let uri = "hy2://my-super-secret-pass@fast-host.org:8443?sni=speedtest.net&insecure=1&obfs=salamander&obfs-password=mysupersecretobfs&mport=10000-20000&up=100mbps&down=300mbps&alpn=h3#Hysteria2-Extreme";
    let res = parse_proxy_uri(uri).expect("Should parse advanced Hysteria2");
    assert_eq!(res.proxy_type, ProxyType::Hysteria2);
    assert_eq!(res.password.as_deref(), Some("my-super-secret-pass"));
    assert_eq!(res.up_mbps, Some(100));
    assert_eq!(res.down_mbps, Some(300));
    assert!(res.insecure);

    let singbox_cfg = generate_singbox_config(&res, RoutingMode::SmartRu, &[], &[]).expect("Singbox gen");
    let json_str = serde_json::to_string(&singbox_cfg).unwrap();
    assert!(json_str.contains("salamander"));
    assert!(json_str.contains("mysupersecretobfs"));
    assert!(json_str.contains("10000-20000"));
    assert!(json_str.contains("100"));
    assert!(json_str.contains("300"));
}

#[test]
fn test_vless_ipv6_host_support() {
    let uri = "vless://a1b2c3d4-e5f6-7890-abcd-ef1234567890@[2001:db8::1]:8443?security=reality&sni=example.com#IPv6-Node";
    let res = parse_proxy_uri(uri).expect("Should parse IPv6 host in brackets");
    assert_eq!(res.server, "2001:db8::1");
    assert_eq!(res.server_port, 8443);
    assert_eq!(res.tag, "IPv6-Node");
}

#[test]
fn test_vless_boundary_ports() {
    let uri_min = "vless://a1b2c3d4-e5f6-7890-abcd-ef1234567890@1.2.3.4:1?security=none#MinPort";
    let res_min = parse_proxy_uri(uri_min).expect("Port 1 is valid");
    assert_eq!(res_min.server_port, 1);

    let uri_max = "vless://a1b2c3d4-e5f6-7890-abcd-ef1234567890@1.2.3.4:65535?security=none#MaxPort";
    let res_max = parse_proxy_uri(uri_max).expect("Port 65535 is valid");
    assert_eq!(res_max.server_port, 65535);

    let uri_invalid_port = "vless://a1b2c3d4-e5f6-7890-abcd-ef1234567890@1.2.3.4:70000#Overflow";
    assert!(parse_proxy_uri(uri_invalid_port).is_err());
}

#[test]
fn test_hysteria2_special_chars_in_password() {
    let uri = "hy2://p%40ss%3Aw%23rd_123%24%25@my-hy2.vpn.org:443?sni=speed.org#SpecialPass";
    let res = parse_proxy_uri(uri).expect("Should parse encoded password");
    assert_eq!(res.proxy_type, ProxyType::Hysteria2);
    assert_eq!(res.password.as_deref(), Some("p@ss:w#rd_123$%"));
    assert_eq!(res.server, "my-hy2.vpn.org");
    assert_eq!(res.sni.as_deref(), Some("speed.org"));
}

#[test]
fn test_singbox_all_routing_modes() {
    let uri = "vless://a1b2c3d4-e5f6-7890-abcd-ef1234567890@1.2.3.4:443?security=reality&sni=dl.google.com&fp=chrome&pbk=191QzR9mR22s6g4d98X-F5J7_Nn8QfHq9zY18_7k6jM&sid=1a2b&flow=xtls-rprx-vision#TestOutbound";
    let cfg = parse_proxy_uri(uri).expect("Valid URI");

    // Mode 1: Smart RU
    let json_smart = generate_singbox_config(&cfg, RoutingMode::SmartRu, &[], &[]);
    let s_smart = serde_json::to_string(&json_smart).unwrap();
    assert!(s_smart.contains("t.me"));
    assert!(s_smart.contains("discord.com"));
    assert!(s_smart.contains("googlevideo.com"));
    assert!(s_smart.contains("TestOutbound"));

    // Mode 2: Global
    let json_global = generate_singbox_config(&cfg, RoutingMode::Global, &[], &[]);
    let s_global = serde_json::to_string(&json_global).unwrap();
    assert!(s_global.contains("\"final\":\"TestOutbound\"") || s_global.contains("TestOutbound"));

    // Mode 3: Custom
    let custom_domains = vec!["special-corp.org".to_string(), "internal-leak.com".to_string()];
    let custom_ips = vec!["100.64.0.0/10".to_string()];
    let json_custom = generate_singbox_config(&cfg, RoutingMode::Custom, &custom_domains, &custom_ips);
    let s_custom = serde_json::to_string(&json_custom).unwrap();
    assert!(s_custom.contains("special-corp.org"));
    assert!(s_custom.contains("internal-leak.com"));
    assert!(s_custom.contains("100.64.0.0/10"));
}

#[test]
fn test_zapret_all_six_presets() {
    for strat in 1..=5 {
        let cfg = ZapretConfig {
            enabled: true,
            strategy: strat,
            youtube_fix: true,
            discord_fix: true,
            general_dpi: true,
            custom_nfqws_args: None,
        };
        let cmd = build_nfqws_command(&cfg);
        assert!(!cmd.is_empty(), "Strategy {} should generate command arguments", strat);
        assert!(cmd.iter().any(|arg| arg.contains("--dpi-desync")));
    }

    // Strategy 6: Custom
    let custom_cfg = ZapretConfig {
        enabled: true,
        strategy: 6,
        youtube_fix: false,
        discord_fix: false,
        general_dpi: false,
        custom_nfqws_args: Some("--dpi-desync=fake,split2 --dpi-desync-ttl=5".into()),
    };
    let custom_cmd = build_nfqws_command(&custom_cfg);
    assert!(custom_cmd.contains(&"--dpi-desync=fake,split2".into()));
    assert!(custom_cmd.contains(&"--dpi-desync-ttl=5".into()));
}

#[test]
fn test_network_ip_and_cidr_edge_cases() {
    assert!(is_valid_ipv4("0.0.0.0"));
    assert!(is_valid_ipv4("127.0.0.1"));
    assert!(is_valid_ipv4("192.168.1.254"));
    assert!(is_valid_ipv4("255.255.255.255"));

    assert!(!is_valid_ipv4("256.0.0.1"));
    assert!(!is_valid_ipv4("192.168.1"));
    assert!(!is_valid_ipv4("192.168.1.1.1"));
    assert!(!is_valid_ipv4("example.com"));

    assert!(is_valid_cidr("192.168.1.0/24"));
    assert!(is_valid_cidr("0.0.0.0/0"));
    assert!(is_valid_cidr("10.0.0.1/32"));
    assert!(!is_valid_cidr("192.168.1.0/33"));
    assert!(!is_valid_cidr("192.168.1.0/-1"));
    assert!(!is_valid_cidr("192.168.1.0/24/32"));
    assert!(!is_valid_cidr("invalid-ip/24"));
}

#[test]
fn test_mac_filter_edge_cases() {
    assert!(is_valid_mac("00:11:22:33:44:55"));
    assert!(is_valid_mac("aa:bb:cc:dd:ee:ff"));
    assert!(is_valid_mac("AA:BB:CC:DD:EE:FF"));
    assert!(is_valid_mac("AA-BB-CC-DD-EE-FF"));

    assert!(!is_valid_mac("00:11:22:33:44"));
    assert!(!is_valid_mac("00:11:22:33:44:55:66"));
    assert!(!is_valid_mac("GG:11:22:33:44:55"));
    assert!(!is_valid_mac(""));
    assert!(!is_valid_mac(";;;"));
}
