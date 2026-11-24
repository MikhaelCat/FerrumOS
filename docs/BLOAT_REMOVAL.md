# FerrumOS Storage and Memory Footprint Audit

This document details the quantitative resource recovery achieved by eliminating legacy, insecure, or obsolete packages from the FerrumOS firmware distribution on the NAG SNR-CPE-MD1.1 router (MediaTek MT7620A, 8 MB SPI NOR Flash, 64 MB DDR2 RAM).

---

## 1. Footprint Optimization Summary

| Component Category | Flash Reclaimed | RAM Reclaimed | Operational Rationale |
| :--- | :--- | :--- | :--- |
| **Storage & File Sharing Daemons** | ~3,850 KB (~3.85 MB) | ~16 MB | Target hardware lacks external USB storage ports. |
| **Legacy & Broken VPNs** | ~1,300 KB (~1.30 MB) | ~6 MB | Insecure PPTP/L2TP/StrongSwan replaced by in-kernel WireGuard. |
| **Monolithic C HTTPD & ASP Parser** | ~450 KB (~0.45 MB) | ~4 MB | Blocking architecture replaced by async Rust `ferrum-core`. |
| **Insecure Network Services** | ~300 KB (~0.30 MB) | ~2 MB | Plain-text Telnet and print servers purged; Dropbear SSH retained. |
| **Unused Netfilter ALG Modules** | ~200 KB (~0.20 MB) | ~1.5 MB | Unnecessary packet inspection helpers pruned from kernel. |
| **TOTAL RESOURCE RECOVERY** | **~6.10 MB Flash** | **~29.5 MB RAM** | **Over 65% physical RAM free; flash headroom > 2.4 MB.** |

---

## 2. Subsystem Footprint Audit

### 2.1 Storage and File Sharing Suite (Purged)
The NAG SNR-CPE-MD1.1 router PCB does not have an external USB port soldered to the chassis. Retaining storage daemons consumes valuable flash storage and background memory without operational utility.

- **Samba File Server (`samba36` / `wsdd2`)**:
  - *Flash Reclaimed*: ~1,600 KB.
  - *RAM Reclaimed*: ~8–12 MB per active connection.
  - *Rationale*: Windows SMB file sharing daemon. Obsolete on portless hardware.
- **FTP Daemon (`vsftpd`)**:
  - *Flash Reclaimed*: ~180 KB.
  - *RAM Reclaimed*: ~1.5 MB.
  - *Rationale*: Obsolete without local storage; secure file transfer is provided via SCP/SFTP over Dropbear SSH.
- **BitTorrent & Download Utilities (`transmission`, `aria2`)**:
  - *Flash Reclaimed*: ~1,200 KB.
  - *RAM Reclaimed*: ~25 MB under active download load.
  - *Rationale*: Excessive memory requirements risking catastrophic Out-Of-Memory (OOM) kernel panics.
- **DLNA Media Servers (`minidlna`, `xupnpd`)**:
  - *Flash Reclaimed*: ~550 KB.
  - *RAM Reclaimed*: ~6 MB.
  - *Rationale*: Replaced by lightweight `udpxy` (~45 KB) for multicast IPTV relaying.
- **USB Cellular Modem Drivers (`uqmi`, `usb-modeswitch`, `comgt`)**:
  - *Flash Reclaimed*: ~320 KB.
  - *Rationale*: Hardware lacks physical USB connection for cellular dongles.

---

### 2.2 Deprecated and Insecure VPN Protocols (Purged)

- **PPTP Server (`accel-pptp`)**:
  - *Rationale*: The MS-CHAPv2 authentication protocol contains known cryptographic vulnerabilities susceptible to offline dictionary and hash-cracking attacks.
- **L2TP Daemon (`xl2tpd`, `rp-l2tp`)**:
  - *Rationale*: High protocol overhead and brittle NAT traversal.
- **IPsec IKEv2 (`strongswan`)**:
  - *Flash Reclaimed*: ~900 KB.
  - *RAM Reclaimed*: ~6 MB.
  - *Rationale*: Complex configuration with high kernel XFRM overhead.
- **Tor Proxy (`tor`)**:
  - *Flash Reclaimed*: ~2,300 KB (uncompressed GeoIP database alone exceeds 1 MB).
  - *RAM Reclaimed*: > 20 MB.
  - *Rationale*: Excessive memory consumption incompatible with 64 MB hardware budget.

**Modern Replacement**:
- **WireGuard**: Occupies ~120 KB, executes in kernel space, utilizes modern cryptographic primitives (Curve25519, ChaCha20-Poly1305, BLAKE2s), handles NAT traversal automatically, and operates with less than 2 MB RAM overhead.

---

### 2.3 Legacy Userland Daemons (Purged)

- **Telnet Server (`telnetd`)**:
  - *Rationale*: Transmits administrative credentials in plain text across local networks.
  - *Replacement*: Dropbear SSH with public-key authentication (`authorized_keys`).
- **Network Print Daemons (`p910nd`, `LPRng`)**:
  - *Rationale*: Hardware lacks USB printer connectivity.
- **Legacy C HTTPD & ASP/EJ Engine**:
  - *Rationale*: The legacy C HTTP daemon used single-threaded blocking I/O and parsed `.asp` templates using 128-byte line buffers, causing high CPU latency and page rendering delays.
  - *Replacement*: `ferrum-core` (asynchronous Rust daemon with zero-copy static asset delivery and REST JSON endpoints).

---

### 2.4 Netfilter and Kernel Module Pruning

The following legacy Netfilter Application Layer Gateway (ALG) modules were pruned from the kernel configuration:
- Pruned: `nf_conntrack_tftp`, `nf_conntrack_irc`, `nf_conntrack_h323`, `nf_conntrack_talk`.
- Retained: `nf_conntrack_ftp`, `nf_conntrack_sip`, `nf_conntrack_rtsp`.

*Operational Impact*: Reduces Netfilter hook overhead, decreases kernel memory slab allocation, and lowers per-packet forwarding latency.
