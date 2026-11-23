# Padavan-Next: Complete Feature Matrix & Inventory

This document provides a comprehensive inventory of all functional modules and features present in the current Padavan firmware (NAG SNR-CPE-MD1.1), categorized by subsystem with architectural evaluation and rewrite recommendations.

---

## Evaluation Legend

- **Status / Value**:
  - `CORE`: Mandatory for basic routing and network operation.
  - `ESSENTIAL`: Critical feature expected by modern users.
  - `OPTIONAL`: Useful feature, can be modular or disabled for space.
  - `LEGACY`: Outdated mechanism, candidate for removal or replacement.
- **Rewrite Target**:
  - `Rust`: Reimplement as an asynchronous, memory-safe daemon or tool.
  - `Modern C`: Modernize existing C code (refactor, clean up, remove blocking calls).
  - `Keep`: Keep upstream implementation (e.g. standard kernel module or trusted tool).
  - `Drop`: Deprecate or remove to save flash/RAM.

---

## 1. WAN & Internet Connection

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **IPoE (DHCP Client)** | BusyBox `udhcpc` + `rc` | `CORE` | `Rust` / `Keep` | Standard WAN DHCP client. Reliable, but script hook can be replaced with native IPC. |
| **Static IP** | Kernel `iproute2` + `rc` | `CORE` | `Rust` | Manual IP configuration, gateway, and DNS assignment. |
| **PPPoE Client** | `pppd` + `rp-pppoe.so` | `CORE` | `Keep` | Standard DSL/Fiber PPPoE client. Stable C daemon, keep with optimized configuration. |
| **PPTP / L2TP WAN** | `xl2tpd` / `accel-pptp` | `OPTIONAL` | `Modern C` | Legacy ISP connection modes (e.g. Beeline Russia). Needed only for specific legacy providers. |
| **Dual Access (MAN + WAN)** | Custom `rc` routing | `ESSENTIAL` | `Rust` | Simultaneous access to ISP local subnet and VPN WAN. |
| **VLAN 802.1Q on WAN** | Linux 8021q module | `ESSENTIAL` | `Keep` | Tagged WAN/Internet/IPTV/VoIP interface separation. |
| **MTU / MSS Clamping** | `iptables` TCPMSS target | `CORE` | `Keep` | Clamps MSS for VPN and PPPoE to prevent packet fragmentation. |
| **Internet Connectivity Check** | `detect_internet` script | `ESSENTIAL` | `Rust` | Background ping/DNS probe to verify real internet reachability. Needs async rewrite. |

---

## 2. LAN & Switch Management

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **LAN Subnet & Bridge** | Linux `bridge` + `brctl` | `CORE` | `Keep` | Bridges Ethernet LAN ports and Wi-Fi interfaces into `br0`. |
| **DHCP Server** | `dnsmasq` | `CORE` | `Keep` | Ultra-fast, lightweight DHCP server with static lease binding and WPAD/PXE options. |
| **DNS Forwarder & Cache** | `dnsmasq` | `CORE` | `Keep` | Local caching DNS server with upstream forwarding and custom hosts. |
| **Switch Port Configuration** | `mii_mgr` + `switch` | `CORE` | `Rust` / `Modern C` | 10/100M speed, duplex, auto-negotiation, and link status monitoring via SoC registers. |
| **Port Isolation / VLANs** | MT7620 internal switch | `ESSENTIAL` | `Modern C` | Hardware VLAN segmentation of switch ports (e.g. dedicated IPTV port). |
| **Jumbo Frames** | Switch register tuning | `OPTIONAL` | `Keep` | Up to 2048-byte frames for LAN throughput optimization. |
| **Green Ethernet (EEE 802.3az)** | Switch PHY registers | `OPTIONAL` | `Keep` | Energy-efficient Ethernet power saving when link is idle. |

---

## 3. Wi-Fi 2.4 GHz (802.11b/g/n)

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **2.4 GHz AP Driver** | Ralink `rt2860v2` (v2.7) | `CORE` | `Keep` | Proprietary SoC Wi-Fi driver. 2T2R up to 300 Mbps. Hardware-tied, must keep. |
| **WPA2 / WPA3 Security** | In-driver / `wpa_supplicant` | `CORE` | `Keep` | WPA-PSK, WPA2-PSK (AES/TKIP), 802.1x Enterprise. |
| **Channel & Bandwidth** | Driver ioctl (`iwpriv`) | `CORE` | `Rust` (API) | Channels 1–13, 20/40 MHz auto coexistence, Auto channel selection. |
| **WMM & QoS** | Driver WMM queues | `ESSENTIAL` | `Keep` | Multimedia prioritization and Power Save mode. |
| **Guest Wi-Fi AP** | Virtual BSSID (`ra1`) | `ESSENTIAL` | `Rust` (API) | Isolated guest network with VLAN tag and separate subnet. |
| **WDS / Repeater Mode** | Driver WDS / AP-Client | `OPTIONAL` | `Keep` | Wireless bridging between routers. |
| **MAC ACL Filtering** | Driver MAC table | `OPTIONAL` | `Rust` (API) | Allow/Deny list of client MAC addresses. |
| **TX Power & Green AP** | Driver power table | `OPTIONAL` | `Rust` (API) | Dynamic transmit power control to reduce RF interference. |

---

## 4. Wi-Fi 5 GHz (802.11a/n/ac)

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **5 GHz AP Driver** | MediaTek `mt7610` PCIe | `CORE` | `Keep` | 1T1R 802.11ac driver up to 433 Mbps. Proprietary PCIe driver. |
| **Channel Bandwidth** | 20 / 40 / 80 MHz | `CORE` | `Keep` | VHT80 support for maximum 802.11ac throughput. |
| **DFS Support** | In-driver radar detection | `OPTIONAL` | `Keep` | Radar detection on UNII-2/UNII-2 Extended channels. |
| **Transmit Beamforming (TxBF)** | Driver explicit BF | `ESSENTIAL` | `Keep` | Directs RF energy toward 802.11ac client devices. |
| **Guest AP (5 GHz)** | Virtual BSSID (`rai1`) | `ESSENTIAL` | `Rust` (API) | Isolated 5 GHz guest network. |

---

## 5. Firewall, NAT & Security

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **Hardware NAT (HWNAT / PPE)** | `hw_nat` kernel driver | `CORE` | `Keep` | MediaTek Packet Processing Engine flow offload. Zero CPU routing. |
| **Stateful Packet Inspection (SPI)** | `iptables` / `nf_conntrack` | `CORE` | `Keep` | Standard Linux stateful firewall. |
| **NAT Traversal (SIP, FTP, RTSP)** | Netfilter ALG helpers | `OPTIONAL` | `Modernize` | Application-level gateways for legacy VoIP and FTP protocols. |
| **Port Forwarding (Virtual Server)** | `iptables -t nat PREROUTING` | `CORE` | `Rust` (API) | Redirect external ports to internal LAN IPs. |
| **DMZ / Exposed Host** | `iptables` 1:1 NAT | `ESSENTIAL` | `Rust` (API) | Expose a single internal IP to all incoming WAN traffic. |
| **UPnP & NAT-PMP** | `miniupnpd` | `CORE` | `Keep` | Automatic dynamic port forwarding for consoles and game servers. |
| **DoS & Flood Protection** | `iptables` rate limits | `ESSENTIAL` | `Rust` | SYN flood, UDP flood, and ICMP ping of death mitigation. |
| **URL & Content Filter** | `iptables` string match | `LEGACY` | `Rust` / `Drop` | Ineffective on HTTPS. Candidate for replacement with DNS-based blocking. |
| **IP / Port Outbound Filter** | `iptables` FORWARD rules | `OPTIONAL` | `Rust` (API) | Parental control / device access scheduling. |

---

## 6. Routing & Multicast / IPTV

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **Static Routes** | Kernel routing table | `CORE` | `Rust` (API) | User-defined static IPv4/IPv6 routes. |
| **IGMP Proxy** | `igmpproxy` | `ESSENTIAL` | `Modern C` | Relays multicast IPTV streams from WAN to LAN. |
| **UDPxy (UDP to HTTP)** | `udpxy` | `ESSENTIAL` | `Keep` | Converts UDP multicast IPTV streams to HTTP unicast for smart TVs and PCs. |
| **xUPNPd** | `xupnpd` | `OPTIONAL` | `Drop` | Heavy DLNA media server for IPTV playlists. Removed in modern builds to save flash. |

---

## 7. VPN & Remote Access

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **WireGuard** | `wireguard` kernel module | `ESSENTIAL` | `Keep` / `Rust` | Modern, high-speed, low-memory encrypted VPN tunnel. |
| **OpenVPN Client / Server** | `openvpn` | `OPTIONAL` | `Keep` | Heavy OpenSSL dependency (~1.6 MB). Useful for corporate tunnels. |
| **PPTP / L2TP Server** | `accel-pptpd` / `xl2tpd` | `LEGACY` | `Drop` | Insecure legacy VPN protocols. Deprecated in favor of WireGuard. |
| **IPsec (StrongSwan)** | `strongswan` | `OPTIONAL` | `Drop` | Heavy enterprise VPN. Rarely needed on 8MB flash home routers. |
| **Tor Proxy** | `tor` | `OPTIONAL` | `Drop` | Too heavy (~2.3 MB) for 8MB flash and 64MB RAM. |

---

## 8. DPI Bypass & Anti-Censorship (Zapret)

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **Zero-Copy NFQUEUE Desync** | `nfqws` (v72.13) | `ESSENTIAL` | `Rust` (Supervisor)| Wire-speed TCP/UDP packet desynchronization with no memory copying. |
| **Transparent Proxy Mode** | `tpws` (v72.13) | `OPTIONAL` | `Keep` | Fallback transparent TCP proxy on port 9999 for non-NFQ environments. |
| **YouTube 4K & YT Music Bypass** | TLS SNI split + fake fooling | `ESSENTIAL` | `Rust` (Config) | Bypasses ISP DPI throttling on YouTube video servers without VPN speed penalty. |
| **Discord Voice Bypass** | UDP L7 `discord,stun` filter | `ESSENTIAL` | `Rust` (Config) | Bypasses blocking of Discord voice servers on UDP ports 50000–65535. |
| **QUIC Blocking (UDP 443)** | `iptables` FORWARD reject | `ESSENTIAL` | `Keep` | Forces browsers from HTTP/3 to TLS TCP where DPI bypass is 100% reliable. |
| **Dynamic Hostlist Management** | Shell scripts / text files | `ESSENTIAL` | `Rust` | Web-managed lists of blocked domains with auto-download from trusted feeds. |

---

## 9. System Administration & Core Services

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **Web UI HTTP Server** | `httpd` (C / ASP / EJ) | `CORE` | **`Rust` (next-httpd)** | Prime rewrite candidate. Replace single-threaded C server with fast async REST daemon. |
| **Web UI Frontend** | Bootstrap 2.3 + ASP + jQuery | `CORE` | **Modern SPA** | Replace heavy server-rendered ASP with lightweight reactive SPA (Preact/Vue/Tailwind). |
| **SSH Server** | `dropbear` | `CORE` | `Keep` | Lightweight MIPS SSH server. Key-based authentication and SCP support. |
| **Telnet Server** | BusyBox `telnetd` | `LEGACY` | `Drop` | Insecure plain-text shell. Keep disabled by default, prioritize SSH. |
| **NTP Client** | BusyBox `ntpd` | `CORE` | `Keep` | Time synchronization with pool.ntp.org. |
| **Cron Scheduler** | BusyBox `crond` | `CORE` | `Keep` | Periodic scheduled tasks (log rotate, DDNS updates, reboots). |
| **Hardware Watchdog** | MT7620 WDT registers | `CORE` | `Keep` | Hardware timer that reboots the router in case of kernel panic or freeze. |
| **Dynamic DNS (DDNS)** | `inadyn` | `ESSENTIAL` | `Rust` / `Keep` | Updates public IP on No-IP, DynDNS, Cloudflare, DuckDNS, etc. |
| **Configuration Backup / Restore** | NVRAM export / import | `CORE` | `Rust` (API) | Save router settings to file and restore with validation. |
| **Firmware Upgrade** | `upgrade.cgi` + `mtd_write` | `CORE` | `Rust` (API) | In-browser firmware flashing with checksum validation and safe reboot. |
| **ZRAM Compressed Swap** | Linux kernel ZRAM | `ESSENTIAL` | `Keep` | Doubles effective RAM using 50% LZO compressed in-memory swap. |

---

## 10. USB & Storage Services (Optional / Legacy)

*Note: The SNR-CPE-MD1.1 board does not have an external USB port soldered by default, but the MT7620A SoC has USB pins.*

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **Samba File Sharing** | `samba3` / `samba36` | `OPTIONAL` | `Drop` | Unnecessary on devices without USB storage; consumes ~1.5 MB flash. |
| **FTP Server** | `vsftpd` | `OPTIONAL` | `Drop` | Unnecessary without storage. |
| **Download Daemons** | `aria2` / `transmission` | `OPTIONAL` | `Drop` | Unnecessary without storage. |
| **3G / 4G Modem Support** | `uqmi` / `usb-modeswitch` | `OPTIONAL` | `Drop` | Disabled on SNR-MD1 to save flash. |

---

## 11. Diagnostics & Real-Time Monitoring

| Feature | Current Implementation | Value | Rewrite Target | Description / Rationale |
| :--- | :--- | :--- | :--- | :--- |
| **Real-time CPU & RAM Usage** | `/proc/stat` + AJAX poll | `CORE` | **`Rust` (WebSocket)** | Real-time system load graphs streamed over WebSocket without HTTP polling overhead. |
| **Network Traffic Graphs** | `/proc/net/dev` polling | `CORE` | **`Rust` (WebSocket)** | Live per-interface RX/TX bandwidth meters (WAN, LAN, 2.4G, 5G). |
| **Active Connection Tracker** | `/proc/net/nf_conntrack` | `ESSENTIAL` | `Rust` (API) | Inspect active NAT connections, IP addresses, ports, and states. |
| **ARP & Client Device Table** | `/proc/net/arp` + DHCP leases | `CORE` | `Rust` (API) | List of all connected devices with hostnames, IPs, MACs, and signal strengths. |
| **Wi-Fi Site Survey** | `iwpriv` scan results | `ESSENTIAL` | `Rust` (API) | Scan neighboring Wi-Fi networks to identify least congested channels. |
| **System Log Viewer** | `syslogd` + `logread` | `CORE` | `Rust` (API) | Real-time filterable log viewer with level and keyword filtering. |
| **Ping & Traceroute Tools** | BusyBox utilities | `ESSENTIAL` | `Rust` (API) | Network diagnostic utilities runnable directly from the web interface. |
