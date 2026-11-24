# FerrumOS Subsystem and Feature Specification

This document provides a comprehensive inventory of all functional modules and features present in FerrumOS (NAG SNR-CPE-MD1.1), categorized by subsystem with architectural evaluation and implementation status.

---

## Status Classification

- `IMPLEMENTED`: Production-ready, compiled, and verified in FerrumOS v1.0.0.
- `IN-KERNEL`: Native Linux kernel driver or hardware subsystem offload.
- `OPTIONAL`: Modular feature, enabled per deployment requirements.
- `PURGED`: Deprecated or removed from firmware image to enforce storage and memory bounds.

---

## 1. WAN & Internet Connectivity

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **IPoE (DHCP Client)** | BusyBox `udhcpc` + `ferrum-core` | `IMPLEMENTED` | Core Routing | Standard RFC 2131 DHCP client with asynchronous lease tracking. |
| **Static IPv4** | Linux `iproute2` + `ferrum-core` | `IMPLEMENTED` | Core Routing | Manual IP assignment, gateway, subnet mask, and DNS configuration. |
| **PPPoE Client** | `pppd` + `rp-pppoe.so` | `IMPLEMENTED` | Core Routing | Point-to-Point Protocol over Ethernet with automated keepalive. |
| **Dual Access (MAN + WAN)** | Linux routing tables | `IMPLEMENTED` | Routing | Simultaneous access to ISP local intranet routes and internet gateway. |
| **802.1Q VLAN Tagging** | Linux `8021q` module | `IMPLEMENTED` | Switching | Hardware VLAN tagging on WAN (`eth2.2`) for separated Internet/IPTV services. |
| **MTU / MSS Clamping** | `iptables` TCPMSS target | `IMPLEMENTED` | Firewall | Path MTU discovery assistance; clamps MSS to 1452 bytes for PPPoE. |
| **Connectivity Watchdog** | `ferrum-core` background loop | `IMPLEMENTED` | Monitoring | Asynchronous ICMP/DNS probe verifying real gateway reachability. |

---

## 2. LAN & Switch Management

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **LAN Bridge (`br0`)** | Linux `bridge` | `IMPLEMENTED` | Core Switching | Bridges physical switch LAN ports (`eth2.1`) and wireless AP interfaces. |
| **DHCP Server** | `dnsmasq` | `IMPLEMENTED` | Network Service | Lightweight DHCP server with static MAC-to-IP lease binding and WPAD support. |
| **DNS Cache & Forwarder** | `dnsmasq` | `IMPLEMENTED` | Network Service | Low-latency DNS caching proxy with upstream server cycling. |
| **Switch Port Management** | `switch_mtk_esw` | `IN-KERNEL` | Hardware Switch | Port link negotiation (10/100M, full/half duplex) via MT7620 internal PHYs. |
| **Port Isolation / VLANs** | MT7620 Switch Registers | `IMPLEMENTED` | Switching | Hardware port grouping and isolation (e.g. dedicated IPTV set-top box port). |
| **Jumbo Frames** | Switch register tuning | `IMPLEMENTED` | Hardware Switch | Supports frame sizes up to 2048 bytes for intra-LAN forwarding throughput. |
| **Green Ethernet (EEE)** | IEEE 802.3az PHY registers | `IMPLEMENTED` | Power | Dynamic PHY transmitter power down during idle port conditions. |

---

## 3. Wi-Fi 2.4 GHz Subsystem (802.11b/g/n)

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **2.4 GHz AP Driver** | MediaTek `rt2860v2` | `IN-KERNEL` | Wireless MAC | 2T2R MIMO up to 300 Mbps PHY rate; hardware-integrated in MT7620A SoC. |
| **WPA2 / WPA3 Security** | In-driver / `wpa_supplicant` | `IMPLEMENTED` | Wireless Security | WPA2-PSK (AES-CCMP), mixed WPA/WPA2, and 802.1X Enterprise. |
| **Channel & Bandwidth** | Driver ioctl (`iwpriv`) | `IMPLEMENTED` | Radio Config | Channels 1–13, 20/40 MHz auto-coexistence, background channel scanner. |
| **Fast BSS Transition** | IEEE 802.11r FT | `IMPLEMENTED` | Roaming | Fast roaming key distribution enabling client handoff in < 10 milliseconds. |
| **Radio Resource Mgmt** | IEEE 802.11k / 802.11v | `IMPLEMENTED` | Roaming | Neighbor reporting and BSS transition management frames for roaming steering. |
| **Guest Wireless Network** | Virtual BSSID `ra1` | `IMPLEMENTED` | Wireless | Dedicated guest SSID with kernel-level isolation from primary LAN (`br0`). |
| **Client MAC Filtering** | In-driver ACL table | `IMPLEMENTED` | Security | Whitelist / blacklist policy enforcement per wireless radio. |
| **Transmit Power Control** | Driver EEPROM offsets | `IMPLEMENTED` | Radio Config | Transmit power adjustment (10% to 100%) to mitigate co-channel interference. |

---

## 4. Wi-Fi 5 GHz Subsystem (802.11a/n/ac)

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **5 GHz AP Driver** | MediaTek `mt7610` PCIe | `IN-KERNEL` | Wireless MAC | 1T1R 802.11ac up to 433 Mbps PHY rate; connected via internal PCIe bus. |
| **Channel Bandwidth** | 20 / 40 / 80 MHz (VHT80) | `IMPLEMENTED` | Radio Config | High-throughput 80 MHz channel bonding on UNII-1 and UNII-3 bands. |
| **Dynamic Frequency Selection** | In-driver radar detection | `IMPLEMENTED` | Compliance | Radar detection and dynamic channel vacation on UNII-2/2E bands. |
| **Explicit Beamforming** | Hardware TxBF | `IMPLEMENTED` | Radio Config | Directional RF energy focusing toward 802.11ac compliant client devices. |
| **5 GHz Guest Network** | Virtual BSSID `rai1` | `IMPLEMENTED` | Wireless | Isolated 5 GHz guest network with isolated broadcast domain and firewall rules. |

---

## 5. Packet Forwarding, Offloading & Firewall

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **Hardware Flow Offload** | Ralink PPE (`hw_nat.ko`) | `IN-KERNEL` | Acceleration | Hardware flow engine handles established flows at wire speed (0% CPU load). |
| **Software Fast-Path** | Linux Netfilter flowtable | `IMPLEMENTED` | Acceleration | Direct interface-to-interface forwarding bypassing iptables traversal. |
| **Stateful Firewall (SPI)** | Linux Netfilter / `iptables` | `IMPLEMENTED` | Security | Stateful inspection tracking up to 16,384 concurrent connection states. |
| **Port Forwarding (DNAT)** | `iptables -t nat PREROUTING` | `IMPLEMENTED` | Routing | Dynamic and static WAN-to-LAN port redirection. |
| **DMZ (Exposed Host)** | `iptables` 1:1 NAT mapping | `IMPLEMENTED` | Routing | Bidirectional port mapping of all unassigned inbound ports to one LAN host. |
| **UPnP / NAT-PMP** | `miniupnpd` | `IMPLEMENTED` | Core Service | Dynamic port mapping protocol for gaming consoles and workstations. |
| **Flood & DoS Mitigation** | `iptables` rate limits | `IMPLEMENTED` | Security | Token bucket filtering on SYN flood, UDP amplification, and ICMP ping sweeps. |

---

## 6. Routing & Multicast Services

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **Static Routing** | Linux Kernel FIB | `IMPLEMENTED` | Routing | User-defined static IPv4/IPv6 destination route entries. |
| **IGMP Proxy** | `igmpproxy` | `IMPLEMENTED` | IPTV | Relays IGMPv2/v3 multicast membership reports from LAN to WAN. |
| **UDP-to-HTTP Proxy** | `udpxy` | `IMPLEMENTED` | IPTV | Converts raw UDP multicast video streams into unicast HTTP streams. |
| **BGP Dynamic Routing** | BIRD Lite | `IMPLEMENTED` | Routing | BGP peering with Antizapret to synchronize blocked IP routes to FIB table 100. |

---

## 7. Cryptographic Tunnels & Proxy Engines

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **WireGuard VPN** | Linux Kernel Module (`wireguard`) | `IMPLEMENTED` | Tunneling | Modern, high-speed VPN tunnel utilizing ChaCha20-Poly1305 and Curve25519. |
| **Sing-Box Multi-Protocol**| `sing-box` binary | `IMPLEMENTED` | Proxy Client | High-performance proxy engine with in-kernel TPROXY and TUN routing. |
| **VLESS Protocol** | `sing-box` (v1.10+) | `IMPLEMENTED` | Proxy Client | Lightweight, stateless proxy protocol with TLS / XTLS-Reality masking. |
| **Hysteria2 Protocol** | `sing-box` (v1.10+) | `IMPLEMENTED` | Proxy Client | UDP-based transport protocol optimized for high packet loss connections. |
| **TUIC v5 Protocol** | `sing-box` (v1.10+) | `IMPLEMENTED` | Proxy Client | QUIC-based multiplexed transport protocol with low connection latency. |
| **ShadowTLS v3 Protocol** | `sing-box` (v1.10+) | `IMPLEMENTED` | Proxy Client | Masquerades proxy connections behind valid third-party TLS handshakes. |

---

## 8. Anti-Censorship & DPI Mitigation (Zapret)

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **NFQUEUE Packet Desync** | Zapret `nfqws` (v72.13) | `IMPLEMENTED` | DPI Bypass | Zero-copy Netfilter queue interception on Queue 200 with minimal latency. |
| **YouTube 4K Mitigation** | `nfqws` TLS Desynchronization | `IMPLEMENTED` | DPI Bypass | TLS ClientHello splitting (`split-pos=1,midsld`) with `badseq,md5sig` fooling. |
| **Discord Voice Mitigation**| `nfqws` UDP Layer-7 Filter | `IMPLEMENTED` | DPI Bypass | Intercepts UDP ports 50000–65535 applying fake header fooling to voice packets. |
| **QUIC Rejection (UDP 443)**| `iptables` FORWARD REJECT | `IMPLEMENTED` | DPI Bypass | Rejects UDP 443 with ICMP port-unreachable, forcing reliable TCP TLS fallback. |
| **Dynamic Hostlist Sync** | `ferrum-core` in-memory set | `IMPLEMENTED` | DPI Bypass | Automated retrieval and compilation of domain lists into runtime IPSET tables. |

---

## 9. System Administration & Core Daemons

| Feature | Implementation | Status | Subsystem Role | Technical Specification |
| :--- | :--- | :--- | :--- | :--- |
| **Core Daemon** | `ferrum-core` (Rust) | `IMPLEMENTED` | System Control | Asynchronous REST API, WebSocket telemetry, and process supervisor. |
| **Web Interface** | Preact + Tailwind SPA | `IMPLEMENTED` | Management | Zero-lag Single-Page Application embedded into binary with immutable caching. |
| **SSH Daemon** | Dropbear SSH | `IMPLEMENTED` | Security | Hardened SSH server with Ed25519 and RSA public-key authentication. |
| **Time Synchronization** | BusyBox `ntpd` | `IMPLEMENTED` | System Service | Network Time Protocol daemon maintaining synchronization with NTP pools. |
| **Task Scheduling** | BusyBox `crond` | `IMPLEMENTED` | System Service | Lightweight cron daemon for scheduled certificate updates and maintenance. |
| **Hardware Watchdog** | MT7620 WDT (`/dev/watchdog`)| `IMPLEMENTED` | Reliability | 30-second hardware timer pulling CPU reset line on software deadlocks. |
| **WAN Speedtest** | `ferrum-core` network engine | `IMPLEMENTED` | Diagnostics | High-precision microsecond-metered latency and throughput diagnostic engine. |
| **ZRAM Compressed Swap** | Linux kernel ZRAM | `IMPLEMENTED` | Memory | 32 MB compressed swap in RAM using LZO compression, preventing OOM freezes. |
| **Atomic Configuration** | Flash partition `mtd1` | `IMPLEMENTED` | Storage | A/B double-buffered configuration slots with CRC32 integrity checks. |

---

## 10. Purged Legacy Components (Footprint Reduction)

| Purged Component | Upstream Role | Flash Saved | RAM Saved | Elimination Justification |
| :--- | :--- | :--- | :--- | :--- |
| **Samba (`samba36`)** | SMB/CIFS File Sharing | ~1,600 KB | ~12 MB | Target hardware lacks external USB storage ports. |
| **FTP Server (`vsftpd`)** | FTP File Sharing | ~180 KB | ~1.5 MB | Unusable without local disk storage; replaced by SCP over SSH. |
| **Torrent Daemons (`aria2`)**| Download Managers | ~1,200 KB | ~25 MB | Extreme memory footprint exceeding total device RAM capacity. |
| **DLNA Media Server (`minidlna`)**| Media Streaming | ~550 KB | ~6 MB | Redundant; IPTV streams handled efficiently by `udpxy` (~45 KB). |
| **3G/4G Modem Drivers** | USB Cellular Modems | ~320 KB | ~1 MB | Unusable without physical USB interface. |
| **PPTP Server (`accel-pptp`)**| Legacy VPN | ~280 KB | ~2 MB | Cryptographically broken MS-CHAPv2 protocol; superseded by WireGuard. |
| **L2TP Server (`xl2tpd`)** | Legacy VPN | ~310 KB | ~2.5 MB | High overhead and brittle NAT traversal; superseded by WireGuard. |
| **IPsec (`strongswan`)** | Enterprise VPN | ~900 KB | ~6 MB | High complexity and kernel XFRM memory overhead. |
| **Tor Daemon (`tor`)** | Onion Routing | ~2,300 KB | ~20 MB | Memory footprint causes catastrophic OOM on 64 MB devices. |
| **Telnet Daemon (`telnetd`)** | Remote Shell | ~60 KB | ~0.5 MB | Transmits clear-text credentials; permanently replaced by Dropbear SSH. |
| **Print Servers (`p910nd`)** | Network Printing | ~120 KB | ~0.8 MB | Unusable without physical USB printer port. |
| **C HTTPD & ASP Parser** | Legacy Web Server | ~450 KB | ~4 MB | Blocking single-threaded architecture; replaced by `ferrum-core`. |
