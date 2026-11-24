# FerrumOS Hardware Specification (NAG SNR-CPE-MD1.1)

This document details the low-level hardware architecture, register maps, GPIO assignments, memory boundaries, and RF parameters of the **NAG SNR-CPE-MD1.1** wireless router based on the **MediaTek MT7620A** System-on-Chip.

---

## 1. System-on-Chip (SoC) Architecture

```
+---------------------------------------------------------------------------------+
|                              MediaTek MT7620A SoC                               |
|                                                                                 |
|   +-----------------------+                    +----------------------------+   |
|   | MIPS 24KEc CPU Core   |                    | Packet Processing Engine   |   |
|   | 580 / 600 MHz         |                    | (HWNAT / FOE Flow Engine)  |   |
|   | 64KB I-Cache / 32KB D |                    +----------------------------+   |
|   +-----------------------+                                                     |
|                                                +----------------------------+   |
|   +-----------------------+                    | 5-Port 10/100 Switch       |   |
|   | 2.4GHz 2T2R Wi-Fi MAC |                    | Integrated 100M PHYs       |   |
|   | 802.11b/g/n (300Mbps) |                    +----------------------------+   |
|   +-----------------------+                                                     |
|                                                +----------------------------+   |
|   +-----------------------+                    | PCIe Gen 1 Host Bridge     |   |
|   | DDR2 Memory Controller|                    | --> MT7610EN 5GHz Wi-Fi    |   |
|   | 16-bit, up to 128MB   |                    +----------------------------+   |
|   +-----------------------+                                                     |
+---------------------------------------------------------------------------------+
```

- **Processor Model**: MediaTek MT7620A
- **Silicon Revision**: `0206`
- **CPU Architecture**: MIPS32 Release 2 (MIPS 24KEc core)
- **BogoMIPS**: 385.02 (at 580 MHz default) / ~399.00 (at 600 MHz PLL)
- **Primary Instruction Cache (L1 I-Cache)**: 64 KB, 4-way set associative, VIPT, 32-byte cache line.
- **Primary Data Cache (L1 D-Cache)**: 32 KB, 4-way set associative, PIPT, 32-byte cache line.
- **Operating Frequencies**:
  - CPU Core Clock: 580 MHz default (600 MHz with `CONFIG_RALINK_MT7620_PLL600=y`)
  - OCP / Bus Clock: 193 MHz (200 MHz at 600 MHz PLL)
  - System Clock: 193 MHz
  - Reference Crystal: 20 MHz XTAL

---

## 2. Memory & Flash Storage Specification

### 2.1 RAM (64 MB DDR2-400)
- **Memory Chip**: 16-bit DDR2 SDRAM (Nanya or Winbond)
- **Physical Address Range**: `0x00000000` to `0x04000000` (64 MB total)
- **Usable Kernel Memory**: 61,840 KB after kernel decompression and early page tables.
- **ZRAM Compressed Swap**: Virtual block device configured at 50% RAM (`~32 MB`), utilizing LZO compression to prevent out-of-memory states under high connection concurrency.

### 2.2 SPI NOR Flash (8 MB)
- **Flash Model**: Winbond W25Q64BV (SPI NOR, 8,192 KB / 64 Mbit)
- **SPI Bus Clock**: 24 MHz
- **Erase Block Size**: 64 KB
- **Page Programming Size**: 256 bytes
- **MTD Partition Map**:
  - `mtd0`: `0x00000000 - 0x00030000` (192 KB) -> Bootloader (U-Boot, Read-Only)
  - `mtd1`: `0x00030000 - 0x00040000` (64 KB)  -> Configuration (A/B Transactional)
  - `mtd2`: `0x00040000 - 0x00050000` (64 KB)  -> Factory (RF Calibration, Read-Only)
  - `mtd3`: `0x00050000 - Dynamic`    (~1.0 MB) -> Kernel (`zImage.lzma`)
  - `mtd4`: `Dynamic - 0x007E0000`    (~4.6 MB) -> RootFS (SquashFS 4.0 XZ)
  - `mtd5`: `0x007E0000 - 0x00800000` (128 KB) -> Storage (`/etc/storage` user files)

---

## 3. Ethernet Switch & Port Architecture

- **Switch Model**: MediaTek MT7620 embedded 5-port 10/100 Fast Ethernet switch with integrated PHYs.
- **Port Mapping**:
  - Port 0: LAN Port 1 (`eth2.1`, VLAN ID 1)
  - Port 1: LAN Port 2 (`eth2.1`, VLAN ID 1)
  - Port 2: LAN Port 3 (`eth2.1`, VLAN ID 1)
  - Port 3: LAN Port 4 (`eth2.1`, VLAN ID 1)
  - Port 4: WAN Port (`eth2.2`, VLAN ID 2)
  - Port 6: CPU RGMII Port (Internal GMAC, VLAN 1 and VLAN 2 Tagged)
- **Switch Control**: Programmed via `ioctl` on `/dev/esw` and register interface at `/proc/net/ralink_esw`.
- **Packet Acceleration**: Ralink Packet Processing Engine (PPE HWNAT) offloads IPv4/IPv6 header rewrites and checksum calculations directly in silicon.

---

## 4. Wireless Radio Subsystems

### 4.1 2.4 GHz Band (SoC Integrated)
- **Radio MAC/BBP**: MT7620 integrated 2T2R (2 Transmit, 2 Receive).
- **Supported Standards**: IEEE 802.11b, 802.11g, 802.11n.
- **Maximum PHY Rate**: 300 Mbps (40 MHz channel, 400ns Short GI).
- **Antenna Configuration**: 2 external omnidirectional antennas.
- **Kernel Driver**: `rt2860v2` Linux kernel module.

### 4.2 5 GHz Band (PCIe Transceiver)
- **Radio MAC/BBP**: MediaTek MT7610EN connected via internal PCIe bus `0000:01:00.0` (Vendor: `14c3`, Device: `7650`).
- **Supported Standards**: IEEE 802.11a, 802.11n, 802.11ac.
- **Maximum PHY Rate**: 433 Mbps (80 MHz channel, VHT80, 256-QAM).
- **Antenna Configuration**: 1 internal/external antenna.
- **Kernel Driver**: `mt7610` Linux kernel module.

---

## 5. GPIO, LEDs, and Peripherals

| Function | Pin / GPIO | Active State | Subsystem Description |
| :--- | :--- | :--- | :--- |
| **Reset Button** | GPIO 13 | Active LOW | Hold for 3s during boot for failsafe recovery; hold for 5s for factory reset. |
| **Power LED** | GPIO 0 / SoC | Active LOW | Power and system initialization indicator. |
| **LAN Port LEDs** | Switch Internal | Dynamic Link/Act | Hardware-driven link and activity indication per RJ45 port. |
| **WAN Port LED** | Switch Internal | Dynamic Link/Act | Hardware-driven link and activity indication for WAN RJ45 port. |

---

## 6. Serial Console Interface (UART)

- **Port**: UART Lite (`ttyS0`)
- **Memory Base Address**: `0x10000c00`
- **Interrupt Vector**: IRQ 20
- **Baud Rate**: `57600 8N1` (57600 baud, 8 data bits, no parity, 1 stop bit)
- **Kernel Console Parameter**: `console=ttyS0,57600n8`
- **Header Pinout**: 4-pin unpopulated UART header (Pin 1: VCC 3.3V, Pin 2: TX, Pin 3: RX, Pin 4: GND).
