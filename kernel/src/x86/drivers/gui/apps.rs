//! The window contents: Programs, Task Manager, Scheduler Lab, Memory and
//! System Info. All numbers come from the running kernel.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use microos_core::research::Dist;

use super::canvas::{mix, Rect};
use super::desktop::*;
use super::lab::{fmt_j, POLICIES};
use crate::task::TaskState;
use crate::{memory, programs, scheduler, timer};

fn preset(name: &str) -> &'static str {
    match name {
        "hello" => "hello world",
        "counter" => "counter 5 200 &",
        "primes" => "primes 200000",
        "fib" => "fib 40",
        "spin" => "spin 5 &",
        "memtest" => "memtest 1024",
        "pingpong" => "pingpong 10 100 &",
        "user" => "user hello 2",
        "template" => "template 3",
        _ => "help",
    }
}

impl Desktop {
    // -------------------------------------------------------------- Programs

    pub fn draw_programs(&mut self, r: Rect) {
        let x0 = r.x + 24;
        let mut y = r.y + TITLE + 16;
        self.txt(x0, y, "Programs written in Rust. Each one runs as its own task.", INK_2);
        y += 34;
        let mut rows: Vec<(String, &'static str, &'static str)> = programs::PROGRAMS
            .iter()
            .filter(|p| p.name != "template")
            .map(|p| (String::from(p.name), p.about, preset(p.name)))
            .collect();
        rows.push((String::from("user priv"), "a user-mode program tries a privileged instruction; the kernel stops it", "user priv"));
        rows.push((String::from("user badptr"), "a user-mode program passes a bad pointer; the kernel refuses it", "user badptr"));
        for (name, about, cmd) in rows {
            if y + 44 > r.y + r.h - 10 {
                break;
            }
            let row = Rect::new(r.x + 14, y - 6, r.w - 28, 44);
            if row.contains(self.mx, self.my) {
                self.c.rrect(row, 10, 0xEEF1F6, 256);
            }
            self.txtb(x0, y, &name, INK);
            self.txt_fit(x0, y + 18, about, INK_2, r.w - 24 - 24 - 90);
            self.button(Rect::new(r.x + r.w - 24 - 64, y + 3, 64, 28), "Run", ACCENT, 0xFFFFFF, Act::Run(cmd), true);
            y += 46;
        }
    }

    // -------------------------------------------------------------- Task Manager

    pub fn draw_tasks(&mut self, r: Rect) {
        let x0 = r.x + 24;
        let y = r.y + TITLE + 18;
        let tasks = scheduler::snapshot();
        let info = scheduler::run_info();
        let (pname, q) = info.as_ref().map_or(("-", 0), |i| (i.0, i.1));
        self.txtb(x0, y, "Running programs", INK);
        let cpu = format!("CPU {}%", self.cpu.total / 10);
        self.txtb_r(r.x + r.w - 24, y, &cpu, INK);
        self.txt(x0, y + 22, &format!("{} tasks · {} scheduler, {} ms time slice", tasks.len(), pname, q), INK_2);
        let bar = Rect::new(r.x + r.w - 24 - 160, y + 26, 160, 8);
        self.c.rrect(bar, 4, 0xE3E7EE, 256);
        self.c.rrect(Rect::new(bar.x, bar.y, (bar.w * self.cpu.total as i32 / 1000).max(8), bar.h), 4, ACCENT, 256);
        let hy = y + 54;
        self.c.fill(Rect::new(r.x + 1, hy - 6, r.w - 2, 28), 0xEEF1F6);
        for (dx, h) in [(0, "PID"), (56, "Name"), (214, "Status"), (334, "CPU"), (414, "CPU time")] {
            self.txtb(x0 + dx, hy, h, INK_2);
        }
        let row0 = hy + 30;
        let max_rows = ((r.y + r.h - 70 - row0) / 28).max(0) as usize;
        for (i, t) in tasks.iter().take(max_rows).enumerate() {
            let ry = row0 + i as i32 * 28;
            let row = Rect::new(r.x + 1, ry - 6, r.w - 2, 28);
            if self.sel_tid == Some(t.tid) {
                self.c.fill(row, 0xDCE7FF);
            } else if i % 2 == 1 {
                self.c.fill(row, 0xFAFBFC);
            }
            self.hit(row, Act::SelectTask(t.tid));
            let share = self.share_of(t.tid);
            let (status, sc) = match t.state {
                TaskState::Running => ("Running", 0x13A662),
                TaskState::Ready => ("Waiting", ACCENT),
                TaskState::Blocked => ("Sleeping", INK_2),
                TaskState::Terminated => ("Finished", 0x98A2B3),
            };
            self.txt(x0, ry, &format!("{}", t.tid), INK_2);
            self.txtb_fit(x0 + 56, ry, t.name, INK, 150);
            self.txt(x0 + 214, ry, status, sc);
            self.txt(x0 + 334, ry, &format!("{}.{}%", share / 10, share % 10), INK);
            let secs = t.cpu_ticks * timer::tick_us() / 1000;
            self.txt(x0 + 414, ry, &format!("{}.{:02} s", secs / 1000, secs % 1000 / 10), INK);
        }
        let sel_name = self.sel_tid.and_then(|tid| tasks.iter().find(|t| t.tid == tid)).map(|t| t.name);
        let can_end = matches!(sel_name, Some(n) if n != "shell" && n != "desktop");
        let hint = match sel_name {
            Some("shell" | "desktop") => "The shell and the desktop keep running.",
            Some(_) => "Asks the program to stop (like kill).",
            None => "Select a program to end it.",
        };
        self.txt(x0, r.y + r.h - 40, hint, INK_2);
        self.button(Rect::new(r.x + r.w - 24 - 116, r.y + r.h - 50, 116, 32), "End task", RED, 0xFFFFFF, Act::EndTask, can_end);
    }

    // -------------------------------------------------------------- Memory

    pub fn draw_memory(&mut self, r: Rect) {
        let x0 = r.x + 24;
        let mut y = r.y + TITLE + 16;
        let h = memory::stats();
        self.txtb(x0, y, "Kernel heap", INK);
        self.txtb_r(r.x + r.w - 24, y, &format!("{} of {} MiB", fmt_kib(h.bytes_in_use), h.capacity >> 20), INK);
        self.txt(x0, y + 22, &format!("Each square is 1/128 of the {} MiB heap; darker means fuller.", h.capacity >> 20), INK_2);
        y += 52;
        let mut occ = [0u8; 128];
        memory::occupancy(&mut occ);
        let (cell, gap) = (28, 5);
        let gx = r.x + (r.w - (16 * (cell + gap) - gap)) / 2;
        for (i, &o) in occ.iter().enumerate() {
            let c = mix(0xE3E8F2, ACCENT, o as u32 * 256 / 255);
            let (cx, cy) = (gx + (i as i32 % 16) * (cell + gap), y + (i as i32 / 16) * (cell + gap));
            self.c.rrect(Rect::new(cx, cy, cell, cell), 6, c, 256);
        }
        y += 8 * (cell + gap) + 14;
        let frag = if h.total_free > 0 { 100 - (h.largest_free * 100 / h.total_free) } else { 0 };
        let rows = [
            ("In use", format!("{} KiB", h.bytes_in_use / 1024)),
            ("Peak", format!("{} KiB", h.peak_bytes / 1024)),
            ("Free blocks", format!("{}", h.free_blocks)),
            ("Largest free block", format!("{} MiB", h.largest_free >> 20)),
            ("Fragmentation", format!("{}%", frag)),
            ("Allocations / frees", format!("{} / {}", h.allocs, h.frees)),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let (col, row) = (i as i32 % 2, i as i32 / 2);
            let (kx, ky) = (x0 + col * 280, y + row * 24);
            self.txt(kx, ky, k, INK_2);
            self.txtb_r(kx + 252, ky, v, INK);
        }
    }

    // -------------------------------------------------------------- System Info

    pub fn draw_info(&mut self, r: Rect) {
        let x0 = r.x + 28;
        let mut y = r.y + TITLE + 24;
        let logo = Rect::new(x0, y, 56, 56);
        self.c.rrect(logo, 16, ACCENT, 256);
        self.txtb(logo.x + 24, logo.y + 20, "N", 0xFFFFFF);
        self.txtb(x0 + 74, y + 6, "NEXUS", INK);
        self.txt(x0 + 74, y + 28, &format!("Micro-OS v{} · written in Rust", env!("CARGO_PKG_VERSION")), INK_2);
        y += 84;
        let h = memory::stats();
        let (sw, sh) = super::resolution();
        let up = timer::now_us() / 1_000_000;
        let sched = scheduler::run_info().map_or(String::from("-"), |(p, q, pre, ..)| format!("{}, {} ms slice{}", p, q, if pre { ", preemptive" } else { "" }));
        let d = self.clock;
        let rows: [(&str, String); 8] = [
            ("Processor", String::from("x86-64, 1 CPU (kernel in ring 0)")),
            ("Started by", String::from("GRUB (Multiboot2)")),
            ("Screen", format!("{} × {}, 32-bit colour", sw, sh)),
            ("Kernel memory", format!("{} KiB of {} MiB in use", h.bytes_in_use / 1024, h.capacity >> 20)),
            ("Scheduler", sched),
            ("Timer", format!("{} ticks per second", 1_000_000 / timer::tick_us().max(1))),
            ("Up time", format!("{}:{:02}:{:02}", up / 3600, up / 60 % 60, up % 60)),
            ("Date", format!("{:02}/{:02}/{} {:02}:{:02}", d.day, d.month, d.year, d.hour, d.minute)),
        ];
        for (k, v) in rows.iter() {
            self.txt(x0, y, k, INK_2);
            self.txtb_fit(x0 + 140, y, v, INK, r.w - 140 - 56);
            y += 30;
        }
    }

    // -------------------------------------------------------------- Scheduler Lab

    pub fn draw_lab(&mut self, r: Rect) {
        let top = r.y + TITLE + 14;
        let left = Rect::new(r.x + 16, top, 304, r.h - TITLE - 28);
        let right = Rect::new(r.x + 336, top, r.w - 336 - 16, r.h - TITLE - 28);
        for p in [left, right] {
            self.c.rrect(p, 12, 0xFFFFFF, 256);
            self.c.outline(p, 12, 0x000000, 24);
        }
        let busy = self.lab.running();

        // ---- left: live
        let (x0, mut y) = (left.x + 18, left.y + 16);
        self.txtb(x0, y, "Right now", INK);
        y += 26;
        let info = scheduler::run_info();
        let (pname, q) = info.as_ref().map_or(("-", 0), |i| (i.0, i.1));
        self.txt(x0, y, &format!("{} · {} ms time slice", pname, q), INK_2);
        y += 30;
        let l = self.lab.live;
        let rows = [
            ("Context switches", format!("{}/s", l.dispatches_s)),
            ("Preemptions", format!("{}/s", l.preempt_s)),
            ("Jobs finished", format!("{}/s", l.finished_s)),
        ];
        for (k, v) in rows.iter() {
            self.txt(x0, y, k, INK_2);
            self.txtb_r(left.x + left.w - 18, y, v, INK);
            y += 24;
        }
        y += 6;
        self.txt(x0, y, "Fairness (Jain)", INK_2);
        self.txtb_r(left.x + left.w - 18, y, &fmt_j(l.jain_bp), INK);
        let bar = Rect::new(x0, y + 24, left.w - 36, 8);
        self.c.rrect(bar, 4, 0xE3E7EE, 256);
        if let Some(j) = l.jain_bp {
            self.c.rrect(Rect::new(bar.x, bar.y, (bar.w * j as i32 / 10_000).max(8), bar.h), 4, WARM, 256);
        }
        y += 52;
        self.txt(x0, y, "Add real jobs to the system:", INK_2);
        y += 26;
        self.button(Rect::new(x0, y, left.w - 36, 32), "Add uniform jobs", 0xEEF1F6, INK, Act::AddJobs(Dist::Uniform), !busy);
        y += 40;
        self.button(Rect::new(x0, y, left.w - 36, 32), "Add bimodal jobs", 0xEEF1F6, INK, Act::AddJobs(Dist::Bimodal), !busy);
        y += 52;
        let note = "Switch the scheduler on the desktop (right side), add jobs, and watch fairness and throughput change as the real kernel runs them.";
        for line in wrap(note, 36) {
            self.txt(x0, y, &line, INK_2);
            y += 18;
        }

        // ---- right: experiment
        let (x0, mut y) = (right.x + 18, right.y + 16);
        self.txtb(x0, y, "Fairness vs throughput", INK);
        y += 22;
        self.txt(x0, y, "Does maximum throughput give fair sharing?", INK_2);
        y += 30;
        if busy {
            self.button(Rect::new(x0, y, 130, 32), "Stop", RED, 0xFFFFFF, Act::LabStop, true);
            let pr = Rect::new(x0 + 146, y + 11, right.w - 36 - 146, 10);
            self.c.rrect(pr, 5, 0xE3E7EE, 256);
            self.c.rrect(Rect::new(pr.x, pr.y, (pr.w * self.lab.progress() as i32 / 100).max(10), pr.h), 5, WARM, 256);
            let lbl = format!("Running {}", self.lab.current_label());
            self.txt(pr.x, pr.y - 20, &lbl, INK);
        } else {
            self.button(Rect::new(x0, y, 150, 32), "Run experiment", ACCENT, 0xFFFFFF, Act::LabRun, true);
            self.txt(x0 + 166, y + 8, "about 40 seconds", INK_2);
        }
        y += 46;

        // scatter: throughput (x) against fairness (y)
        let plot = Rect::new(x0 + 40, y, right.w - 36 - 40 - 8, 120);
        self.c.rrect(plot, 8, 0xF6F8FB, 256);
        for i in 0..=4 {
            let gy = plot.y + 10 + (plot.h - 20) * i / 4;
            self.c.fill(Rect::new(plot.x, gy, plot.w, 1), 0xE6EAF1);
            let lbl = ["1.0", "0.75", "0.5", "0.25", "0"][i as usize];
            self.txt_r(plot.x - 6, gy - 8, lbl, INK_2);
        }
        self.txt_c(plot.x + plot.w / 2, plot.y + plot.h + 6, "jobs finished per 100 ms", INK_2);
        let rows = self.lab.rows.clone();
        if rows.is_empty() {
            self.txt_c(plot.x + plot.w / 2, plot.y + plot.h / 2 - 8, if busy { "measuring..." } else { "no result yet" }, 0x98A2B3);
        } else {
            let lo = rows.iter().map(|r| r.thr_x100).min().unwrap_or(0).saturating_sub(40) as i32;
            let hi = rows.iter().map(|r| r.thr_x100).max().unwrap_or(0) as i32 + 40;
            let span = (hi - lo).max(1);
            for (i, row) in rows.iter().enumerate() {
                let Some(j) = row.jain_bp else { continue };
                let px = plot.x + 20 + ((row.thr_x100 as i32 - lo) * (plot.w - 70) / span);
                let py = plot.y + 10 + (plot.h - 20) - ((plot.h - 20) * j as i32 / 10_000);
                let col = policy_color(i);
                self.c.disc(px, py, 6, col);
                self.txtb(px + 9, py - 8, CHART_LABELS[i % 6], col);
            }
        }
        y += 120 + 30;

        // table
        self.c.fill(Rect::new(right.x + 8, y - 4, right.w - 16, 24), 0xEEF1F6);
        self.txtb(x0, y, "Policy", INK_2);
        self.txtb(x0 + 150, y, "Jobs per 100 ms", INK_2);
        self.txtb_r(right.x + right.w - 18, y, "Fairness", INK_2);
        y += 26;
        for (i, row) in rows.iter().enumerate() {
            self.c.disc(x0 + 5, y + 7, 5, policy_color(i));
            self.txtb(x0 + 18, y, row.label, INK);
            self.txt(x0 + 150, y, &format!("{}.{:02}", row.thr_x100 / 100, row.thr_x100 % 100), INK);
            self.txtb_r(right.x + right.w - 18, y, &fmt_j(row.jain_bp), INK);
            y += 22;
        }
        y += 10;
        if !self.lab.note.is_empty() {
            let note = self.lab.note.clone();
            for line in wrap(&note, (right.w as usize - 36) / 8).into_iter().take(3) {
                self.txt(x0, y, &line, INK);
                y += 18;
            }
        } else {
            self.txt(x0, y, "Same jobs for every policy: bimodal, offered load 1.1,", INK_2);
            self.txt(x0, y + 18, "0.2 ms modelled switch cost, as in the R3 study.", INK_2);
        }
    }
}

const CHART_LABELS: [&str; 6] = ["FCFS", "SJF", "SRTF", "HRRN", "RR2", "RR8"];

fn fmt_kib(bytes: usize) -> String {
    let kib = bytes / 1024;
    if kib >= 1024 {
        format!("{}.{}", kib / 1024, kib % 1024 * 10 / 1024)
    } else {
        format!("0.{:02}", kib * 100 / 1024)
    }
}

fn policy_color(i: usize) -> u32 {
    [0x7C8BA1, WARM, 0xFF6B6B, 0xB28DFF, TEAL, ACCENT][i % POLICIES.len()]
}
