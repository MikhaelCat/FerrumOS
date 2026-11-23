# FerrumOS: Anti-Brick & Fail-Safe Architecture

One of the most dangerous risks when writing custom firmware for embedded routers is **bricking** the device (making it unable to boot or inaccessible over the network).

**FerrumOS** incorporates a multi-layer **Anti-Brick Guarantee** designed to prevent hardware damage, flash corruption, and software lockouts under any circumstance.

---

## 1. Threat Analysis: Why Routers Brick

```
+------------------------------------+-------------------------------------------+
| Common Bricking Cause             | FerrumOS Defense Mechanism                |
+------------------------------------+-------------------------------------------+
| Overwriting Factory RF Calibration | Kernel-level MTD write lock on mtd2       |
| Corrupting Bootloader (U-Boot)     | Kernel-level MTD write lock on mtd0       |
| Power loss during Flash write      | Atomic A/B double-buffering with CRC32   |
| Bad network config / IP lockout    | Automatic Failsafe RAM reset (192.168.1.1)|
| Kernel crash / Infinite reboot loop| Hardware Watchdog (WDT) + Safe-Mode Boot  |
| Flashing corrupt / truncated image | Pre-flash header, size & SHA-256 checks   |
+------------------------------------+-------------------------------------------+
```

---

## 2. Multi-Layer Anti-Brick Guarantees

### Layer 1: Hardware-Enforced Partition Locking (`mtd0` & `mtd2`)
- **Partition `mtd0` (Bootloader / U-Boot, 192 KB)**: Contains the low-level hardware initialization code.
- **Partition `mtd2` (Factory / EEPROM, 64 KB)**: Contains unique radio frequency (RF) power amplification tables, crystal calibration offsets, and factory MAC addresses for the MT7620 2.4G and MT7610 5G radios.
- **FerrumOS Protection**:
  In the kernel driver (`linux-3.4.x/drivers/mtd/mtdsplitter_parts.h`), `mtd0` and `mtd2` are explicitly registered with read-only flags:
  ```c
  /* Enforce Read-Only at the kernel VFS layer */
  parts[0].mask_flags = MTD_WRITEABLE; // mtd0 Bootloader is READ-ONLY
  parts[2].mask_flags = MTD_WRITEABLE; // mtd2 Factory EEPROM is READ-ONLY
  ```
  Even if a malicious script, corrupted process, or root command runs `dd if=/dev/zero of=/dev/mtd2` or `mtd_write`, the kernel returns `EPERM` (Operation not permitted). The hardware calibration is 100% immune to erasure.

---

### Layer 2: Atomic Configuration Double-Buffering
- The persistent configuration partition `mtd1` (64 KB) is split into two 32 KB blocks: **Slot A** and **Slot B**.
- Each block contains:
  1. Magic Header (`0x46455252` — `"FERR"`).
  2. 64-bit Generation Counter.
  3. CRC32 Checksum.
  4. Serialized Configuration Payload.
- **Write Process**:
  1. FerrumOS writes the new configuration to the inactive slot.
  2. Computes and writes the CRC32 checksum.
  3. Atomically updates the generation counter and commits to flash.
- **Power Loss Recovery**:
  If power is disconnected while writing, the active slot remains intact. On boot, FerrumOS calculates the CRC32 of both slots:
  - If Slot B is invalid/corrupt, it boots from Slot A.
  - If both are invalid, it loads the **immutable in-memory factory default configuration** (`192.168.1.1`, DHCP enabled) without crashing.

---

### Layer 3: Physical Reset Pin Failsafe Mode (GPIO 13)
- The physical Reset button is wired to SoC pin **GPIO 13** (Active LOW).
- **Boot Detection**:
  - During the first 3 seconds of boot (when the power LED blinks rapidly), holding the Reset button triggers **FerrumOS Emergency Failsafe**.
- **Failsafe Behavior**:
  1. Completely ignores any settings on flash.
  2. Mounts RootFS in read-only RAM overlay.
  3. Configures LAN interface `br0` with static IP `192.168.1.1/24`.
  4. Starts a minimal emergency HTTP recovery server on port 80.
  5. The user can open `http://192.168.1.1/` in any browser and upload a clean firmware image or reset configuration in one click.

---

### Layer 4: Hardware Watchdog Timer (MT7620 WDT)
- The MT7620A SoC has a built-in hardware watchdog timer linked directly to the CPU reset line.
- The supervisor daemon (`ferrum-stated`) pings `/dev/watchdog` every 5 seconds.
- If the system hangs, locks in an infinite loop, or runs out of memory:
  1. Watchdog pings stop.
  2. After 30 seconds of inactivity, the hardware timer pulls the CPU reset line.
  3. The router reboots cleanly into Safe-Mode.
  4. If three consecutive watchdog resets occur within 5 minutes, FerrumOS disables all custom user scripts and Zapret DPI rules, booting in clean rescue mode.

---

### Layer 5: Pre-Flash Verification & Safe Flashing
Before `mtd_write` writes a byte of new firmware to `mtd3`/`mtd4`:
1. **Magic Header Check**: Verifies the TRX header (`0x30524448` — `"HDR0"`).
2. **Product ID Check**: Confirms the target board matches `"SNR-MD1"`.
3. **CRC32 & LZMA Integrity**: Unpacks and verifies the kernel compression header.
4. **Boundary Limit Check**: Confirms the total image size does not exceed `7,929,856 bytes` (to prevent spilling into the Storage partition `mtd5`).
5. **Sync to RAM**: The entire image is staged in `/tmp` (RAM tmpfs) before writing begins; network drops during upload will never trigger a partial flash.
