# Padavan-Next: Hardware Specification (NAG SNR-CPE-MD1.1)

This document details the low-level hardware registers, GPIOs, memory limits, and pin assignments of the **NAG SNR-CPE-MD1.1** router based on the **MediaTek MT7620A** SoC.

---

## 1. System-on-Chip (SoC) Specifications

```
  +---------------------------------------------------------------+
  |                     MediaTek MT7620A SoC                      |
  |                                                               |
  |   +-----------------------+     +--------------------------+  |
  |   | MIPS 24KEc Core       |     | Packet Processing Engine |  |
  |   | 580 / 600 MHz         |     | (HWNAT / FOE Flow Engine)|  |
  |   | 64KB I-Cache / 32KB D |     +--------------------------+  |
  |   +-----------------------+                                   |
  |                                 +--------------------------+  |
  |   +-----------------------+     | 5-Port 10/100 Switch     |  |
  |   | 2.4GHz 2T2R Wi-Fi MAC |     | Integrated 100M PHYs     |  |
  |   | 802.11b/g/n (300Mbps) |     +--------------------------+  |
  |   +-----------------------+                                   |
  |                                 +--------------------------+  |
  |   +-----------------------+     | PCIe Gen 1 Host Bridge   |  |
  |   | DDR2 Memory Controller|     | --> MT7610EN 5GHz Wi-Fi  |  |
  |   | 16-bit, up to 128MB   |     +--------------------------+  |
  |   +-----------------------+                                   |
  +---------------------------------------------------------------+
```

- **Model**: MediaTek MT7620A
- **Silicon Revision**: `0206`
- **CPU Architecture**: MIPS32 Release 2 (MIPS 24KEc)
- **BogoMIPS**: 385.02 (at 580 MHz) / ~399 (at 600 MHz)
- **Primary Instruction Cache**: 64 KB, 4-way associative, VIPT, 32-byte cache line.
- **Primary Data Cache**: 32 KB, 4-way associative, PIPT, 32-byte cache line.
- **Clock Frequencies**:
  - CPU Clock: 580 MHz default (600 MHz with `CONFIG_RALINK_MT7620_PLL600=y`)
  - OCP / Bus Clock: 193 MHz (200 MHz at 600 MHz PLL)
  - System Clock: 193 MHz
  - Reference Crystal: 20 MHz XTAL

---

## 2. Memory & Storage Layout

### 2.1 RAM (64 MB DDR2)
- **Chip**: 16-bit DDR2 SDRAM (Nanya or Winbond)
- **Physical Address Space**: `0x00000000` to `0x04000000` (64 MB total)
- **Usable Memory**: 61,840 KB after kernel decompression and boot structures.
- **ZRAM**: Virtual compressed block device configured at 50% RAM (`~32 MB`), dynamically freeing inactive pages to prevent out-of-memory (OOM) conditions during peak connection load.

### 2.2 SPI NOR Flash (8 MB)
- **Chip**: Winbond W25Q64BV (SPI-NOR, 8,192 KB / 64 Mbit)
- **SPI Clock**: 24 MHz
- **Erase Block Size**: 64 KB
- **Page Size**: 256 bytes
- **MTD Layout**:
  - `mtd0`: `0x00000000 - 0x00030000` (192 KB) -> Bootloader (U-Boot)
  - `mtd1`: `0x00030000 - 0x00040000` (64 KB) -> NVRAM Config
  - `mtd2`: `0x00040000 - 0x00050000` (64 KB) -> Factory (EEPROM RF Calibration)
  - `mtd3`: `0x00050000 - 0x0014C2C0` (Variable) -> Kernel (lzma)
  - `mtd4`: `0x0014C2C0 - 0x007E0000` (Variable) -> RootFS (SquashFS 4.0 XZ)
  - `mtd5`: `0x007E0000 - 0x00800000` (128 KB) -> Storage (`/etc/storage` user files)

---

## 3. Ethernet Switch & Port Architecture

- **Switch Type**: MT7620 internal 5-port 10/100 Fast Ethernet switch with embedded PHYs.
- **Port Mapping**:
  - Port 0: LAN 1
  - Port 1: LAN 2
  - Port 2: LAN 3
  - Port 3: LAN 4
  - Port 4: WAN (Internet)
  - Port 6: CPU interface (Internal GMAC / RGMII)
- **Switch Controller**: Accessed via `mii_mgr` and `/proc/net/ralink_esw`.
- **Hardware Acceleration**:
  - Ralink PPE (Packet Processing Engine) offloads IPv4/IPv6 packet header rewrites and checksum verification in silicon.
  - Zero CPU utilization during wire-speed 100 Mbps WAN-to-LAN bidirectional throughput.

---

## 4. Wireless RF Subsystems

### 4.1 2.4 GHz Band (SoC Internal)
- **Radio MAC/BBP**: MT7620 built-in 2T2R (2 Transmit, 2 Receive).
- **Standards**: IEEE 802.11b, 802.11g, 802.11n.
- **Maximum Theoretical PHY Rate**: 300 Mbps (40 MHz channel, 400ns Short GI).
- **Antenna Configuration**: 2 external antennas.
- **Driver**: `rt2860v2` Linux kernel module.

### 4.2 5 GHz Band (PCIe Transceiver)
- **Radio MAC/BBP**: MediaTek MT7610EN connected via internal PCIe bus `0000:01:00.0` (Vendor: `14c3`, Device: `7650`).
- **Standards**: IEEE 802.11a, 802.11n, 802.11ac.
- **Maximum Theoretical PHY Rate**: 433 Mbps (80 MHz channel, VHT80, 256-QAM).
- **Antenna Configuration**: 1T1R internal/external antenna.
- **Driver**: `mt7610` Linux kernel module.

---

## 5. GPIO, LEDs, and Buttons

From `user/shared/include/ralink_board.h`:

| Function | Pin / GPIO | Active State | Description |
| :--- | :--- | :--- | :--- |
| **Reset Button** | GPIO 13 | Active LOW | Hold for 5s to restore factory defaults. |
| **WPS Button** | Not present | N/A | Board does not feature dedicated WPS key. |
| **Power LED** | GPIO 0 / SoC | Active LOW | Power and boot indicator. |
| **LAN LEDs** | Switch controlled | Dynamic | Hardware link/activity indicators on RJ45 ports. |
| **WAN LED** | Switch controlled | Dynamic | Hardware link/activity indicator on WAN port. |

---

## 6. Serial Console (UART)

- **Port**: UART Lite (`ttyS0`)
- **Base Address**: `0x10000c00`
- **Interrupt**: IRQ 20
- **Baud Rate**: `57600 8N1` (57600 baud, 8 data bits, no parity, 1 stop bit)
- **Kernel Command Line**: `console=ttyS0,57600n8`
- **PCB Headers**: 4-pin unpopulated UART header (VCC 3.3V, GND, TX, RX) located near the SoC.
