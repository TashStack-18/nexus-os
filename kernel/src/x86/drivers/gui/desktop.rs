//! The desktop task: wallpaper with a large clock, a menu bar, widgets, a
//! dock and movable windows (Terminal, Programs, Task Manager, Scheduler Lab,
//! Memory, System Info). It runs as an ordinary kernel task, so the scheduler
//! shares the CPU between it, the shell and any programs.
//!
//! Every number shown comes from the running kernel: tasks and CPU shares
//! from the scheduler, memory from the heap allocator, the clock from the
//! PC's RTC. The Terminal window is the kernel shell itself.
//!
//! Drawing is immediate-mode: each frame the whole screen is composed into an
//! off-screen canvas and clickable areas ("hits") are recorded as they are
//! drawn; a click is matched against the hits of the last frame, topmost first.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use microos_core::research::Dist;
use microos_core::sched::PolicyKind;

use super::canvas::{clock_mask, mix, Canvas, Mask, Rect, Rgb};
use super::font::Fonts;
use super::input::{Event, Input};
use super::lab::{bursts_for, Lab};
use super::rtc;
use super::super::vga::{self, COLS, ROWS};
use crate::boot::Framebuffer;
use crate::{memory, scheduler, timer};

// ------------------------------------------------------------------ palette

pub const WHITE: Rgb = 0xF6F8FC;
pub const MUTED: Rgb = 0xA7B2C8;
pub const ACCENT: Rgb = 0x5B8CFF;
pub const TEAL: Rgb = 0x34D1BF;
pub const WARM: Rgb = 0xFFB86B;
pub const RED: Rgb = 0xFF5F57;
pub const YELLOW: Rgb = 0xFEBC2E;
pub const GREEN: Rgb = 0x28C840;
pub const BODY: Rgb = 0xF7F8FA;
pub const INK: Rgb = 0x1B2230;
pub const INK_2: Rgb = 0x667085;
pub const LINE: Rgb = 0xE3E7EE;
const TERM_BG: Rgb = 0x161A22;
const TITLE_ON: Rgb = 0xECEEF3;
const TITLE_OFF: Rgb = 0xF4F5F8;

/// VGA colour index (0-15) -> RGB, a calm palette for the terminal.
pub fn term_color(idx: u8) -> Rgb {
    const P: [Rgb; 16] = [
        0x161A22, 0x5B8DEF, 0x7FB069, 0x4FB3BF, 0xE06C6C, 0xC08AE0, 0xE5B567, 0xD8DCE3,
        0x7F8796, 0x8AB4FF, 0xA6D189, 0x7FD1DB, 0xFF8A80, 0xD8B4FE, 0xF5D08A, 0xFFFFFF,
    ];
    P[(idx & 15) as usize]
}

pub const BAR: i32 = 30;
pub const TITLE: i32 = 36;

// ------------------------------------------------------------------ apps and windows

#[derive(Clone, Copy, PartialEq)]
pub enum App {
    Terminal,
    Programs,
    Tasks,
    Lab,
    Memory,
    Info,
}

impl App {
    pub const DOCK: [App; 6] = [App::Terminal, App::Programs, App::Tasks, App::Lab, App::Memory, App::Info];
    pub fn title(self) -> &'static str {
        match self {
            App::Terminal => "Terminal",
            App::Programs => "Programs",
            App::Tasks => "Task Manager",
            App::Lab => "Scheduler Lab",
            App::Memory => "Memory",
            App::Info => "System Info",
        }
    }
    pub fn size(self) -> (i32, i32) {
        match self {
            App::Terminal => (COLS as i32 * 8 + 20, ROWS as i32 * 16 + TITLE + 14),
            App::Programs => (560, 590),
            App::Tasks => (600, 430),
            App::Lab => (840, 580),
            App::Memory => (600, 470),
            App::Info => (500, 400),
        }
    }
}

pub struct Window {
    pub app: App,
    pub r: Rect,
    pub open: bool,
    pub minimized: bool,
}

#[derive(Clone, Copy)]
pub enum Act {
    Focus(App),
    Open(App),
    Close(App),
    Min(App),
    Policy(PolicyKind),
    Quantum(i32),
    Run(&'static str),
    SelectTask(usize),
    EndTask,
    LabRun,
    LabStop,
    AddJobs(Dist),
    NexusMenu,
    Shutdown,
}

pub struct Hit {
    pub r: Rect,
    pub act: Act,
}

pub struct Cpu {
    last_ticks: u64,
    last_idle: u64,
    per_task: Vec<(usize, u64, u32)>, // tid, cpu_ticks at last sample, share in 0.1%
    pub total: u32,                  // busy share in 0.1%
    last_sample_us: u64,
}

pub struct Desktop {
    pub fb: Framebuffer,
    pub c: Canvas,
    wallpaper: Vec<u32>,
    pub fonts: Fonts,
    pub wins: Vec<Window>, // z-order: last = top
    pub focus: Option<App>,
    pub mx: i32,
    pub my: i32,
    buttons: u8,
    drag: Option<(App, i32, i32)>,
    pub menu_open: bool,
    pub sel_tid: Option<usize>,
    pub grid: Box<[u16; COLS * ROWS]>,
    pub cursor: usize,
    pub clock: rtc::DateTime,
    pub blink: bool,
    pub cpu: Cpu,
    pub lab: Lab,
    pub hits: Vec<Hit>,
    clock_mask: Option<(u8, u8, Mask)>,
    pub dirty: bool,
}

pub fn run(fb: Framebuffer) {
    let (w, h) = (fb.width, fb.height);
    let mut d = Desktop {
        fb,
        c: Canvas::new(w, h),
        wallpaper: wallpaper(w, h),
        fonts: Fonts::load(),
        wins: Vec::new(),
        focus: None,
        mx: w as i32 / 2,
        my: h as i32 / 2,
        buttons: 0,
        drag: None,
        menu_open: false,
        sel_tid: None,
        grid: alloc::vec![0x0720u16; COLS * ROWS].into_boxed_slice().try_into().expect("grid size"),
        cursor: 0,
        clock: rtc::now(),
        blink: true,
        cpu: Cpu { last_ticks: 0, last_idle: 0, per_task: Vec::new(), total: 0, last_sample_us: 0 },
        lab: Lab::new(),
        hits: Vec::new(),
        clock_mask: None,
        dirty: true,
    };
    for (i, app) in App::DOCK.into_iter().enumerate() {
        let (ww, wh) = app.size();
        let x = ((w as i32 - ww) / 2 + (i as i32 - 2) * 34).clamp(10, (w as i32 - ww - 10).max(10));
        let y = (BAR + 26 + (i as i32 % 3) * 26).min(h as i32 - wh - 96).max(BAR + 6);
        d.wins.push(Window { app, r: Rect::new(x, y, ww, wh), open: false, minimized: false });
    }
    let mut input = Input::new();
    let mut frame = 0u64;
    while !crate::shell::finished() {
        while let Some(ev) = input.poll() {
            d.event(ev);
        }
        d.lab.step();
        if frame % 8 == 0 {
            d.tick();
        }
        if let Some(cur) = vga::snapshot(&mut d.grid) {
            d.cursor = cur;
            if d.is_open(App::Terminal) {
                d.dirty = true;
            }
        }
        if d.dirty {
            d.render();
            d.c.present(&d.fb);
            d.dirty = false;
        }
        frame += 1;
        scheduler::sleep_ticks(15);
    }
}

/// An original wallpaper: a deep blue-violet gradient with three soft glows.
pub fn wallpaper(w: usize, h: usize) -> Vec<u32> {
    let mut px = alloc::vec![0u32; w * h];
    let (top, bottom) = (0x0B1230u32, 0x1B2C5E);
    // (centre x, centre y, radius, colour, strength 0..256)
    let orbs: [(i64, i64, i64, u32, u32); 3] = [
        (w as i64 * 20 / 100, h as i64 * 85 / 100, w as i64 * 38 / 100, 0x1FA3A3, 150),
        (w as i64 * 82 / 100, h as i64 * 22 / 100, w as i64 * 34 / 100, 0x7C4DFF, 130),
        (w as i64 * 58 / 100, h as i64 * 105 / 100, w as i64 * 30 / 100, 0xFF7A59, 110),
    ];
    for y in 0..h {
        let base = mix(top, bottom, (y * 256 / h) as u32);
        for x in 0..w {
            let mut c = base;
            for &(ox, oy, r, col, k) in orbs.iter() {
                let (dx, dy) = (x as i64 - ox, y as i64 - oy);
                let (d2, r2) = (dx * dx + dy * dy, r * r);
                if d2 < r2 {
                    let t = ((r2 - d2) * 256 / r2) as u32;
                    let t = (t * t) >> 8;
                    c = mix(c, col, (t * k) >> 8);
                }
            }
            px[y * w + x] = c;
        }
    }
    px
}

impl Desktop {
    pub fn w(&self) -> i32 {
        self.fb.width as i32
    }
    pub fn h(&self) -> i32 {
        self.fb.height as i32
    }

    pub fn is_open(&self, app: App) -> bool {
        self.wins.iter().any(|w| w.app == app && w.open && !w.minimized)
    }

    fn win(&mut self, app: App) -> &mut Window {
        self.wins.iter_mut().find(|w| w.app == app).unwrap()
    }

    pub fn open(&mut self, app: App) {
        let i = self.wins.iter().position(|w| w.app == app).unwrap();
        let mut w = self.wins.remove(i);
        w.open = true;
        w.minimized = false;
        self.wins.push(w);
        self.focus = Some(app);
        self.dirty = true;
    }

    fn close(&mut self, app: App) {
        self.win(app).open = false;
        self.refocus();
    }

    fn minimize(&mut self, app: App) {
        self.win(app).minimized = true;
        self.refocus();
    }

    fn refocus(&mut self) {
        self.focus = self.wins.iter().rev().find(|w| w.open && !w.minimized).map(|w| w.app);
        self.dirty = true;
    }

    pub fn hit(&mut self, r: Rect, act: Act) {
        self.hits.push(Hit { r, act });
    }

    // -------------------------------------------------------------- input

    fn event(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => {
                if k == 0x1B && self.menu_open {
                    self.menu_open = false;
                    self.dirty = true;
                } else if self.focus == Some(App::Terminal) {
                    super::push_key(k);
                }
            }
            Event::Mouse { dx, dy, buttons } => {
                let (ox, oy) = (self.mx, self.my);
                self.mx = (self.mx + dx).clamp(0, self.w() - 1);
                self.my = (self.my + dy).clamp(0, self.h() - 1);
                if (self.mx, self.my) != (ox, oy) {
                    self.dirty = true;
                    if let Some((app, gx, gy)) = self.drag {
                        let (w, h, mx, my) = (self.w(), self.h(), self.mx, self.my);
                        let win = self.win(app);
                        win.r.x = (mx - gx).clamp(-win.r.w + 100, w - 100);
                        win.r.y = (my - gy).clamp(BAR, h - 110);
                    }
                }
                let pressed = buttons & 1 != 0 && self.buttons & 1 == 0;
                let released = buttons & 1 == 0 && self.buttons & 1 != 0;
                self.buttons = buttons;
                if pressed {
                    self.click(self.mx, self.my);
                }
                if released {
                    self.drag = None;
                }
            }
        }
    }

    fn click(&mut self, x: i32, y: i32) {
        self.dirty = true;
        let was_menu = self.menu_open;
        self.menu_open = false;
        let act = self.hits.iter().rev().find(|h| h.r.contains(x, y)).map(|h| h.act);
        match act {
            Some(Act::Focus(app)) => {
                self.open(app);
                let r = self.wins.iter().find(|w| w.app == app).map(|w| w.r).unwrap();
                if y < r.y + TITLE {
                    self.drag = Some((app, x - r.x, y - r.y));
                }
            }
            Some(Act::Open(app)) => {
                // clicking the dock icon of the focused window minimises it
                if self.focus == Some(app) && self.is_open(app) && x > 0 && y >= self.h() - 100 {
                    self.minimize(app);
                } else {
                    self.open(app);
                }
            }
            Some(Act::Close(app)) => self.close(app),
            Some(Act::Min(app)) => self.minimize(app),
            Some(Act::NexusMenu) => self.menu_open = !was_menu,
            Some(Act::Shutdown) => {
                self.open(App::Terminal);
                super::push_str("exit\r");
            }
            Some(Act::Policy(kind)) => {
                if !self.lab.running() {
                    let q = scheduler::run_info().map_or(4, |r| r.1).max(1);
                    scheduler::switch_policy(kind, q, bursts_for(kind), 0);
                }
            }
            Some(Act::Quantum(delta)) => {
                if !self.lab.running() {
                    if let Some((_, q, ..)) = scheduler::run_info() {
                        let kind = super::lab::current_kind();
                        scheduler::switch_policy(kind, (q as i32 + delta).clamp(1, 20) as u32, bursts_for(kind), 0);
                    }
                }
            }
            Some(Act::Run(cmd)) => {
                self.open(App::Terminal);
                super::push_str(cmd);
                super::push_key(b'\r');
            }
            Some(Act::SelectTask(tid)) => self.sel_tid = Some(tid),
            Some(Act::EndTask) => {
                if let Some(tid) = self.sel_tid {
                    let name = scheduler::snapshot().into_iter().find(|t| t.tid == tid).map(|t| t.name);
                    if !matches!(name, Some("shell" | "desktop") | None) {
                        scheduler::request_cancel(tid);
                    }
                }
            }
            Some(Act::LabRun) => self.lab.start(),
            Some(Act::LabStop) => self.lab.stop(),
            Some(Act::AddJobs(dist)) => {
                if !self.lab.running() {
                    self.lab.add_jobs(dist);
                }
            }
            None => {}
        }
    }

    // -------------------------------------------------------------- periodic

    fn tick(&mut self) {
        let now = rtc::now();
        if (now.hour, now.minute) != (self.clock.hour, self.clock.minute) {
            self.dirty = true;
        }
        self.clock = now;
        self.lab.sample();
        let us = timer::now_us();
        if us - self.cpu.last_sample_us >= 1_000_000 {
            self.sample_cpu(us);
            self.dirty = true;
        }
        if self.lab.running() {
            self.dirty = true;
        }
        let phase = (us / 500_000) % 2 == 0;
        if self.focus == Some(App::Terminal) && phase != self.blink {
            self.blink = phase;
            self.dirty = true;
        }
    }

    fn sample_cpu(&mut self, us: u64) {
        let Some((_, _, _, stats, _)) = scheduler::run_info() else { return };
        let dt = stats.ticks.saturating_sub(self.cpu.last_ticks).max(1);
        let didle = stats.idle_ticks.saturating_sub(self.cpu.last_idle);
        if self.cpu.last_ticks != 0 {
            self.cpu.total = (1000 - (didle * 1000 / dt).min(1000)) as u32;
        }
        let mut next = Vec::new();
        for t in scheduler::snapshot() {
            let prev = self.cpu.per_task.iter().find(|p| p.0 == t.tid).map(|p| p.1);
            let share = prev.map_or(0, |p| (t.cpu_ticks.saturating_sub(p) * 1000 / dt).min(1000) as u32);
            next.push((t.tid, t.cpu_ticks, share));
        }
        self.cpu.per_task = next;
        self.cpu.last_ticks = stats.ticks;
        self.cpu.last_idle = stats.idle_ticks;
        self.cpu.last_sample_us = us;
    }

    pub fn share_of(&self, tid: usize) -> u32 {
        self.cpu.per_task.iter().find(|p| p.0 == tid).map_or(0, |p| p.2)
    }

    // -------------------------------------------------------------- text helpers

    pub fn txt(&mut self, x: i32, y: i32, s: &str, c: Rgb) {
        self.c.text(&self.fonts.regular, x, y, s, c);
    }
    pub fn txtb(&mut self, x: i32, y: i32, s: &str, c: Rgb) {
        self.c.text(&self.fonts.bold, x, y, s, c);
    }
    pub fn txt_fit(&mut self, x: i32, y: i32, s: &str, c: Rgb, max_w: i32) {
        self.c.text_fit(&self.fonts.regular, x, y, s, c, max_w);
    }
    pub fn txtb_fit(&mut self, x: i32, y: i32, s: &str, c: Rgb, max_w: i32) {
        self.c.text_fit(&self.fonts.bold, x, y, s, c, max_w);
    }
    /// Right-aligned text ending at `right`.
    pub fn txt_r(&mut self, right: i32, y: i32, s: &str, c: Rgb) {
        let x = right - Canvas::text_w(s);
        self.txt(x, y, s, c);
    }
    pub fn txtb_r(&mut self, right: i32, y: i32, s: &str, c: Rgb) {
        let x = right - Canvas::text_w(s);
        self.txtb(x, y, s, c);
    }
    pub fn txtb_c(&mut self, cx: i32, y: i32, s: &str, c: Rgb) {
        let x = cx - Canvas::text_w(s) / 2;
        self.txtb(x, y, s, c);
    }
    pub fn txt_c(&mut self, cx: i32, y: i32, s: &str, c: Rgb) {
        let x = cx - Canvas::text_w(s) / 2;
        self.txt(x, y, s, c);
    }

    /// A button: filled rounded rectangle with centred label, registers its hit.
    pub fn button(&mut self, r: Rect, label: &str, fill: Rgb, ink: Rgb, act: Act, enabled: bool) {
        let hover = enabled && r.contains(self.mx, self.my);
        let fill = if !enabled { 0xE4E7EC } else if hover { mix(fill, 0xFFFFFF, 40) } else { fill };
        self.c.rrect(r, 8, fill, 256);
        let ink = if enabled { ink } else { 0x98A2B3 };
        self.txtb_c(r.x + r.w / 2, r.y + (r.h - 16) / 2, label, ink);
        if enabled {
            self.hit(r, act);
        }
    }

    pub fn policy_title(kind: PolicyKind) -> &'static str {
        match kind {
            PolicyKind::RoundRobin => "Round Robin",
            PolicyKind::Priority => "Priority",
            PolicyKind::PriorityAging => "Priority + aging",
            PolicyKind::Mlfq => "MLFQ",
            PolicyKind::Fcfs => "FCFS",
            PolicyKind::Sjf => "SJF",
            PolicyKind::Srtf => "SRTF",
            PolicyKind::Hrrn => "HRRN",
        }
    }

    // -------------------------------------------------------------- frame

    fn render(&mut self) {
        self.hits.clear();
        self.c.px.copy_from_slice(&self.wallpaper);
        self.draw_big_clock();
        self.draw_widgets();
        let order: Vec<App> = self.wins.iter().filter(|w| w.open && !w.minimized).map(|w| w.app).collect();
        for app in order {
            let r = self.wins.iter().find(|w| w.app == app).unwrap().r;
            self.draw_window(app, r);
        }
        self.draw_menubar();
        self.draw_dock();
        if self.menu_open {
            self.draw_nexus_menu();
        }
        self.draw_pointer();
    }

    fn draw_big_clock(&mut self) {
        let d = self.clock;
        let cx = self.w() / 2;
        // weekday and date
        const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
        const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
        let date = format!("{}, {} {}", DAYS[weekday(d.year, d.month, d.day)], d.day, MONTHS[(d.month.clamp(1, 12) - 1) as usize]);
        self.txtb_c(cx, BAR + 34, &date, 0xDCE4F5);
        // large time, rendered once per minute
        let key = (d.hour, d.minute);
        if self.clock_mask.as_ref().map_or(true, |m| (m.0, m.1) != key) {
            let s = format!("{:02}:{:02}", d.hour, d.minute);
            self.clock_mask = Some((key.0, key.1, clock_mask(&s, 62, 128, 11.0, 20)));
        }
        let (mw, _) = self.clock_mask.as_ref().map(|m| (m.2.w as i32, m.2.h as i32)).unwrap_or((0, 0));
        let x = cx - mw / 2;
        if let Some((_, _, m)) = self.clock_mask.as_ref() {
            // soft shadow, then the digits
            self.c.blit_mask(x + 3, BAR + 64 + 4, m, 0x000000, 70);
            self.c.blit_mask(x, BAR + 64, m, 0xFFFFFF, 236);
        }
        let up = timer::now_us() / 1_000_000;
        let sub = format!("kernel up {}:{:02}:{:02}", up / 3600, up / 60 % 60, up % 60);
        self.txt_c(cx, BAR + 64 + 128 + 18, &sub, MUTED);
    }

    // -------------------------------------------------------------- widgets

    fn draw_widgets(&mut self) {
        let (w, _h) = (self.w(), self.h());
        // ---- System: rings
        let sys = Rect::new(24, BAR + 18, 252, 176);
        self.c.shadow(sys, 16);
        self.c.glass(sys, 16);
        self.txtb(sys.x + 18, sys.y + 14, "SYSTEM", MUTED);
        let heap = memory::stats();
        let heap_pm = (heap.bytes_in_use * 1000 / heap.capacity.max(1)) as u32;
        let jain = self.lab.live.jain_bp;
        let rings: [(&str, u32, String, Rgb); 3] = [
            ("CPU", self.cpu.total, format!("{}%", self.cpu.total / 10), ACCENT),
            ("Heap", heap_pm.max(8), format!("{}%", heap_pm / 10), TEAL),
            ("Fair", jain.map_or(0, |j| j / 10), super::lab::fmt_j(jain), WARM),
        ];
        for (i, (label, frac, val, col)) in rings.iter().enumerate() {
            let cx = sys.x + 52 + i as i32 * 74;
            let cy = sys.y + 84;
            self.c.ring(cx, cy, 30, 7, *frac, *col, 0x2A3350);
            self.txtb_c(cx, cy - 8, val, WHITE);
            self.txt_c(cx, cy + 40, label, MUTED);
        }
        // ---- Calendar
        let cal = Rect::new(24, sys.y + sys.h + 14, 252, 196);
        self.c.shadow(cal, 16);
        self.c.glass(cal, 16);
        let d = self.clock;
        const MONTHS: [&str; 12] = ["JANUARY", "FEBRUARY", "MARCH", "APRIL", "MAY", "JUNE", "JULY", "AUGUST", "SEPTEMBER", "OCTOBER", "NOVEMBER", "DECEMBER"];
        let title = format!("{} {}", MONTHS[(d.month.clamp(1, 12) - 1) as usize], d.year);
        self.txtb(cal.x + 18, cal.y + 14, &title, MUTED);
        for (i, l) in ["M", "T", "W", "T", "F", "S", "S"].iter().enumerate() {
            self.txtb_c(cal.x + 30 + i as i32 * 32, cal.y + 40, l, 0x8391AD);
        }
        let first = (weekday(d.year, d.month, 1) + 6) % 7; // Monday = 0
        let days = days_in_month(d.year, d.month);
        for day in 1..=days {
            let idx = first as i32 + day as i32 - 1;
            let (col, row) = (idx % 7, idx / 7);
            let (cx, cy) = (cal.x + 30 + col * 32, cal.y + 68 + row * 22);
            let today = day == d.day;
            if today {
                self.c.disc(cx, cy + 8, 11, ACCENT);
            }
            self.txtb_c(cx, cy, &format!("{}", day), if today { 0xFFFFFF } else { 0xD3DBEA });
        }
        // ---- Scheduler panel
        let pw = 268;
        let sch = Rect::new(w - 24 - pw, BAR + 18, pw, 408);
        self.c.shadow(sch, 16);
        self.c.glass(sch, 16);
        self.txtb(sch.x + 18, sch.y + 14, "SCHEDULER", MUTED);
        let info = scheduler::run_info();
        let (pname, q, preempt) = info.as_ref().map_or(("-", 0, false), |i| (i.0, i.1, i.2));
        let kind = super::lab::current_kind();
        self.txtb(sch.x + 18, sch.y + 38, Self::policy_title(kind), WHITE);
        let sub = format!("{} · {}", pname, if preempt { "preemptive" } else { "cooperative" });
        self.txt(sch.x + 18, sch.y + 58, &sub, MUTED);
        let policies = [
            PolicyKind::RoundRobin, PolicyKind::Priority, PolicyKind::PriorityAging, PolicyKind::Mlfq,
            PolicyKind::Fcfs, PolicyKind::Sjf, PolicyKind::Srtf, PolicyKind::Hrrn,
        ];
        let busy = self.lab.running();
        for (i, p) in policies.into_iter().enumerate() {
            let r = Rect::new(sch.x + 16 + (i as i32 % 2) * 118, sch.y + 86 + (i as i32 / 2) * 34, 112, 28);
            let sel = p == kind;
            let hover = !busy && r.contains(self.mx, self.my);
            let fill = if sel { ACCENT } else if hover { 0x2E3A5C } else { 0x1D2742 };
            self.c.rrect(r, 8, fill, if busy && !sel { 120 } else { 256 });
            self.txtb_c(r.x + r.w / 2, r.y + 6, Self::short_name(p), if sel { 0xFFFFFF } else { 0xC9D3E8 });
            if !busy {
                self.hit(r, Act::Policy(p));
            }
        }
        // time slice
        let uses_q = matches!(kind, PolicyKind::RoundRobin | PolicyKind::Priority | PolicyKind::PriorityAging | PolicyKind::Mlfq);
        let qy = sch.y + 232;
        self.txt(sch.x + 18, qy + 6, "Time slice", MUTED);
        if uses_q {
            let minus = Rect::new(sch.x + pw - 130, qy, 28, 28);
            let plus = Rect::new(sch.x + pw - 46, qy, 28, 28);
            for (r, l, dlt) in [(minus, "-", -1), (plus, "+", 1)] {
                let hover = !busy && r.contains(self.mx, self.my);
                self.c.rrect(r, 8, if hover { 0x2E3A5C } else { 0x1D2742 }, 256);
                self.txtb_c(r.x + 14, r.y + 6, l, WHITE);
                if !busy {
                    self.hit(r, Act::Quantum(dlt));
                }
            }
            self.txtb_c(sch.x + pw - 74, qy + 6, &format!("{} ms", q), WHITE);
        } else {
            self.txt_r(sch.x + pw - 18, qy + 6, "runs to completion", MUTED);
        }
        // live numbers
        let l = self.lab.live;
        let rows = [
            ("Context switches", format!("{}/s", l.dispatches_s)),
            ("Preemptions", format!("{}/s", l.preempt_s)),
            ("Jobs finished", format!("{}/s", l.finished_s)),
            ("Fairness (Jain)", super::lab::fmt_j(l.jain_bp)),
        ];
        for (i, (k, v)) in rows.iter().enumerate() {
            let y = sch.y + 276 + i as i32 * 22;
            self.txt(sch.x + 18, y, k, MUTED);
            self.txtb_r(sch.x + pw - 18, y, v, WHITE);
        }
        let open = Rect::new(sch.x + 16, sch.y + 372, pw - 32, 22);
        let hover = open.contains(self.mx, self.my);
        self.txtb_c(open.x + open.w / 2, open.y + 3, "Open Scheduler Lab", if hover { WHITE } else { 0x9DB7FF });
        self.hit(open, Act::Open(App::Lab));
    }

    fn short_name(p: PolicyKind) -> &'static str {
        match p {
            PolicyKind::RoundRobin => "Round Robin",
            PolicyKind::Priority => "Priority",
            PolicyKind::PriorityAging => "Prio+aging",
            PolicyKind::Mlfq => "MLFQ",
            PolicyKind::Fcfs => "FCFS",
            PolicyKind::Sjf => "SJF",
            PolicyKind::Srtf => "SRTF",
            PolicyKind::Hrrn => "HRRN",
        }
    }

    // -------------------------------------------------------------- chrome

    fn draw_menubar(&mut self) {
        let w = self.w();
        self.c.shade(Rect::new(0, 0, w, BAR), 0x070B18, 170);
        self.c.shade(Rect::new(0, BAR - 1, w, 1), 0xFFFFFF, 22);
        // NEXUS menu
        let n = Rect::new(8, 3, 92, BAR - 6);
        if n.contains(self.mx, self.my) || self.menu_open {
            self.c.rrect(n, 6, 0xFFFFFF, 30);
        }
        self.c.rrect(Rect::new(14, 7, 16, 16), 5, ACCENT, 256);
        self.txtb(17, 7, "N", 0xFFFFFF);
        self.txtb(38, 7, "NEXUS", WHITE);
        self.hit(n, Act::NexusMenu);
        // centre pill geometry first, so the menu words can stop before it
        let pill_text = {
            let info = scheduler::run_info();
            let (pname, q) = info.as_ref().map_or(("-", 0), |i| (i.0, i.1));
            format!("{} · {} ms slice · {} Hz timer", pname, q, 1_000_000 / timer::tick_us().max(1))
        };
        let pill_w = Canvas::text_w(&pill_text) + 40;
        let pill_x = (w - pill_w) / 2;
        // app menu words
        let mut x = 118;
        let focus_title = self.focus.map(|a| a.title());
        for app in [App::Terminal, App::Programs, App::Tasks, App::Lab, App::Memory] {
            let label = app.title();
            let lw = Canvas::text_w(label);
            if x + lw + 16 > pill_x - 8 {
                break;
            }
            let r = Rect::new(x - 8, 3, lw + 16, BAR - 6);
            let active = focus_title == Some(label);
            if r.contains(self.mx, self.my) {
                self.c.rrect(r, 6, 0xFFFFFF, 30);
            }
            if active {
                self.txtb(x, 7, label, WHITE);
            } else {
                self.txt(x, 7, label, 0xC7D0E4);
            }
            self.hit(r, Act::Open(app));
            x += lw + 28;
        }
        // centre pill: live scheduler state
        let pill = pill_text;
        let pr = Rect::new(pill_x, 4, pill_w, BAR - 8);
        self.c.rrect(pr, (BAR - 8) / 2, 0xFFFFFF, 26);
        self.c.disc(pr.x + 14, pr.y + (pr.h) / 2, 3, if self.lab.running() { WARM } else { GREEN });
        self.txt(pr.x + 26, 7, &pill, WHITE);
        self.hit(pr, Act::Open(App::Lab));
        // right side: tick, cpu, clock
        let d = self.clock;
        let clock = format!("{:02}:{:02}", d.hour, d.minute);
        self.txtb_r(w - 14, 7, &clock, WHITE);
        let cpu = format!("CPU {}%", self.cpu.total / 10);
        let cx = w - 14 - Canvas::text_w(&clock) - 22;
        self.txt_r(cx, 7, &cpu, 0xC7D0E4);
        let tick = format!("tick {}", timer::ticks());
        self.txt_r(cx - Canvas::text_w(&cpu) - 22, 7, &tick, MUTED);
    }

    fn draw_nexus_menu(&mut self) {
        let r = Rect::new(8, BAR + 4, 232, 96);
        self.c.shadow(r, 12);
        self.c.rrect(r, 12, 0x12182B, 238);
        self.c.outline(r, 12, 0xFFFFFF, 36);
        let items: [(&str, Act); 2] = [("About NEXUS", Act::Open(App::Info)), ("Shut down", Act::Shutdown)];
        for (i, (label, act)) in items.into_iter().enumerate() {
            let ir = Rect::new(r.x + 8, r.y + 8 + i as i32 * 40, r.w - 16, 36);
            if ir.contains(self.mx, self.my) {
                self.c.rrect(ir, 8, ACCENT, 256);
            }
            let col = if matches!(act, Act::Shutdown) && !ir.contains(self.mx, self.my) { 0xFF8A80 } else { WHITE };
            self.txtb(ir.x + 14, ir.y + 10, label, col);
            self.hit(ir, act);
        }
    }

    fn draw_dock(&mut self) {
        let (w, h) = (self.w(), self.h());
        let n = App::DOCK.len() as i32;
        let (isz, gap) = (56, 14);
        let dw = n * isz + (n + 1) * gap;
        let dock = Rect::new((w - dw) / 2, h - 86, dw, 76);
        self.c.shadow(dock, 24);
        self.c.rrect(dock, 24, 0x0A1022, 150);
        self.c.outline(dock, 24, 0xFFFFFF, 40);
        for (i, app) in App::DOCK.into_iter().enumerate() {
            let slot = Rect::new(dock.x + gap + i as i32 * (isz + gap), dock.y + 10, isz, isz);
            let hover = slot.contains(self.mx, self.my);
            let grow = if hover { 8 } else { 0 };
            let ir = Rect::new(slot.x - grow / 2, slot.y - grow, isz + grow, isz + grow);
            self.draw_app_icon(app, ir);
            if self.wins.iter().any(|w| w.app == app && w.open) {
                self.c.disc(slot.x + isz / 2, dock.y + dock.h - 8, 2, WHITE);
            }
            if hover {
                let label = app.title();
                let lw = Canvas::text_w(label) + 20;
                let tr = Rect::new(slot.x + isz / 2 - lw / 2, dock.y - 36, lw, 26);
                self.c.rrect(tr, 8, 0x0A1022, 230);
                self.c.outline(tr, 8, 0xFFFFFF, 36);
                self.txtb(tr.x + 10, tr.y + 5, label, WHITE);
            }
            self.hit(slot, Act::Open(app));
        }
    }

    pub fn draw_app_icon(&mut self, app: App, r: Rect) {
        let (c1, c2) = match app {
            App::Terminal => (0x2B3342, 0x151A24),
            App::Programs => (0x6E8BFF, 0x4562E0),
            App::Tasks => (0x39D98A, 0x13A662),
            App::Lab => (0xFFB86B, 0xF0833A),
            App::Memory => (0x34D1BF, 0x17A6A0),
            App::Info => (0x9B8CFF, 0x6A58E0),
        };
        let rad = r.w / 4;
        self.c.rrect(r, rad, c2, 256);
        self.c.rrect(Rect::new(r.x, r.y, r.w, r.h * 6 / 10), rad, c1, 256);
        self.c.rrect(Rect::new(r.x, r.y + r.h / 3, r.w, r.h - r.h / 3), rad, c2, 256);
        self.c.outline(r, rad, 0xFFFFFF, 40);
        let (cx, cy, u) = (r.x + r.w / 2, r.y + r.h / 2, r.w / 14);
        match app {
            App::Terminal => {
                self.c.line((cx - 5 * u) as f32, (cy - 3 * u) as f32, (cx - 1 * u) as f32, cy as f32, 1.6, TEAL);
                self.c.line((cx - 1 * u) as f32, cy as f32, (cx - 5 * u) as f32, (cy + 3 * u) as f32, 1.6, TEAL);
                self.c.line((cx + 1 * u) as f32, (cy + 3 * u) as f32, (cx + 5 * u) as f32, (cy + 3 * u) as f32, 1.6, WHITE);
            }
            App::Programs => {
                for (dx, dy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                    self.c.rrect(Rect::new(cx + dx * 3 * u - 2 * u + if dx > 0 { u / 2 } else { -u / 2 }, cy + dy * 3 * u - 2 * u + if dy > 0 { u / 2 } else { -u / 2 }, 4 * u, 4 * u), u, WHITE, 256);
                }
            }
            App::Tasks => {
                for (i, hgt) in [4, 7, 5, 9].into_iter().enumerate() {
                    let x = cx - 6 * u + i as i32 * 3 * u;
                    self.c.rrect(Rect::new(x, cy + 5 * u - hgt * u, 2 * u, hgt * u), u / 2, WHITE, 256);
                }
            }
            App::Lab => {
                self.c.line((cx - 5 * u) as f32, (cy - 5 * u) as f32, (cx - 5 * u) as f32, (cy + 5 * u) as f32, 1.0, WHITE);
                self.c.line((cx - 5 * u) as f32, (cy + 5 * u) as f32, (cx + 5 * u) as f32, (cy + 5 * u) as f32, 1.0, WHITE);
                for (dx, dy, col) in [(-1, 2, 0xFFFFFF), (1, -2, 0xFFFFFF), (3, -3, 0xFFFFFF), (-3, 3, 0xFFFFFF)] {
                    self.c.disc(cx + dx * u, cy + dy * u, (u * 3 / 4).max(2), col);
                }
            }
            App::Memory => {
                for i in 0..3 {
                    for j in 0..3 {
                        let a = if (i + j) % 2 == 0 { 256 } else { 150 };
                        self.c.rrect(Rect::new(cx - 6 * u + i * 4 * u, cy - 6 * u + j * 4 * u, 3 * u, 3 * u), u / 2, WHITE, a);
                    }
                }
            }
            App::Info => {
                self.c.disc(cx, cy - 4 * u, (u * 3 / 2).max(2), WHITE);
                self.c.rrect(Rect::new(cx - u, cy - u, 2 * u, 6 * u), u / 2, WHITE, 256);
            }
        }
    }

    fn draw_pointer(&mut self) {
        // classic arrow, 1 = outline, 2 = fill
        const ARROW: [&[u8]; 19] = [
            b"1", b"11", b"121", b"1221", b"12221", b"122221", b"1222221", b"12222221", b"122222221",
            b"1222222221", b"12222221111", b"1222121", b"122112 21", b"121  1221", b"11   1221", b"1     1221",
            b"      1221", b"       11", b"",
        ];
        for (dy, row) in ARROW.iter().enumerate() {
            for (dx, &p) in row.iter().enumerate() {
                let c = match p {
                    b'1' => 0x000000,
                    b'2' => 0xFFFFFF,
                    _ => continue,
                };
                self.c.fill(Rect::new(self.mx + dx as i32, self.my + dy as i32, 1, 1), c);
            }
        }
    }

    // -------------------------------------------------------------- windows

    fn draw_window(&mut self, app: App, r: Rect) {
        let focused = self.focus == Some(app);
        let dark = app == App::Terminal;
        self.c.shadow(r, 14);
        self.c.round(r, 14, if dark { TERM_BG } else { BODY });
        // title bar
        self.c.rrect(Rect::new(r.x, r.y, r.w, TITLE + 14), 14, if focused { TITLE_ON } else { TITLE_OFF }, 256);
        self.c.rrect(Rect::new(r.x, r.y + TITLE, r.w, 14), 0, if dark { TERM_BG } else { BODY }, 256);
        self.c.fill(Rect::new(r.x, r.y + TITLE, r.w, 1), if dark { 0x2A3140 } else { LINE });
        self.c.outline(r, 14, if dark { 0xFFFFFF } else { 0x000000 }, if focused { 60 } else { 34 });
        // everything inside this window is clickable as "focus this window"
        self.hit(r, Act::Focus(app));
        // traffic lights
        let (close, min) = (Rect::new(r.x + 14, r.y + 12, 13, 13), Rect::new(r.x + 34, r.y + 12, 13, 13));
        let hov = Rect::new(r.x + 8, r.y + 6, 48, 24).contains(self.mx, self.my);
        self.c.disc(close.x + 6, close.y + 6, 6, if focused || hov { RED } else { 0xC9CDD6 });
        self.c.disc(min.x + 6, min.y + 6, 6, if focused || hov { YELLOW } else { 0xC9CDD6 });
        self.c.disc(r.x + 60 + 6, r.y + 18, 6, if focused || hov { 0x9AA4B5 } else { 0xDADDE3 });
        if hov {
            self.txtb(close.x + 2, close.y - 2, "×", 0x7A1D18);
            self.txtb(min.x + 2, min.y - 3, "–", 0x7A5A06);
        }
        self.hit(Rect::new(close.x - 3, close.y - 3, 19, 19), Act::Close(app));
        self.hit(Rect::new(min.x - 3, min.y - 3, 19, 19), Act::Min(app));
        // title
        let t = app.title();
        self.txtb_c(r.x + r.w / 2, r.y + 10, t, if focused { INK } else { INK_2 });
        let body = Rect::new(r.x + 1, r.y + TITLE + 1, r.w - 2, r.h - TITLE - 2);
        match app {
            App::Terminal => self.draw_terminal(body),
            App::Programs => self.draw_programs(r),
            App::Tasks => self.draw_tasks(r),
            App::Lab => self.draw_lab(r),
            App::Memory => self.draw_memory(r),
            App::Info => self.draw_info(r),
        }
    }

    fn draw_terminal(&mut self, b: Rect) {
        let (x0, y0) = (b.x + 9, b.y + 7);
        for i in 0..COLS * ROWS {
            let cell = self.grid[i];
            let (row, col) = ((i / COLS) as i32, (i % COLS) as i32);
            let bg = (cell >> 12) as u8 & 0x07;
            if bg != 0 {
                self.c.fill(Rect::new(x0 + col * 8, y0 + row * 16, 8, 16), term_color(bg));
            }
            let ch = vga::cp437_to_char((cell & 0xFF) as u8);
            if ch != ' ' {
                let fg = ((cell >> 8) & 0x0F) as u8;
                let font = if fg >= 8 && fg != 8 { &self.fonts.bold } else { &self.fonts.regular };
                self.c.glyph(font, x0 + col * 8, y0 + row * 16, ch, term_color(fg));
            }
        }
        if self.focus == Some(App::Terminal) && self.blink {
            let (row, col) = ((self.cursor / COLS) as i32, (self.cursor % COLS) as i32);
            self.c.shade(Rect::new(x0 + col * 8, y0 + row * 16 + 13, 8, 3), 0xE6EDF3, 220);
        }
    }
}

/// Day of the week, 0 = Sunday (Sakamoto's method).
pub fn weekday(year: u16, month: u8, day: u8) -> usize {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year as i32;
    let m = month.clamp(1, 12) as usize;
    if m < 3 {
        y -= 1;
    }
    ((y + y / 4 - y / 100 + y / 400 + T[m - 1] + day as i32) % 7) as usize
}

pub fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

/// Split `s` into lines of at most `max` characters, at spaces.
pub fn wrap(s: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > max {
            lines.push(core::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}
