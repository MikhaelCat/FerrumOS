use ferrum_core::api::AppState;
use ferrum_core::network::{is_valid_diagnostic_target, sanitize_clean_str, send_wake_on_lan};

#[test]
fn test_command_injection_prevention_in_diagnostics() {
    let malicious_targets = [
        "; rm -rf /",
        "1.1.1.1; cat /etc/shadow",
        "127.0.0.1 && reboot",
        "127.0.0.1 | sh",
        "`reboot`",
        "$(whoami)",
        "1.1.1.1\nreboot",
        "1.1.1.1\rreboot",
        "& echo pwned &",
        "test' OR '1'='1",
        "1.1.1.1; mtd_storage.sh erase",
    ];

    for target in malicious_targets {
        assert!(
            !is_valid_diagnostic_target(target),
            "Malicious target must be rejected: {}",
            target
        );
    }

    let benign_targets = [
        "1.1.1.1",
        "8.8.8.8",
        "ya.ru",
        "google.com",
        "speedtest.net",
        "sub-domain.example.co.uk",
        "my-host-123.org",
    ];

    for target in benign_targets {
        assert!(
            is_valid_diagnostic_target(target),
            "Benign target should be accepted: {}",
            target
        );
    }
}

#[test]
fn test_auth_state_and_token_generation() {
    let state = AppState::new();

    // Default password must be root
    assert_eq!(*state.auth_password.lock().unwrap(), "root");

    // Initially no tokens
    assert_eq!(state.auth_tokens.lock().unwrap().len(), 0);

    // Default proxy is enabled
    assert!(*state.proxy_enabled.lock().unwrap());

    // Default zapret strategy is 2
    assert_eq!(state.zapret_config.lock().unwrap().strategy, 2);
}

#[test]
fn test_sanitization_filters_control_characters() {
    let dirty = "admin\x00\x1b[31m_user\r\n";
    let clean = sanitize_clean_str(dirty, 64);
    assert!(!clean.contains('\0'));
    assert!(!clean.contains('\r'));
    assert!(!clean.contains('\n'));
}

#[test]
fn test_wol_rejects_invalid_mac() {
    let res_invalid = send_wake_on_lan("invalid-mac");
    assert!(res_invalid.is_err(), "Invalid MAC must fail WoL packet dispatch");

    let res_too_short = send_wake_on_lan("AA:BB:CC:DD");
    assert!(res_too_short.is_err());
}
