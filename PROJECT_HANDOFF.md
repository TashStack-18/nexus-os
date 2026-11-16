# NEXUS OS — Technical Handoff for Google Antigravity

**Purpose:** Continue the existing NEXUS / Mini-OS project from its current state. Do not rebuild from scratch.

**Source:** Extracted from the user's supplied Claude conversation export. This document distinguishes reported work from work that was explicitly unverified.

---

## 1. Project goal

The user wants **one complete, polished operating system** to demonstrate to their teacher, rather than several disconnected demos or separate deliverables.

The desired result is a normal-looking desktop OS with:
- A polished NEXUS-style desktop and login/lock experience.
- A functional desktop with task/program launching.
- The user's OS/kernel and scheduler research implemented as usable applications and visible findings.
- A unified NEXUS Lab application containing **Lab**, **Findings**, and **Programs**.
- A bootable ISO that can be demonstrated in Oracle VM VirtualBox.
- A clear deployment guide.

The user specifically said the existing desktop looked too simple and its layout looked bad. Visual quality and a cohesive, presentable experience are important acceptance criteria.

## 2. Environment and execution — verify before changing

Known from the conversation:
- Host machine: Windows.
- Virtual machine software: Oracle VM VirtualBox.
- A Debian-based Micro-OS Linux image was built as a live desktop ISO.
- A separate Rust kernel / x86-64 port ISO also exists.
- The conversation mentions QEMU in one specific place: the **Micro-OS kernel dock launcher starts the Rust kernel using QEMU bundled inside the Linux ISO**. The user previously said they were not using QEMU and clarified Oracle VirtualBox. Do not assume the whole project uses QEMU or that the launcher must remain. Inspect the actual scripts and ask before changing/removing it.
- Earlier notes mention QEMU as a development harness for the Rust kernel lock-screen test, while the lock-screen test was explicitly not performed in VirtualBox at that point.
- The Linux live ISO is intended to be the main demonstration OS. The Rust-only ISO is secondary unless the user decides otherwise.

**Important:** The exact current working directory, repository state, branch, and whether all files/commits are present in the current workspace are not established by this chat export. Inspect the supplied project folder and Git state before acting.

## 3. Project components and reported architecture

### A. Rust Mini-OS kernel / x86 port
Reported features and work:
- Rust-based OS/kernel work.
- x86-64 port for VirtualBox / ISO boot.
- Boot to a lock screen, then full-screen desktop.
- Lock screen reportedly displays clock, date, user badge, and “Click or press Enter to unlock”.
- A “Lock screen” item exists in the NEXUS menu.
- Kernel accepts a GRUB-selected 800×600, 24-bit framebuffer mode and switches the card to 1920×1080 itself.
- Scheduler policies discussed: FCFS, SJF, SRTF, HRRN, and Round Robin (RR).
- Scheduler lab / research experiment.
- System monitoring concepts shown in the desktop: CPU, memory/heap, load, calendar, clock, task/scheduler information.
- A Rust-only ISO exists at one point, but it cannot provide normal Linux Wi-Fi, APT, or Snap Store functionality.

Reported test status:
- At one point, 68/68 checks passed on RISC-V and x86.
- RISC-V repeat boot was byte-identical.
- The x86 test run was stated to be from before the lock screen, so it did not verify the lock-screen changes.
- The lock-screen test was performed in a QEMU development harness only, not VirtualBox.
- A lab experiment completed. Earlier result: RR q=2 and q=8 fairness scores 0.93 and 0.97; SJF completed the most jobs.
- Fairness metrics for FCFS, SJF, and HRRN showed `n/a` because short jobs completed within one 250 ms measurement sample. The conversation described this as a measurement limitation, not a kernel bug.
- Later text-mode R3 experiment (3 seconds per policy, run in WSL and not in the ISO) reported SJF highest throughput at 0.34 fairness, while RR was 0.71 and Linux EEVDF was 0.72. Do not conflate these results with the earlier experiment; they are different reported runs/measurement setups.

### B. Debian-based Micro-OS Linux live ISO
This is the intended single, normal desktop OS for the teacher demonstration.

Reported contents:
- Debian-based live image.
- Xfce desktop.
- NEXUS visual theme: dark top bar, centered clock, dock, wallpaper/colors, NEXUS menu.
- Login screen; login account reported as `student`, password `microos` (also displayed on the login screen).
- Dock reportedly contains five apps: Terminal, Files, Snap Store, Micro-OS kernel, and Network.
- A System card intended to show CPU, memory, load, and calendar.
- Scheduler card.
- `microos-lab` desktop application, with dock icon and menu entry, intended to run the R3 scheduler experiment.
- The Linux image was designed to include network support, laptop hardware support, APT/aptitude, and Snap Store.

Reported staged boot measurements from `results/distro_boottime.md`, using `systemd-analyze` and five boots per stage:
| Stage | Configuration | Reported boot total |
|---|---|---:|
| S0 | Baseline, no network or package manager | 4.08 s |
| S1 | LAN / NetworkManager | 4.18 s |
| S2 | Wi-Fi / wpa_supplicant / RTL8852BE | 4.44 s |
| S3 | Laptop hardware: microcode, graphics, audio, Bluetooth | 4.13 s |
| S4 | APT and aptitude | 4.20 s |
| S5 | snapd, then Snap Store app | 4.69 s |
| S6 | Full Xfce desktop with Snap Store app | 6.57 s |

These measurements were said to come from a KVM test VM, not the user's laptop, and were for a separate disk image/staged build. They were **not verified measurements for the final live ISO**. Do not present them as measured final-ISO boot times.

### C. Scheduler research / lab
Reported policy comparisons:
- FCFS
- SJF
- SRTF
- HRRN
- RR with q=2 and q=8
- Linux EEVDF
- Linux RR (with root)

Workload described in the conversation:
- R3 experiment.
- Bimodal workload.
- 110% load on one core.
- One run used 3 seconds per policy in WSL, not in the ISO.

Research should be presented honestly with the exact test setup, measurement definitions, and limitations. Do not invent findings or combine scores from different runs.

### D. NEXUS desktop / user experience
Design direction:
- NEXUS look and feel.
- Dark top bar.
- Centered clock and date.
- Dock.
- Matching wallpaper and colors.
- NEXUS menu.
- System and Scheduler cards.
- Lock/login screen.
- Cohesive, polished layout rather than a basic Xfce appearance.

Reported UI issues in a prior build:
- Clock widget had a black rectangle behind it.
- Cards overlapped the clock.
- Source fixes were reportedly made, but the rebuilt ISO and final appearance were not verified.
- VM display was still 1280×800 in a later test; full-screen resolution was not verified.

## 4. Historical files, artifacts, and commits mentioned

These are historical paths/names from the Claude conversation. They may or may not exist in the current workspace. Verify them rather than assuming.

### ISO files
- Rust/x86 ISO: `C:\Users\Tashvi\Downloads\micro-os-x86.iso`
  - Historical SHA-256: `349da65941514ca69fd76cf94e7729b540a95cfba9ba8a00365f82fd02725fdc`
- Debian-based live ISO: `C:\Users\Tashvi\Downloads\micro-os-linux-live.iso`
  - Historical size: about 1.7 GB.
  - Historical SHA-256: `38d50b5b03d21ca7ddc2180861679026795753724061cdf5ac2b3d3185f1d0c2`
  - Sidecar checksum mentioned: `micro-os-linux-live.iso.sha256`
- A VirtualBox VM named `Micro-OS-Linux` was reported as already created with the live ISO attached, configured with 3 GB RAM and 2 CPUs. Verify the VM and attached ISO before relying on it.

### Screenshots
Reported under `results/screenshots`:
- `x86_1_lock_screen.png`
- `x86_2_desktop.png`
- `x86_3_scheduler_lab.png`
- Older screenshots that may be stale:
  - `x86_1_desktop.png`
  - `x86_2_start_menu.png`
  - `x86_3_task_manager.png`
  - `x86_4_system_info.png`
- Linux desktop screenshots:
  - `linux_1_login_screen.png`
  - `linux_2_nexus_desktop.png`

### Documentation
- `docs/x86_port.md`
- `README.md`
- `results/distro_boottime.md`
- `x86-64 Port - VirtualBox (CA-6).docx` (reported as an 8-page Word report; source document had four fill-in markers)
- A deployment guide was reportedly being written near the end of the chat, but its final path/content/status is not specified.

The README and `docs/x86_port.md` were explicitly reported as stale at one point: they did not yet describe the Linux ISO or the new lab.

### Git commits mentioned
The chat mentions several local commits:
- `0eac59b`: lock-screen work; explicitly local only, not pushed at that time.
- `c680e17`: Word report work; explicitly not pushed at that time.
- `64a69b8`: Linux live ISO work; explicitly local only, not pushed at that time.
- `5555599`: later state; VM powered off and changes committed locally, not pushed.
- Another session reportedly pushed commits through `e2d6999` to GitHub.

These statements may describe different machines/sessions and commit histories. **Do not reset, cherry-pick, force-push, or assume which commit is current.** Run `git status`, `git log --oneline --decorate -15`, and inspect branches/remotes first.

## 5. Last known status from the supplied conversation

The last explicit task list was:
1. Find research write-up sources.
2. Build one NEXUS Lab app with Lab, Findings, Programs.
3. Polish desktop look.
4. Rebuild ISO and test in VirtualBox.
5. Write deployment guide and commit.

The conversation ended with the user asking Claude to continue after the usage limit reset. A deployment guide was said to have been written and an ISO build was said to be running, but the chat does not contain a confirmed final build result or final VirtualBox test result.

Therefore, current completion status is **unknown** for those last actions. Inspect files, build logs, and Git before deciding what remains.

## 6. Explicitly unverified / must retest

Do not claim these are working until verified on the actual current ISO:
- APT installation/use inside the final live ISO.
- Snap Store launch and functionality inside the final live ISO.
- LAN and Wi-Fi inside the final live ISO.
- Hardware support inside the final live ISO.
- Micro-OS kernel dock launcher inside the final live ISO.
- `microos-lab` launch and experiment completion inside the final live ISO.
- Final desktop layout after the widget fixes.
- Full-screen / desired VirtualBox resolution.
- Final live ISO boot time.
- Whether the Rust kernel launcher uses bundled QEMU and whether that is acceptable to the user.
- Whether the most recent ISO build completed and which ISO file is the latest.
- Whether the deployment guide is complete and accurate.
- Whether the latest local commits have been pushed.
- Whether research sources and citations have been added to the final write-up.

## 7. Recommended continuation plan

### Phase 1 — Audit, no edits
1. Inspect the full project tree.
2. Inspect `git status`, branches, remotes, and recent commits.
3. Identify the latest ISO, build scripts, and build logs.
4. Read README, docs, research/results files, and deployment guide.
5. Identify the current source of truth: Rust kernel, Debian live-image build, desktop config, and lab application.
6. Map which components are implemented in source versus actually included in the latest ISO.
7. Produce a concise status table: implemented / partially implemented / unverified / missing.

Do not modify anything in this phase.

### Phase 2 — Consolidate the one-OS experience
1. Treat the Debian-based NEXUS live desktop as the main demonstrable OS unless the user directs otherwise.
2. Keep the Rust kernel and scheduler work accessible from within that desktop in a way that is clearly explained and tested.
3. Build one NEXUS Lab app with three clear areas:
   - **Lab:** run/inspect scheduler experiments.
   - **Findings:** research findings, charts/tables, methodology, limitations, and citations.
   - **Programs:** launch or manage available task programs/demos.
4. Avoid presenting disconnected ISOs or multiple inconsistent desktops as the final deliverable.
5. Preserve the user's existing working kernel and Linux image configuration.

### Phase 3 — Research integrity
1. Locate the actual research notes, data, scripts, and source references.
2. Verify each displayed metric against its raw output.
3. Clearly distinguish kernel scheduler results from Linux EEVDF/RR comparison results.
4. Include experiment parameters, workload, CPU count/load, time window, fairness/throughput definitions, and known sampling limitations.
5. Do not fabricate references, values, or claims.

### Phase 4 — UI polish
1. Inspect current desktop and app screenshots.
2. Fix overlap, black widget backgrounds, spacing, typography, icon consistency, and dock/menu behavior.
3. Make the System, Scheduler, Lab, Findings, and Programs experiences feel like one product.
4. Verify at the actual VirtualBox display size and in full-screen mode.
5. Capture fresh screenshots only after successful testing.

### Phase 5 — Build and acceptance test
1. Build the ISO using the repository's documented build process.
2. Verify the build output and checksum.
3. Boot the exact newly built ISO in VirtualBox.
4. Test login, desktop, menus, dock, terminal, files, network, APT, Snap Store, lab, findings, programs, and kernel launcher as applicable.
5. Test failure paths and capture logs.
6. Measure boot time on the final ISO if it is a required claim; label the test environment.
7. Ensure the final deployment guide matches the actual ISO and VM settings.

### Phase 6 — Deliverables
- One final NEXUS OS live ISO.
- Checksum file.
- Deployment instructions for VirtualBox.
- Research write-up with verified sources and results.
- Fresh screenshots.
- Updated README and technical docs.
- Git commit(s), with push status clearly reported.

## 8. Guardrails for the coding agent

- Do not start over or replace the project with a generic Linux theme.
- Do not assume QEMU is the primary runtime. Inspect the launcher and explain its role; ask before removing it.
- Do not alter VirtualBox settings or boot configuration without explaining the impact.
- Do not claim a feature works merely because a config or launcher exists.
- Do not use historical checksums as if they describe a newly built ISO.
- Do not mix results from different experiments or environments.
- Do not overwrite the only working ISO or source tree. Make backups before significant changes.
- Make small, reviewable changes and run tests after each meaningful change.
- Ask the user before destructive changes, major architectural changes, or removing existing features.
- Keep the user informed in plain language and provide exact commands to build, run, and test.

---

# Prompt to give Antigravity

Read `PROJECT_HANDOFF.md` and inspect the entire existing repository. This is an existing project; do not rebuild it from scratch.

First, perform **Phase 1 — Audit, no edits** from the handoff:
- inspect the project tree and important source/configuration files;
- inspect Git status, branches, remotes, and recent commits;
- identify the current ISO/build scripts and deployment guide;
- verify which features are implemented, included in the latest ISO, and actually tested;
- identify contradictions or stale documentation.

Do not edit, delete, move, or overwrite files during this audit. Do not assume QEMU is the primary runtime; the user's VM platform is Oracle VirtualBox, and the conversation only mentions QEMU as a possible bundled launcher/development tool. At the end, report a status table and a safe, prioritized plan to deliver one polished NEXUS OS with the integrated Lab, Findings, and Programs experience. Wait for approval before making changes.
