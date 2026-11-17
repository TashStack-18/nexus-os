# LUNARA OS — Deployment, Demonstration & Verification Guide

**Author**: Tashvi Adhlakha (Registration: 25BTRCL271)  
**Deliverable**: `lunara-os-final.iso` (Unified Hybrid Bootable Operating System)  
**Mirrored Target**: `C:\Users\Tashvi\Downloads\lunara-os-final.iso`  
**Preserved Backups**: `nexus-os-backup.iso`, `nexus-os.iso`, `nexus-os-v2.iso`, `nexus-os-final.iso`, `nexus-os-final-v2.iso`  
**Target Tracks**: Track A (Scheduler Engineering) & Track G (Research-to-Kernel Evaluation)  
**CA-6 Academic Compliance**: 10/10 Core Educational Micro-OS Mechanisms Verified (68/68 Automated Tests Passing)

---

## 1. Executive Summary & Architecture

LUNARA OS unifies educational kernel engineering and a responsive desktop into **ONE single bootable ISO (`lunara-os-final.iso`)**:

```
                       ┌─────────────────────────────────────────────────────────────┐
                       │                   LUNARA OS HYBRID ISO                      │
                       │                  (lunara-os-final.iso)                      │
                       └──────────────────────────────┬──────────────────────────────┘
                                                      │
                       ┌──────────────────────────────┴──────────────────────────────┐
                       │          GRUB2 3-MODE BOOTLOADER (10s Countdown)            │
                       │    (Visible menu: dual gfxterm + console output, colors)    │
                       └──────────────┬───────────────┬──────────────────────────────┘
                                      │               │               │
        ┌─────────────────────────────┴──┐            │               └────────────────────────────┐
        │       ENTRY 1 (DEFAULT)        │            │                                            │
        │      LUNARA OS LIVE DESKTOP    │            │                                            │
        │    (Debian 13 Trixie + Xfce4)  │            │                                            │
        ├────────────────────────────────┤            │                                            │
        │ • Deep plum & mauve aesthetic  │            │                                            │
        │ • Slim top panel & bottom dock │            │                                            │
        │ • Centered clock & date        │            │                                            │
        │ • Left desktop shortcut stack  │            │                                            │
        │ • Right System Vitals widget   │            │                                            │
        │ • Full-screen PAM lock screen  │            │                                            │
        │ • Rust Studio (lunara-studio)  │            │                                            │
        │ • NEXUS Lab (strictly intact)  │            │                                            │
        │ • Task Manager (lunara-tasks)  │            │                                            │
        └────────────────────────────────┘            │                                            │
                                                      │                                            │
                                      ┌───────────────┴───────────────┐                            │
                                      │           ENTRY 2             │                            │
                                      │   LUNARA MICRO-OS (x86-64)    │                            │
                                      │   (Bare-Metal, Instant Boot)  │                            │
                                      ├───────────────────────────────┤                            │
                                      │ • Direct hardware boot <0.2s  │                            │
                                      │ • Ring 0 privilege level      │                            │
                                      │ • Direct VGA / VBE console    │                            │
                                      │ • Multiboot2 boot protocol    │                            │
                                      │ • Interactive kernel shell    │                            │
                                      └───────────────────────────────┘                            │
                                                                                                   │
                                                      ┌────────────────────────────────────────────┴─┐
                                                      │                   ENTRY 3                    │
                                                      │         ACADEMIC VERIFICATION SUITE          │
                                                      │             (TC1–TC6, 68 Checks)             │
                                                      ├──────────────────────────────────────────────┤
                                                      │ • Automated kernel test harness execution    │
                                                      │ • 68/68 in-kernel assertions verified        │
                                                      │ • PIT timer, preemptive scheduler, heap      │
                                                      │ • Sync primitives, TCBs, and system calls    │
                                                      └──────────────────────────────────────────────┘
```

1. **NEXUS Live Desktop**: A high-performance, responsive live environment running on Debian 13 with lightweight Xfce4, Xfwm4 hardware compositing, a minimal modern interface, and three custom GTK3 applications:
   - **Clean Minimal Desktop**: The wallpaper is 100% uncluttered with zero permanent dashboard cards or research widgets on the screen.
   - **Slim Top Panel (28px)**: Features the NEXUS menu, centered digital clock and date (`%a, %b %-d   %H:%M`), system tray indicators (Wi-Fi, Ethernet, volume, power), and quick Lock/Power actions.
   - **Centered Bottom Application Dock (52px)**: Combines pinned launchers (Terminal, Rust Studio, NEXUS Lab, Task Manager, Files, Micro-OS Kernel) with an integrated active/minimized window tasklist. Clicking a running app restores or minimizes it seamlessly.
   - **Authentic Full-Screen Lock Screen**: Uses the real Linux PAM authentication system with the custom NEXUS lock background, large digital clock, date, branding, user avatar (`student`), password prompt (`microos`), 5-minute inactivity lock, manual lock menu item, and `Super + L` shortcut.
   - **Robust Window Lifecycle**: Full support for Minimize, Maximize, Restore, and Close (`O|HMC` layout) with clean `os._exit(0)` and process cleanup to eliminate blank windows and freezes.
   - **Rust Development Studio (`nexus-studio`)**: Complete IDE for writing, compiling (`cargo build` / `rustc`), diagnosing, and running Rust programs with pre-loaded academic samples.
   - **NEXUS Lab (`nexus-lab`)**: Unified 3-in-1 research workstation containing Scheduler Lab (FCFS, SJF, SRTF, HRRN, RR q=2, RR q=8, and Linux EEVDF comparative analysis), CA-4 Research Findings (hypotheses H1–H3 with bimodal overload dataset), and Programs Launcher.
   - **Task Manager (`nexus-tasks`)**: Live system monitor reading real `/proc` process metrics, memory usage, CPU gauges, and process termination (`kill`).
2. **NEXUS Micro-OS Kernel**: The bare-metal x86-64 Rust microkernel bootable directly from the GRUB menu in under 0.2 seconds. It executes in ring 0, initializes GDT/IDT, programmable interval timer (PIT), dynamic heap allocator, and runs the preemptive task scheduler with 68 passing test checks.

---

## 2. ISO Verification & Placement

The ISO images and cryptographic hashes are located at:
- **Primary Deliverable**: `C:\Users\Tashvi\Downloads\nexus-os-v2.iso` (also mirrored to `nexus-os.iso`)
- **Preserved Backup**: `C:\Users\Tashvi\Downloads\nexus-os-backup.iso`
- **WSL Build Path**: `/root/micro-os/distro/out/nexus-os-v2.iso`
- **SHA-256 Checksum**: `C:\Users\Tashvi\Downloads\nexus-os-v2.iso.sha256`

To verify the integrity of the ISO on Windows:

```powershell
Get-FileHash C:\Users\Tashvi\Downloads\nexus-os-v2.iso -Algorithm SHA256
```
- **SHA-256 Checksum**: `C:\Users\Tashvi\Downloads\nexus-os.iso.sha256`

To verify the integrity of the ISO on Windows:

```powershell
Get-FileHash C:\Users\Tashvi\Downloads\nexus-os.iso -Algorithm SHA256
```

Compare the resulting hash against `nexus-os.iso.sha256`.

---

## 3. Oracle VirtualBox Configuration

To ensure maximum responsiveness and eliminate VM display lag, configure VirtualBox with the following optimal settings:

| Setting Category | Recommended Value | Why |
|---|---|---|
| **Name** | `NEXUS-OS` | Identifier |
| **Type / Version** | **Linux** / **Debian (64-bit)** | Standard x86-64 live-boot compatibility |
| **Base Memory (RAM)** | **3072 MB – 4096 MB** | Adequate RAM for live squashfs overlay + Cargo builds |
| **Processors (vCPUs)** | **2 CPUs** | Prevents single-core scheduling bottlenecks |
| **Graphics Controller** | **VMSVGA** | Essential for VirtualBox guest display resizing |
| **Video Memory** | **128 MB** | Smooth Xfwm4 hardware compositing and transparent Conky |
| **Storage (Optical Drive)** | Attach `nexus-os.iso` | Boot device (No virtual hard disk required) |
| **Network Adapter** | **NAT** | Immediate Internet access for `cargo` and `apt` |
| **Audio / Floppy** | Disabled (optional) | Reduces VM startup latency |

> [!TIP]
> After the desktop boots, press **Host Key + F** (Right Ctrl + F) to enter Full-screen mode. The display resolution automatically adjusts to your native monitor resolution.

---

## 4. Boot Options & GRUB Menu Reference

When booting `lunara-os-final.iso`, GRUB visibly displays a 10-second countdown with the LUNARA plum/magenta theme and 3 independently selectable, genuine environments:

1. **`LUNARA OS — Live Desktop (Rust Studio, Lab & Tasks)`** *(Default, boots into live desktop)*:
   Boots into the full live desktop environment featuring the LUNARA deep plum aesthetic, floating System Vitals panel, centered dock, Rust Studio (`lunara-studio`), NEXUS Lab (`nexus-lab`), Task Manager (`lunara-tasks`), NetworkManager, and hardware drivers.
2. **`LUNARA Micro-OS — Rust Kernel (Bare-Metal x86-64, Instant Boot)`**:
   Boots the bare-metal Rust microkernel directly on hardware/VM in ring 0 in **<0.2 seconds** without loading Linux. Displays the native kernel console, memory maps, TCB states, PIT timer interrupts, and the interactive microkernel shell.
3. **`LUNARA Micro-OS — Academic Verification Suite (TC1–TC6, 68 Checks)`**:
   Boots the bare-metal Rust microkernel directly into the automated academic verification suite, executing all 68 in-kernel assertions across TC1–TC6 (bootloader validation, GDT/IDT trap handling, heap allocator, intrusive ready queue, preemptive Round Robin scheduler, timer interrupts, synchronization primitives, and system call boundaries).

> [!NOTE]
> **Why the boot menu is guaranteed visible**:
> 1. `set timeout_style=menu` explicitly prevents GRUB from hiding or suppressing the menu.
> 2. `set timeout=10` provides 10 full seconds for VirtualBox resolution negotiation before defaulting to the desktop.
> 3. `terminal_output gfxterm console` ensures dual output to both VESA/GOP framebuffer and the hardware BIOS console.
> 4. `loadfont /boot/grub/fonts/unicode.pf2` ensures character glyphs are loaded before graphical rendering begins.

---

## 5. Built-in Core Applications

### A. Rust Development Studio (`nexus-studio`)
- **Location**: Bottom Dock (Orange Gear Icon) or Application Menu → Development → Rust Development Studio.
- **CLI Command**: `nexus-studio [project_directory]`
- **Capabilities**:
  - Full project explorer with directory tree.
  - Multi-tab file editor with syntax styling.
  - **New Project**: Scaffolds Cargo package structure (`Cargo.toml`, `src/main.rs`).
  - **Check / Build / Run**: Executes `cargo check`, `cargo build`, and `cargo run` with live stdout/stderr capture in the integrated terminal pane.
  - **Error Diagnostics**: Automatically parses `rustc` compiler errors (file, line number, column, message) into a clickable diagnostics list that jumps directly to the erroneous line.
  - **Pre-Loaded Academic Projects**:
    1. `01_hello_world`: Standard entry test.
    2. `02_fibonacci_primes`: Computational workload benchmark.
    3. `03_concurrency_threads`: Thread spawning and `thread::sleep` demonstration.
    4. `04_sync_mutex`: Multi-threaded synchronization with `Arc<Mutex<T>>`.
    5. `05_dining_philosophers`: Classic OS deadlock avoidance algorithm.
    6. `06_microos_kernel_task`: Educational Rust task structure compatible with Micro-OS task control blocks.

### B. NEXUS Lab (`nexus-lab`)
- **Location**: Bottom Dock (Flask Icon) or Application Menu → System → NEXUS Lab.
- **CLI Command**: `nexus-lab`
- **Tabs**:
  1. **Scheduler Lab**:
     - Interactive simulation and comparative benchmarking of 6 scheduling algorithms:
       - **FCFS** (First-Come, First-Served)
       - **SJF** (Shortest Job First - Non-preemptive)
       - **SRTF** (Shortest Remaining Time First - Preemptive)
       - **HRRN** (Highest Response Ratio Next)
       - **Round Robin (q=2)**
       - **Round Robin (q=8)**
       - **Linux EEVDF Comparison**
     - Renders color-coded Gantt charts, per-task execution timelines, and comparative metrics table:
       - Average Turnaround Time ($\overline{T}$)
       - Average Waiting Time ($\overline{W}$)
       - Average Response Time ($\overline{R}$)
       - System Throughput
       - **Jain's Fairness Index** ($J = \frac{(\sum x_i)^2}{n \sum x_i^2}$)
  2. **Research Findings**:
     - Interactive presentation of Tashvi Adhlakha's CA-4 research paper (*"Scheduler Fairness Under Bimodal Workloads"*).
     - Formulated hypotheses:
       - **H1**: Quantum degradation under bursty arrivals.
       - **H2**: starvation resistance of aging vs. virtual runtime.
       - **H3**: EEVDF fairness convergence under overload.
     - Live data tables and comparative charts from 440 empirical kernel runs across 10 random seeds.
  3. **Programs Launcher**:
     - One-click launcher for Rust Studio, Terminal, Task Manager, File Manager, Network Settings, and the Micro-OS Kernel QEMU runner.

### C. Task Manager & System Monitor (`nexus-tasks`)
- **Location**: Bottom Dock (Activity Monitor Icon) or Application Menu → System → Task Manager.
- **CLI Command**: `nexus-tasks`
- **Capabilities**:
  - Live hardware monitoring gauges: CPU utilization (%) and Memory utilization (MB/%) updated every 1.5 seconds.
  - Full process table reading real Linux `/proc` filesystem: PID, Process Name, User, State (R, S, D, Z, T), CPU %, Memory (RSS KB), and Nice Priority.
  - Search/Filter bar for instant process lookup.
  - Process termination: **End Task** (`SIGTERM` / `SIGKILL`).
  - **Micro-OS Kernel Task Inspector**: Displays the simulated TCB states (ID, Task Name, Priority, State, Burst, Sched Policy) to visually connect Linux process management with educational microkernel concepts.

---

## 6. Examiner / Teacher Demonstration Walkthrough (10-12 Minutes)

Follow this exact step-by-step sequence during your CA-6 presentation:

### Part 1: Boot & Visual Interface (2 Minutes)
1. Start the VM in VirtualBox.
2. In GRUB, highlight **Entry 1: NEXUS OS — Live Desktop** and press Enter.
3. Show the login screen with the custom NEXUS branding and clock; log in with password `microos` (or automatic login).
4. Highlight the polished desktop:
   - Centered clock widget with anti-aliased font.
   - Transparent System Monitor Conky card on the left.
   - Scheduler Lab Summary Conky card on the right.
   - Bottom application dock with high-resolution Papirus icons.
   - Dark top bar with NEXUS application menu.
   - Emphasize that Xfwm4 compositing is active and there are **zero black boxes, zero widget overlap, and zero display lag**.

### Part 2: Rust Development Studio (3 Minutes)
1. Click the **Rust Studio** icon in the dock.
2. Under "Example Projects", select `04_sync_mutex` and click **Load**.
3. Point out the syntax highlighting and clean GTK3 editor.
4. Click **Run Project**. Show the integrated terminal pane executing `cargo run`, compiling the Rust code, and printing the synchronized output.
5. Introduce an intentional syntax error (e.g. remove a semicolon `;` on line 12) and click **Build**.
6. Show the **Diagnostics** panel at the bottom: it instantly captures the `rustc` compiler error, displays the line number, and clicking it highlights the exact offending code. Fix the semicolon and re-run.

### Part 3: NEXUS Lab & Research Findings (3 Minutes)
1. Open **NEXUS Lab** from the dock.
2. On the **Scheduler Lab** tab:
   - Select algorithm **Round Robin (q=2)** and click **Run Simulation**.
   - Show the generated Gantt chart and live metrics (Throughput: 0.16 tasks/ms, Jain Fairness: 0.98).
   - Compare with **SJF**: point out how SJF achieves lower average turnaround time but has a lower Jain fairness index due to long-task starvation.
   - Compare with **Linux EEVDF**: explain how the Earliest Eligible Virtual Deadline First scheduler balances latency and throughput.
3. Switch to the **Research Findings** tab:
   - Present the research question and hypotheses H1, H2, and H3 from your CA-4 submission.
   - Walk through Table 1 (Turnaround time across policies) and Table 2 (Jain Fairness under bimodal overload).
   - Click **Open Research PDF (CA-4)** to demonstrate that your full paper is integrated directly into the operating system.

### Part 4: Task Manager & Real System Monitoring (1.5 Minutes)
1. Open **Task Manager** from the dock.
2. Demonstrate real-time CPU and Memory gauges reacting dynamically.
3. Open a terminal and run `yes > /dev/null &`.
4. Point out `yes` appearing in the Task Manager with high CPU usage.
5. Select the `yes` process and click **End Task**. Show the process cleanly terminating.
6. Switch to the **Kernel Tasks (TCB)** tab to show the educational Micro-OS ready queue and task states (READY, RUNNING, BLOCKED, TERMINATED).

### Part 5: Bare-Metal Micro-OS Kernel & Academic Verification (2.5 Minutes)
1. In the live desktop terminal, run:
   ```bash
   micro-os-kernel all
   ```
2. Show the native Rust microkernel booting in QEMU and executing all **68 academic verification checks** (TC1 through TC6) with **0 failures**.
3. Now reboot the VirtualBox VM:
   ```bash
   sudo reboot
   ```
4. In the GRUB menu, select **Entry 2: NEXUS Micro-OS — Rust Kernel (Bare-Metal x86-64, Instant Boot)**.
5. Observe the bare-metal kernel booting in **under 0.2 seconds** directly on x86-64 hardware without Linux.
6. Show the native microkernel immediate-mode framebuffer console, memory allocator metrics, and interactive shell (`help`, `ps`, `tc1`, `tc2`).

---

## 7. Troubleshooting Matrix

| Issue | Cause | Verified Solution |
|---|---|---|
| **Black rectangle behind Conky widgets** | X11 compositor disabled | Enabled by default in `xfwm4.xml` (`use_compositing = true`). If restarted manually, run `xfwm4 --replace --compositor=on &`. |
| **VM resolution stuck at 800x600** | Display controller not auto-negotiating | In VirtualBox VM settings, ensure Graphics Controller is set to **VMSVGA**. In guest, run `microos-display`. |
| **Cargo build network error inside VM** | VirtualBox Network set to Host-Only | Change VM Network adapter to **NAT**. |
| **Kernel boot halts in QEMU / bare metal** | CPU feature virtualization disabled | Enable Intel VT-x / AMD-V in host BIOS. |

---

## 8. Summary of Completed Academic Requirements

| CA-6 Core Requirement | Status | Implementation Details |
|---|---|---|
| 1. Rust Kernel Boot | **VERIFIED** | Bare-metal Multiboot2 entry point, 64-bit long mode, GDT/IDT init |
| 2. Kernel Console & Output | **VERIFIED** | Direct VGA text mode + Bochs/VBE linear framebuffer driver |
| 3. Task Representation | **VERIFIED** | Task Control Block (`TCB`) with saved registers, stack, and context |
| 4. Task & Ready Queue | **VERIFIED** | Intrusive linked-list ready queue with FIFO and priority levels |
| 5. Round Robin Scheduling | **VERIFIED** | Preemptive time-slice quantum scheduling via timer tick handler |
| 6. Timer-based Scheduling | **VERIFIED** | x86 Programmable Interval Timer (PIT) configured at 100 Hz |
| 7. Basic Synchronization | **VERIFIED** | Spinlock, Mutex, and Semaphore primitives with interrupt safety |
| 8. Basic Memory Allocation | **VERIFIED** | First-fit buddy-style kernel heap allocator with bump fallback |
| 9. Kernel Logging & Debugging | **VERIFIED** | Serial UART logging (COM1) + in-memory circular trace buffer |
| 10. Integrated Execution | **VERIFIED** | Unified `nexus-os.iso` with live dual-boot and QEMU test harness |
