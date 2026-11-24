# FerrumOS Fail-Safe and Anti-Brick Architecture

Firmware corruption and device lockouts pose critical operational risks in embedded network equipment. FerrumOS implements a multi-layer anti-brick architecture designed to maintain hardware integrity, guarantee configuration recovery, and provide emergency restoration mechanisms.

---

## 1. Threat Model & Mitigation Matrix

| Failure Mode | Root Cause | FerrumOS Defense Mechanism | Verification Standard |
| :--- | :--- | :--- | :--- |
| **Bootloader Corruption** | Accidental or malicious write to `mtd0` | Kernel-level MTD write lock (`MTD_CAP_ROM`) | `EPERM` returned on write attempt |
| **RF Calibration Loss** | Erasing or modifying `mtd2` EEPROM | Kernel-level MTD write lock (`MTD_CAP_ROM`) | Read-only attribute verified in kernel VFS |
| **Mid-Write Power Loss** | Power severed during flash write | Atomic A/B configuration double-buffering | CRC32 validated; automatic fallback |
| **Network Lockout** | Malformed IP or firewall configuration | Hardware Reset Pin Failsafe Mode | Boots to static RAM recovery (`192.168.1.1`)|
| **Kernel Freeze / Panic** | Infinite loop or memory exhaustion | Hardware Watchdog Timer (MT7620 WDT) | 30s timeout triggers hardware reset |
| **Corrupt Image Flash** | Truncated or corrupted firmware upload | Pre-flash header, model, and CRC32 checks | Rejection prior to flash erase |

---

## 2. Technical Safeguard Specifications

### 2.1 Kernel-Enforced Critical Partition Protection (`mtd0` & `mtd2`)

The SPI NOR flash contains two partitions critical to hardware viability:
- **`mtd0` (Bootloader, 192 KB)**: Contains low-level board initialization and U-Boot binary.
- **`mtd2` (Factory EEPROM, 64 KB)**: Contains unique radio frequency power amplification calibration offsets, crystal frequency adjustments, and factory MAC addresses for the 2.4 GHz and 5 GHz transceivers.

In the Linux kernel MTD driver (`drivers/mtd/mtdsplitter_parts.h`), write operations to these partitions are permanently disabled:

```c
/* Enforce Read-Only at the kernel VFS layer */
parts[0].mask_flags = MTD_WRITEABLE; /* mtd0 Bootloader is permanently READ-ONLY */
parts[2].mask_flags = MTD_WRITEABLE; /* mtd2 Factory EEPROM is permanently READ-ONLY */
```

Any attempt by a user process, shell script, or daemon to open `/dev/mtd0` or `/dev/mtd2` with `O_WRONLY` or `O_RDWR` flags results in an immediate `EPERM` (Operation not permitted) error. Hardware calibration data cannot be erased or overwritten from userland.

---

### 2.2 Transactional A/B Configuration Double-Buffering

The persistent configuration partition `mtd1` (64 KB) is partitioned into two 32 KB blocks: **Slot A** and **Slot B**.

Each block adheres to the following binary structure:
1. **Magic Header** (4 bytes): `0x46455252` (`"FERR"`).
2. **Generation Counter** (8 bytes): Monotonically increasing unsigned 64-bit integer.
3. **CRC32 Checksum** (4 bytes): IEEE 802.3 CRC32 calculated over the payload.
4. **Configuration Payload** (Variable): Serialized JSON configuration.

```
+---------------------------------------------------------------+
|                    mtd1 Partition (64 KB)                     |
+-------------------------------+-------------------------------+
|         Slot A (32 KB)        |         Slot B (32 KB)        |
+-------------------------------+-------------------------------+
| Header: "FERR" (0x46455252)   | Header: "FERR" (0x46455252)   |
| Generation Counter (u64)      | Generation Counter (u64)      |
| Checksum: CRC32 (u32)         | Checksum: CRC32 (u32)         |
| Serialized Config Payload     | Serialized Config Payload     |
+-------------------------------+-------------------------------+
```

**Write Sequence**:
1. Identify the inactive slot (slot with lower generation counter or corrupted CRC).
2. Write updated configuration payload to the inactive slot.
3. Calculate CRC32 checksum over the payload and write to header.
4. Atomically commit the updated generation counter (`max(gen_a, gen_b) + 1`).

**Recovery Protocol**:
On startup, `ferrum-core` reads headers from both slots:
- If Slot A has a valid CRC32 and Slot B is corrupted, Slot A is loaded.
- If both slots have valid checksums, the slot with the higher generation counter is loaded.
- If both slots fail CRC32 verification, FerrumOS reverts to an immutable in-memory default configuration (`192.168.1.1/24`, DHCP enabled) without halting the boot process.

---

### 2.3 Hardware Reset Pin Failsafe Mode (GPIO 13)

The physical Reset button is connected to MediaTek MT7620A **GPIO 13** (Active LOW).

- **Early Boot Detection**: During the initial 3 seconds of the kernel boot process (indicated by rapid blinking of the system LED), holding the Reset button triggers emergency failsafe mode.
- **Failsafe Execution Sequence**:
  1. Bypasses loading of persistent flash configuration (`mtd1`).
  2. Mounts the root filesystem as a read-only RAM overlay (`tmpfs`).
  3. Configures interface `br0` with static IP `192.168.1.1/24` and starts a minimal DHCP server.
  4. Launches an in-memory emergency recovery HTTP endpoint on port 80.
  5. Allows the operator to upload a replacement firmware image or reset configuration via browser.

---

### 2.4 Hardware Watchdog Timer (MT7620 WDT)

The MT7620A SoC incorporates a hardware watchdog timer connected directly to the CPU internal reset line.

- **Supervision**: `ferrum-core` writes keepalive pulses to `/dev/watchdog` every 5 seconds.
- **Timeout**: The watchdog expiration deadline is configured to 30 seconds.
- **Panic Recovery**: If a kernel deadlock, memory starvation freeze, or process hang prevents keepalive writes for 30 consecutive seconds, the hardware watchdog pulls the CPU reset line, initiating a clean reboot.
- **Boot Loop Safeguard**: If three consecutive watchdog reboots occur within a 5-minute window, FerrumOS automatically boots in rescue safe-mode with custom firewall rules, proxy services, and user scripts disabled.

---

### 2.5 Pre-Flash Image Validation Protocol

Prior to executing flash erase or write routines via `mtd_write`, firmware upgrade images undergo strict validation:

1. **Magic Header Verification**: Image must begin with valid TRX header `0x30524448` (`"HDR0"`).
2. **Platform Compatibility Check**: Board model string embedded in image must match `"SNR-MD1"`.
3. **Integrity Check**: Verifies image-wide CRC32 checksum against header specification.
4. **Size Boundary Enforcement**: Verifies total image byte length does not exceed the maximum kernel+rootfs boundary of `8,060,928 bytes`, preventing memory overlap with partition `mtd5`.
5. **RAM Staging**: Images are staged in RAM (`/tmp`) before flashing; network disconnects during upload will not trigger partial or corrupt flash writes.
