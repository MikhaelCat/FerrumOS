# FerrumOS System Architecture and Subsystem Specification

FerrumOS is an embedded Linux operating system and memory-safe control plane engineered specifically for network routing equipment based on the MediaTek MT7620A System-on-Chip (specifically the NAG SNR-CPE-MD1.1 router).

---

## 1. System Topology Architecture

```
+-----------------------------------------------------------------------------------+
|                        Client & Presentation Layer                                |
|  - Preact / Tailwind Single-Page Application (Embedded Static Assets)            |
|  - Asynchronous JSON REST API Client (/api/v1/*)                                  |
|  - Real-Time WebSocket Telemetry Receiver (/ws/telemetry)                         |
+-----------------------------------------------------------------------------------+
                                         |
                                         | HTTP/1.1 REST & WebSocket
                                         v
+-----------------------------------------------------------------------------------+
|                     FerrumOS Core Daemon (ferrum-core)                            |
|  +-----------------------------------------------------------------------------+  |
|  | Asynchronous HTTP & WebSocket Event Loop                                    |  |
|  +-----------------------------------------------------------------------------+  |
|  | Transactional State Engine & A/B Configuration Manager (CRC32 Checksum)     |  |
|  +-----------------------------------------------------------------------------+  |
|  | Subsystem Supervisor: dnsmasq, dropbear, zapret (nfqws), sing-box, bird     |  |
|  +-----------------------------------------------------------------------------+  |
|  | Network Policy Controller: Flow Offload, 802.11r FT, Guest Isolation, WAN  |  |
|  +-----------------------------------------------------------------------------+  |
|  | Microsecond WAN Speedtest & Diagnostic Engine                               |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
             | Netlink / ioctl / sysfs          | Unix Domain Sockets / Signals
             v                                  v
+-----------------------------------------------------------------------------------+
|                         Linux Kernel Space (3.4.113 MIPS)                         |
|  - Ralink PPE Hardware NAT Acceleration Engine (hw_nat.ko)                        |
|  - Netfilter Packet Classification, NFQUEUE (Queue 200), and IPSET Subsystem      |
|  - Fast-Path Flowtable Offload Engine (FLOWOFFLOAD) with TCP BBR Congestion Control|
|  - Wireless Drivers: MediaTek rt2860v2 (2.4 GHz) and mt7610 (5 GHz PCIe)          |
|  - WireGuard Kernel Module (Cryptographic Encrypted Tunnels)                      |
|  - MTD Partition Driver with Kernel-Enforced Read-Only Write Locks on mtd0 & mtd2 |
|  - In-Memory ZRAM Compressed Swap Pool (50% RAM Allocation via LZO Compression)   |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
|                        Hardware Layer (MediaTek MT7620A)                          |
|  - MIPS 24KEc CPU Core @ 600 MHz (PLL600 Overclocked)                             |
|  - 64 MB DDR2-400 SDRAM                                                           |
|  - 8 MB Winbond W25Q64BV SPI NOR Flash Memory                                     |
|  - Integrated 5-Port 10/100 Fast Ethernet Switch (Ports 0-3 LAN, Port 4 WAN)      |
|  - Hardware Watchdog Timer (WDT) and Active-LOW Reset Pin (GPIO 13)               |
+-----------------------------------------------------------------------------------+
```

---

## 2. Core Engineering Principles

### 2.1 Hardware-Enforced Anti-Brick Architecture
- **Immutable Critical Partitions**: Partition `mtd0` (Bootloader, 192 KB) and partition `mtd2` (Factory EEPROM, 64 KB) are registered with the `MTD_CAP_ROM` attribute in the kernel driver. Any write operation initiated by userland or scripts returns `EPERM` at the VFS layer, preventing erasure of hardware calibration tables and bootloaders.
- **Transactional A/B Configuration Storage**: The persistent configuration partition `mtd1` (64 KB) is divided into two 32 KB blocks. Updates are written to the inactive slot with a generation counter and CRC32 checksum. The active pointer is updated only after successful verification.
- **Physical Reset Recovery Mode**: Holding the physical Reset button (GPIO 13) for 3 seconds during early boot initiates an emergency in-memory recovery mode, assigning `192.168.1.1/24` to `br0` and serving an emergency recovery web endpoint.
- **Hardware Watchdog Guard**: The MT7620 hardware watchdog timer automatically reboots the router into safe-mode if `ferrum-core` fails to service `/dev/watchdog` within 30 seconds.

### 2.2 Memory & Storage Footprint Optimization
- **Elimination of Unused Daemons**: Purges legacy storage services (Samba, FTP, DLNA, BitTorrent) that are obsolete on hardware without physical USB ports, recovering approximately 6.1 MB of flash and 29.5 MB of RAM.
- **Single Static Daemon Architecture**: Consolidates management interfaces, APIs, process supervision, and configuration management into a single stripped and UPX-compressed Rust executable (`< 500 KB`).
- **RAM Headroom**: Maintains over 65% of physical memory (approximately 42 MB) free for socket buffers (`sk_buff`), connection tracking (`nf_conntrack`), and network caches.

### 2.3 Wire-Speed Packet Offloading
- **Packet Processing Engine (PPE HWNAT)**: Established TCP and UDP sessions are offloaded directly to the hardware forwarding engine, achieving 100 Mbps line-rate bidirectional throughput at negligible CPU load.
- **Software Fast-Path Integration**: Packets bypassing hardware offloading utilize Linux flowtable fast-path shortcuts, preventing redundant Netfilter chain traversals.

### 2.4 DPI Mitigation & Anti-Censorship
- **NFQUEUE Packet Desynchronization**: Integrated Zapret `nfqws` processes Netfilter queue 200 in zero-copy mode, performing TLS ClientHello splitting (`split-pos=1,midsld`) and TCP sequence fooling.
- **Multi-Protocol Proxy Engine**: Sing-Box handles advanced proxy protocols (VLESS, Hysteria2, TUIC v5, ShadowTLS v3) via kernel TPROXY and TUN interfaces.
- **Dynamic Routing**: Peering with Antizapret via BIRD Lite dynamically inserts blocked prefix routes into FIB table 100.

---

## 3. Flash & Memory Allocation Budgets

### 3.1 Flash Memory Allocation (8 MB Winbond W25Q64BV SPI NOR)

| Partition | Identifier | Offset Range | Allocated Size | Filesystem / Format | Access Policy |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Bootloader | `mtd0` | `0x00000000 - 0x00030000` | 192 KB | U-Boot Binary | **Read-Only (`MTD_CAP_ROM`)** |
| Config | `mtd1` | `0x00030000 - 0x00040000` | 64 KB | A/B Double-Buffer (CRC32) | Read / Write |
| Factory | `mtd2` | `0x00040000 - 0x00050000` | 64 KB | RF Calibration EEPROM | **Read-Only (`MTD_CAP_ROM`)** |
| Kernel | `mtd3` | `0x00050000 - Dynamic` | ~1.0 MB | Linux Kernel (`zImage.lzma`) | Read-Only (Firmware Upgrade)|
| RootFS | `mtd4` | `Dynamic - 0x007E0000` | ~4.6 MB | SquashFS 4.0 (XZ) | Read-Only (Firmware Upgrade)|
| Storage | `mtd5` | `0x007E0000 - 0x00800000` | 128 KB | Gzip Archive (`/etc/storage`)| Read / Write |

Maximum firmware image budget (`mtd3` + `mtd4`): 8,060,928 bytes. The compiled FerrumOS release image is approximately 5.61 MB, leaving 2.45 MB of flash headroom.

### 3.2 Physical RAM Allocation (64 MB DDR2-400)

```
+-----------------------------------------------------------------------------------+
|                        Physical RAM: 64 MB (65,536 KB)                            |
+------------------------+-------------------------+--------------------------------+
| Kernel & Network Core  | Userland Daemons        | Available Memory Headroom      |
| ~12 MB                 | ~10 MB                  | ~42 MB (> 65% Free)            |
+------------------------+-------------------------+--------------------------------+
| - Linux Kernel Image   | - ferrum-core (~2.0 MB) | - sk_buff Socket Buffers       |
| - Netfilter Conntrack  | - dnsmasq (~1.5 MB)     | - Page Cache Buffers           |
| - Wi-Fi Ring Buffers   | - dropbear (~1.0 MB)    | - ZRAM Compressed Swap Pool    |
| - PPE HWNAT Tables     | - zapret / sing-box     | - Transient Stack & Heap Space |
+------------------------+-------------------------+--------------------------------+
```

---

## 4. Software Subsystems Detail

### 4.1 ferrum-core Architecture
The core daemon is structured into modular Rust components:
- `main.rs`: Process entry point, signal handlers (SIGTERM, SIGHUP), and runtime argument parsing.
- `api.rs`: Asynchronous REST router handling API endpoints, query normalization, and error responses.
- `web.rs`: Embedded asset server delivering gzipped Preact SPA bundles with immutable caching headers.
- `state.rs`: Transactional state storage managing active device configuration, schema validation, and A/B flash writes.
- `config.rs`: Strongly-typed configuration structs with `serde` serialization.
- `network.rs`: Hardware offload controller, 802.11r FT, guest isolation rules, and microsecond-metered speedtest engine.
- `dpi.rs`: Zapret `nfqws` supervisor managing process lifecycle, queue attachment, and strategy parameter generation.
- `singbox.rs`: Sing-Box configuration generator supporting VLESS, Hysteria2, TUIC v5, and ShadowTLS v3.
- `parser.rs`: Protocol URI parser converting connection strings into validated configuration structures.

### 4.2 Network Offload Hierarchy
1. **Hardware Flow Engine (PPE HWNAT)**: First packet of each connection traverses Netfilter. Upon completing the three-way handshake, the session is registered in the PPE table. All subsequent packets are processed directly in hardware silicon.
2. **Software Flowtable (Fast-Path)**: Non-offloaded flows utilize Linux kernel flowtable shortcuts, bypassing standard iptables filtering chains after initial state establishment.
3. **TCP Congestion Control**: Configures TCP BBR algorithm for active router TCP terminations, maximizing throughput under packet loss conditions.
