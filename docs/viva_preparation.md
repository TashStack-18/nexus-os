# NEXUS Micro-OS — Technical Viva Defense & CA-6 Compliance Guide

**Author**: Tashvi Adhlakha (25BTRCL271)  
**Project**: NEXUS OS / Micro-OS (CA-6 Operating Systems Engineering)  

---

## 1. Kernel Boot Process
- **RISC-V Path**: QEMU loads OpenSBI firmware in Machine mode (`M-mode`), which configures physical memory and transfers control to `_start` in `kernel/src/boot/entry.S` in Supervisor mode (`S-mode`). `entry.S` initializes the kernel stack pointer (`sp`) and branches to Rust entry point `kmain()` in `kernel/src/main.rs`.
- **x86-64 Path**: GRUB loads the kernel ELF via Multiboot2. Execution starts at `_start` in `kernel/src/x86/boot.S` (32-bit protected mode), enables PAE, sets up initial 4-level page tables with 2 MB huge pages, enables Long Mode (IA32_EFER.LME), and jumps to 64-bit `long_mode_start` which invokes `kmain()`.

## 2. Task Representation
- Defined in `kernel/src/task/mod.rs` as `TaskControlBlock` (TCB).
- Fields:
  - `id`: Unique Task ID (`usize`).
  - `state`: Explicit 4-state lifecycle (`TaskState::Ready`, `Running`, `Blocked`, `Terminated`).
  - `priority`: Static priority integer (0..=7).
  - `context`: CPU register state (`Context` struct holding callee-saved registers: `ra`, `sp`, `s0-s11` for RISC-V; `rsp`, `rbp`, `rbx`, `r12-r15` for x86-64).
  - `stack`: Dedicated page-aligned kernel execution stack (`task::stack::Stack`).
  - `metrics`: CPU runtime cycles, dispatches, preemptions, wait time, and arrival timestamps.

## 3. Ready Queue
- Implemented in `kernel/src/scheduler/mod.rs`.
- Dual-queue design:
  - FIFO queue (`VecDeque<usize>`) for Round Robin / FCFS.
  - Multi-level priority queues (`[VecDeque<usize>; 8]`) for MLFQ and priority scheduling.
- Provides `O(1)` enqueue (`push_back`), `O(1)` task selection (`pop_front`), and clean state transitions (`READY -> RUNNING -> READY` on quantum expiry).

## 4. Round Robin Scheduling
- Operates on actual kernel tasks with configurable time quantum $q$ (default $q=8$ ms / ticks).
- When a task runs, the timer interrupt decrements its remaining quantum slice.
- When slice reaches zero, `schedule()` saves the current task context via assembly `switch_to`, moves the task to the back of the ready queue, and dispatches the head task.
- Guarantees $O(1)$ dispatching overhead and near-perfect Jain's fairness index ($J \approx 0.99$ under $q=2$ ms, $J=0.96$ under $q=8$ ms).

## 5. Timer-Driven Scheduling
- Architecture-specific periodic timer:
  - **RISC-V**: Uses SBI `set_timer` extension to schedule timer interrupts every 1,000 µs (1 ms).
  - **x86-64**: Programmable Interval Timer (PIT, Intel 8254) channel 0 configured in Mode 3 (Square Wave Generator) delivering IRQ0 interrupts at 1,000 Hz.
- Trap handler (`trap::trap_handler` / `x86::trap`) increments `ticks` counter and calls `scheduler::on_tick()`.
- **Preemptive vs Cooperative**: If preemption is enabled, when a task's quantum expires on a tick, `switch_to` preempts it immediately. If cooperative, tasks run until explicit `yield_now()`.

## 6. Synchronization
- Three primary primitives in `kernel/src/sync/`:
  - `SpinLock<T>`: Busy-wait mutual exclusion with atomic `compare_exchange_weak` and PAUSE hints.
  - `IrqSafeSpinLock<T>`: Disables CPU interrupts before acquiring the lock to prevent interrupt-handler deadlocks, restoring interrupt flags on release.
  - `Mutex<T>`: Blocking mutex. When contested, places contending tasks into the `BLOCKED` state on a wait-list, yielding the CPU rather than burning CPU cycles.
  - `Semaphore`: Counting semaphore supporting `wait()` and `signal()` with FIFO wait queues.

## 7. Memory Allocation
- First-Fit Free-List Allocator implemented in `kernel/src/memory/mod.rs`.
- Memory managed as a linked chain of free blocks with boundary tags / block headers.
- Allocation searches for the first block with `size >= requested`. If larger, splits into allocated block and residual free block.
- Deallocation coalesces adjacent free blocks to prevent external heap fragmentation.
- Robust boundary checks: rejects invalid pointers, double-frees, and cleanly handles out-of-memory (OOM) requests.

## 8. Unsafe Rust Usage & Justification
- Pure Rust safe abstractions encapsulate necessary hardware primitives:
  - **Context Switch** (`switch.S`): Assembly register save/restore requires `extern "C"` pointers. Invariant: Valid stack pointers and 16-byte alignment.
  - **MMIO Registers** (`volatile_read` / `volatile_write`): Required for 16550 UART and VGA framebuffer. Invariant: Fixed physical base addresses mapped identity or in supervisor paging.
  - **CPU Interrupt Flags**: `arch::intr_on()` / `arch::intr_off()`. Invariant: Called inside critical section wrappers with RAII restoration guards.

## 9. CA-4 Research Specialization Connection
- Directly connects empirical research on CPU scheduling fairness vs throughput to the microkernel:
  - **H1 (Overload Trade-off)**: Verified that under $\rho=1.10$ overload, SJF achieves maximum throughput (7.02 jobs/100ms) but starves long tasks ($J=0.39$), whereas Round Robin achieves $J=0.99$.
  - **H3 (Quantum Sweep)**: Proved context-switch overhead dominates at $q=1$ ms (19.6% CPU lost), while $q=128$ ms collapses into FCFS ($J=0.19$). Optimal sweet spot identified at $q=8$ ms ($J=0.96$, 6.65 jobs/100ms).

## 10. Linux vs Micro-OS Architectural Comparison
| Aspect | Linux Kernel (Desktop) | NEXUS Micro-OS (Kernel) |
| :--- | :--- | :--- |
| **Task Struct** | `task_struct` (>8 KB, 200+ fields, namespaces, namespaces) | Compact `TCB` (128 bytes, clean 4-state lifecycle) |
| **Scheduling** | EEVDF (Earliest Eligible Virtual Deadline First) | Round Robin + Pluggable Policies (SJF, SRTF, MLFQ) |
| **Memory** | Multi-level Buddy Allocator + SLUB + Demand Paging | First-fit Free-List Allocator with header coalescing |
| **Codebase** | ~35 million lines of C, monolithic | ~8,000 lines of pure Rust, `#![no_std]`, zero C code |

## 11. Testing Methodology
- Automated in-kernel test harness (`scenarios::run`) executing 68 distinct assertions:
  - **TC1**: Minimum 2-task Round Robin validation.
  - **TC2**: Multi-task scaling (3–5 tasks).
  - **TC3**: Boundary conditions (quantum $q=1$, 0-cycle jobs, single-byte allocs).
  - **TC4**: Stress workload (rapid allocation churn, queue flooding).
  - **TC5**: Controlled failure injection (invalid pointers, double frees, deliberate page faults).
  - **TC6**: Repeatability verification over multiple random seed runs.

## 12. Known Limitations & Future Work
- Educational microkernel currently runs single-core (`smp=1`); multi-core SMP scheduling with per-core runqueues is planned for next iteration.
- Paging is currently identity-mapped for kernel space; user-space hardware MMU page table separation with TLB shootdowns is simulated via software context separation.
