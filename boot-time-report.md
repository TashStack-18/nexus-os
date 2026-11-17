# MICRO-OS — Boot-Time Measurement Report (Stages S0 to S7)

## Overview & Methodology
This report provides precise empirical boot-time measurements across the progressive build stages of **MICRO-OS**, verifying the timing impact of each added subsystem as required by the OS coursework specification.

Measurements were conducted using `systemd-analyze` inside the guest environment for Linux userspace stages (S0–S6) and direct hardware cycle counters (`rdtsc`) / programmable interval timer (PIT) for the bare-metal Rust microkernel (S7). 

- **Test Machine**: HP Victus 15-fa2xxx / Oracle VM VirtualBox & KVM, 4 vCPUs, 4 GiB RAM.
- **Kernel**: Linux `6.12.116+deb13-amd64` / Rust Microkernel `0.1.0` (x86-64 freestanding bare-metal).
- **Repetitions**: 5 normal cold boots per stage (Mean ± Standard Deviation). First boot reflects initial machine-id and fontcache generation.

---

## Stage-by-Stage Boot Measurements

| Stage | Component Added | Packages | Root Size | Kernel (s) | Userspace (s) | Total Time (s) | Delta (s) | First Boot (s) | Ready Target | Slowest Systemd Unit |
|:---:|:---|---:|---:|---:|---:|---:|---:|---:|:---|:---|
| **S0** | Minimal base (kernel, systemd) — no package manager, no network | 106 | 331 MiB | 2.28 | 1.79 | **4.08 ± 0.58** | *baseline* | 4.44 | `multi-user.target` 1.14s | `dev-vda2.device` (2.048s) |
| **S1** | + LAN networking (`NetworkManager`, Realtek Ethernet firmware) | 159 | 398 MiB | 2.42 | 1.76 | **4.18 ± 0.14** | +0.10 | 3.90 | `multi-user.target` 1.42s | `dev-vda2.device` (559ms) |
| **S2** | + Wi-Fi (`wpa_supplicant`, `iw`, regulatory db, RTL8852BE driver + fw) | 167 | 403 MiB | 2.52 | 1.92 | **4.44 ± 0.16** | +0.26 | 4.25 | `multi-user.target` 1.57s | `dev-vda2.device` (593ms) |
| **S3** | + Laptop hardware (Intel microcode, graphics, audio, Bluetooth fw) | 182 | 530 MiB | 2.32 | 1.80 | **4.13 ± 0.19** | -0.31 | 7.31 | `multi-user.target` 1.43s | `dev-vda2.device` (543ms) |
| **S4** | + APT and aptitude package managers (Debian repositories) | 193 | 563 MiB | 2.50 | 1.70 | **4.20 ± 0.55** | +0.07 | 4.89 | `multi-user.target` 1.27s | `dev-vda2.device` (457ms) |
| **S5** | + `snapd` (Snap package daemon & AppArmor socket activation) | 207 | 648 MiB | 2.33 | 2.35 | **4.69 ± 0.62** | +0.49 | 6.25 | `multi-user.target` 2.35s | `snapd.apparmor.service` (796ms) |
| **S6** | + Full XFCE Desktop, LightDM, Theme & Snap Store GUI | 514 | 2535 MiB | 2.15 | 4.42 | **6.57 ± 0.38** | +1.88 | 48.94 | `graphical.target` 4.42s | `snapd.apparmor.service` (2.303s) |
| **S7** | + Independent Rust Microkernel (Bare-Metal x86-64, Instant Boot) | 0 | 6.9 MiB | 0.04 | 0.00 | **0.04 ± 0.002** | -6.53 | 0.04 | Ring 0 Kernel Shell | PIT Calibration (40ms) |

---

## Analysis of Boot-Time Impact

1. **Networking Impact (S0 → S1, S2)**:
   - Introducing `NetworkManager` and the Realtek LAN driver increased boot time by only 0.10s.
   - Adding `wpa_supplicant`, `iw`, and the RTL8852BE driver/firmware added 0.26s due to interface enumeration and asynchronous regulatory domain checks.

2. **Hardware Enablement (S2 → S3)**:
   - Loading Intel microcode and display/sound firmware produced a slight net improvement (-0.31s) because early kernel driver initialization stabilized faster.

3. **Package Managers (S3 → S4, S5)**:
   - Built-in APT and aptitude added virtually no daemon overhead (+0.07s) since APT operates strictly on-demand via CLI.
   - Adding `snapd` introduced a measurable +0.49s impact, primarily driven by `snapd.apparmor.service` parsing AppArmor security profiles.

4. **Graphical Desktop (S5 → S6)**:
   - Loading the full XFCE desktop environment, LightDM display manager, System Vitals monitor, and the Snap Store background service increased userspace initialization to 4.42s, bringing the total normal boot time to **6.57 seconds**.
   - First-boot initialization took 48.94s due to initial one-time font cache creation (`fc-cache`), Snap Store seed decompression, and desktop MIME database generation.

5. **Bare-Metal Rust Microkernel Comparison (S7)**:
   - The freestanding Rust microkernel bypasses Linux userspace completely, executing in 32-bit/64-bit protected Ring 0 directly from GRUB Multiboot2 in **0.04 seconds (40 ms)**, achieving instant interactive readiness with zero background daemon overhead.
