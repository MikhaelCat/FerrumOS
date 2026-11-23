# FerrumOS: Bloatware Removal & Resource Recovery Audit

A central goal of **FerrumOS** is ruthlessly eliminating legacy, insecure, or useless software packages that were inherited from 2012-era router firmware.

On a device with **8 MB SPI Flash** and **64 MB RAM** without external USB ports (like the NAG SNR-CPE-MD1.1), carrying file servers and legacy tunneling protocols is deadweight.

---

## 1. Summary of Recovered Resources

| Category | Flash Memory Saved | RAM Footprint Saved | Security & Stability Benefit |
| :--- | :--- | :--- | :--- |
| **USB & File Sharing Daemons** | ~3,850 KB (~3.85 MB) | ~16 MB | Eliminates unneeded disk daemons on a portless board. |
| **Legacy & Broken VPNs** | ~1,300 KB (~1.30 MB) | ~6 MB | Drops insecure PPTP/L2TP/StrongSwan in favor of WireGuard. |
| **Monolithic C HTTPD & ASP Parser** | ~450 KB (~0.45 MB) | ~4 MB | Eliminates blocking synchronous macro execution & Web UI lag. |
| **Insecure / Obsolete Network Services**| ~300 KB (~0.30 MB) | ~2 MB | Eliminates plain-text Telnet and print servers. |
| **Unused Netfilter / ALG Kernel Modules**| ~200 KB (~0.20 MB) | ~1.5 MB | Drops legacy packet inspection helpers. |
| **TOTAL RECOVERED** | **~6.1 MB Flash** | **~29.5 MB RAM** | **Massive free headroom, zero OOM risk.** |

---

## 2. Detailed Audit of Eliminated Components

### 2.1 USB & File Sharing Suite (PURGED)
*The SNR-CPE-MD1.1 has no external USB port. Bundling storage daemons is 100% wasted space.*

- ❌ **Samba (`samba3` / `samba36` + `wsdd2`)**:
  - *Flash Waste*: ~1,600 KB.
  - *RAM Waste*: ~8–12 MB per active connection.
  - *Rationale*: Windows SMB file sharing server. Pointless without USB flash drives.
- ❌ **FTP Server (`vsftpd`)**:
  - *Flash Waste*: ~180 KB.
  - *Rationale*: Obsolete without local storage; replaced by secure SCP/SFTP over SSH if needed.
- ❌ **Torrent & Download Daemons (`transmission`, `aria2`)**:
  - *Flash Waste*: ~1,200 KB.
  - *RAM Waste*: Extreme (up to 25 MB RAM during active seeding).
- ❌ **Media Streaming (`minidlna`, `xupnpd`)**:
  - *Flash Waste*: ~550 KB.
  - *Rationale*: Replaced by ultra-lightweight `udpxy` (~45 KB) for IPTV streams.
- ❌ **3G/4G USB Modem Drivers (`uqmi`, `usb-modeswitch`, `comgt`)**:
  - *Flash Waste*: ~320 KB.

---

### 2.2 Deprecated & Insecure VPN Protocols (PURGED)
- ❌ **PPTP Server (`accel-pptpd`)**:
  - *Rationale*: MS-CHAPv2 authentication has been cryptographically broken for over a decade. Vulnerable to near-instant offline cracking.
- ❌ **L2TP Server (`xl2tpd`, `rp-l2tp`)**:
  - *Rationale*: High overhead, double-encapsulation, difficult NAT traversal.
- ❌ **IPsec IKEv2 (`strongswan`)**:
  - *Flash Waste*: ~900 KB.
  - *RAM Waste*: High. Complex configuration with high kernel XFRM overhead.
- ❌ **Tor Proxy (`tor`)**:
  - *Flash Waste*: ~2,300 KB (uncompressed GeoIP database alone is > 1 MB).
  - *RAM Waste*: > 20 MB. Impossible to run reliably on 64 MB RAM without killing routing.

✅ **The Modern Replacement**:
- **WireGuard**: Only **~120 KB**, runs directly inside the Linux kernel, uses state-of-the-art cryptography (Curve25519, ChaCha20-Poly1305, BLAKE2s), handles NAT seamlessly, and achieves wire-speed 100 Mbps throughput with `< 2 MB` RAM.

---

### 2.3 Legacy Userland Daemons (PURGED)
- ❌ **Telnet Server (`telnetd`)**:
  - Transmits passwords in plain text across the local network.
  - *Replacement*: Use **Dropbear SSH** with public key authentication (`~/.ssh/authorized_keys`).
- ❌ **Network Print Servers (`p910nd`, `LPRng`)**:
  - *Rationale*: No USB printer port exists on the hardware.
- ❌ **Monolithic C HTTPD & ASP/EJ Engine**:
  - *Rationale*: The single-threaded C server blocks on external commands, parses `.asp` files byte-by-byte using 128-byte buffers, and generates hundreds of tiny system calls per page request.
  - *Replacement*: **`ferrum-httpd`** (Rust-based async server with zero-copy static asset serving and JSON APIs).

---

### 2.4 Kernel & Netfilter Pruning
- ❌ **Unneeded Netfilter Application Layer Gateways (ALGs)**:
  - Removed: `nf_conntrack_tftp`, `nf_conntrack_irc`, `nf_conntrack_h323`, `nf_conntrack_talk`.
  - Retained: `nf_conntrack_ftp`, `nf_conntrack_sip`, `nf_conntrack_rtsp`.
  - *Benefit*: Streamlined packet classification, smaller kernel memory footprint, and lower per-packet latency.
