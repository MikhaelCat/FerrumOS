# Padavan-Next: Modernization & Rewrite Roadmap

This roadmap outlines the systematic, phased transformation of the legacy Padavan monolithic architecture into **Padavan-Next** — a modern, memory-safe, modular, and lightning-fast router operating system.

---

## 🎯 Architectural Principles

1. **Safety First**: Eliminate C memory vulnerabilities (buffer overflows, format string bugs, dangling pointers) by authoring new userland services in **Rust** targeting `mipsel-unknown-linux-musl` or `mipsel-unknown-linux-uclibc`.
2. **API-First Design**: Completely decouple backend system state from the presentation layer. Every setting, action, and telemetry metric must be accessible via a strongly-typed REST and WebSocket API.
3. **Zero-Lag Presentation**: Single-page application (SPA) architecture. The browser downloads a lightweight, pre-compressed bundle once and caches it permanently. No full-page reloads, no synchronous server-side ASP macro parsing.
4. **Hardware Maximization**: Keep the proven, high-performance Linux 3.4 kernel and Ralink HWNAT PPE flow acceleration in place while shedding legacy userland bloat.

---

## 🗺️ Phased Roadmap

```mermaid
flowchart LR
    P1["Phase 1: API & Modern UI"] --> P2["Phase 2: Core State Engine"]
    P2 --> P3["Phase 3: DPI & Smart Routing"]
    P3 --> P4["Phase 4: Kernel & Driver Polish"]
```

---

### Phase 1: High-Performance Async API & Modern SPA WebUI
*Target: Replace legacy `httpd` ASP macro server with a sub-millisecond async REST daemon and sleek SPA.*

- [ ] **1.1. `next-httpd` Micro-Daemon**:
  - Implement a lightweight HTTP/1.1 & WebSocket daemon in **Rust** (using `hyper` / `tiny_http` / custom `epoll`).
  - Binary footprint target: `< 1.2 MB` stripped, `< 2.5 MB` RAM usage.
  - Built-in static asset compression (Gzip / Brotli) with immutable caching headers (`ETag`, `Cache-Control: public, max-age=31536000`).
- [ ] **1.2. RESTful Configuration & Telemetry API (`/api/v1`)**:
  - `GET /api/v1/system/status`: Real-time CPU, RAM, uptime, and load.
  - `GET /api/v1/network/interfaces`: Interface status, IP, MAC, link state.
  - `GET /api/v1/wifi/clients`: Connected 2.4G and 5G client devices with RSSI.
  - `POST /api/v1/wifi/settings`: Update SSID, WPA2 key, channel, tx power.
  - `POST /api/v1/system/reboot` & `POST /api/v1/system/upgrade`.
- [ ] **1.3. WebSocket Live Streaming (`/ws/telemetry`)**:
  - Push real-time bandwidth metrics (`/proc/net/dev`) at 1 Hz directly to connected clients without HTTP polling overhead.
- [ ] **1.4. Modern SPA Frontend**:
  - Build with **Preact** / **Vue 3** + **Tailwind CSS**.
  - Production build compiled into a single static bundle (`index.html`, `app.js`, `app.css`) under **~150 KB gzipped**.
  - Sleek dark mode dashboard with zero layout shift and instant tab switching.

---

### Phase 2: Typed State Engine & Process Supervisor
*Target: Replace untyped NVRAM and fragile shell scripts with a typed, transactional state manager.*

- [ ] **2.1. `next-stated` (Configuration Daemon)**:
  - Strongly typed configuration schema (Rust structs with `serde`).
  - Strict input validation before committing (prevent invalid IP ranges, illegal Wi-Fi channels, or malformed netmasks).
  - Transactional persistence to Flash partition `mtd1` with CRC32 integrity verification.
- [ ] **2.2. Event-Driven Subsystem Notifications**:
  - Replace coarse `restart_services` shell calls with targeted IPC events.
  - Changing Wi-Fi passphrase reconfigures the Wi-Fi AP via driver ioctl without resetting WAN or tearing down active TCP connections.
- [ ] **2.3. Clean Init & Process Supervision**:
  - Lightweight supervisor that monitors critical daemons (`dnsmasq`, `dropbear`, `zapret`) and restarts them automatically if crashed.

---

### Phase 3: DPI Bypass & Smart Traffic Management
*Target: Turn Zapret anti-censorship and traffic shaping into a first-class, web-managed router subsystem.*

- [ ] **3.1. `next-zapret` Native Integration**:
  - Direct management of `nfqws` processes and NFQUEUE chains.
  - Web UI strategy editor: enable/disable YouTube 4K desync, Discord Voice UDP bypass, or QUIC drop with one click.
- [ ] **3.2. Dynamic Hostlist Synchronizer**:
  - Scheduled background updates of blocked domain lists (e.g., from community lists or user custom lists).
  - Memory-efficient binary search lookup in RAM for millions of domains.
- [ ] **3.3. Policy-Based Routing (PBR)**:
  - Route specific domains, IP subnets, or client devices (smart TVs, consoles) through VPN or direct WAN with DPI desync.

---

### Phase 4: Kernel, Toolchain & CI/CD Modernization
*Target: Automated builds, reproducible images, and kernel footprint optimization.*

- [ ] **4.1. Modern Cross-Compilation Pipeline**:
  - Containerized Rust + GCC cross-toolchain targeting `mipsel-unknown-linux-musl`.
  - GitHub Actions workflow for automated firmware builds on every Git tag.
- [ ] **4.2. Kernel Footprint Optimization**:
  - Prune unused legacy networking protocols (AppleTalk, IPX, obsolete Netfilter modules).
  - Optimize SquashFS compression dictionaries for faster decompression on MIPS 24KEc.
- [ ] **4.3. Automated Smoke Tests**:
  - QEMU-based MIPS system emulation testing boot sequence, API endpoints, and firewall rules in CI before flashing to physical hardware.
