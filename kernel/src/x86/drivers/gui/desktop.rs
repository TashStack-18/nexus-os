//! The desktop task: window manager, taskbar, Start menu, desktop icons and
//! the three apps (Terminal, Task Manager, System Info).
//!
//! Every number shown comes from the running kernel: the task list and CPU
//! shares from the scheduler, memory from the heap allocator, the clock from
//! the PC's RTC. The Terminal window is the kernel shell itself.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use super::canvas::{Canvas, Rect, Rgb};
use super::font::Fonts;
use super::input::{Event, Input};
use super::rtc;
use super::super::vga::{self, COLS, ROWS};
use crate::boot::Framebuffer;
use crate::{memory, scheduler, task::TaskState, timer};

// ------------------------------------------------------------------ palette

const TASKBAR: Rgb = 0x111827;
const TASKBAR_LINE: Rgb = 0x2B3446;
const ACCENT: Rgb = 0x2563EB;
const TEXT: Rgb = 0x111827;
const TEXT_2: Rgb = 0x4B5563;
const MUTED: Rgb = 0x9CA3AF;
const WHITE: Rgb = 0xFFFFFF;
const TITLE_ON: Rgb = 0xF3F4F6;
const TITLE_OFF: Rgb = 0xE5E7EB;
const BORDER: Rgb = 0xC7CCD4;
const TERM_BG: Rgb = 0x1C1F26;
const SELECT: Rgb = 0xDBEAFE;
const RED: Rgb = 0xDC2626;
const GREEN: Rgb = 0x16A34A;

/// VGA colour index (0-15) -> RGB, a calm palette for the terminal.
pub fn term_color(idx: u8) -> Rgb {
    const P: [Rgb; 16] = [
        0x1C1F26, 0x5B8DEF, 0x7FB069, 0x4FB3BF, 0xE06C6C, 0xC08AE0, 0xE5B567, 0xD8DCE3,
        0x7F8796, 0x8AB4FF, 0xA6D189, 0x7FD1DB, 0xFF8A80, 0xD8B4FE, 0xF5D08A, 0xFFFFFF,
    ];
    P[(idx & 15) as usize]
}

// ------------------------------------------------------------------ apps and windows

#[derive(Clone, Copy, PartialEq)]
enum App {
    Terminal,
    Tasks,
    About,
}

impl App {
    const ALL: [App; 3] = [App::Terminal, App::Tasks, App::About];
    fn title(self) -> &'static str {
        match self {
            App::Terminal => "Terminal",
            App::Tasks => "Task Manager",
            App::About => "System Info",
        }
    }
    fn tile(self) -> Rgb {
        match self {
            App::Terminal => 0x1F2937,
            App::Tasks => 0x059669,
            App::About => 0x2563EB,
        }
    }
    fn size(self) -> (i32, i32) {
        match self {
            App::Terminal => (COLS as i32 * 8 + 20, ROWS as i32 * 16 + TITLE + 16),
            App::Tasks => (600, 440),
            App::About => (500, 420),
        }
    }
}

const TITLE: i32 = 34;
const BAR: i32 = 48;

struct Window {
    app: App,
    r: Rect,
    open: bool,
    minimized: bool,
}

#[derive(Clone, Copy)]
enum Menu {
    Open(App),
    Run(&'static str),
    Shutdown,
}

const MENU_RUN: [(&str, &str); 5] = [
    ("Say hello", "hello world"),
    ("Busy program (5 s)", "spin 5 &"),
    ("Count prime numbers", "primes 200000"),
    ("User-mode program", "user hello 2"),
    ("Rule-breaking program", "user priv"),
];

struct Cpu {
    last_ticks: u64,
    last_idle: u64,
    per_task: Vec<(usize, u64, u32)>, // tid, cpu_ticks at last sample, share in 0.1%
    total: u32,                      // busy share in 0.1%
    last_sample_us: u64,
}

struct Desktop {
    fb: Framebuffer,
    c: Canvas,
    wallpaper: Vec<u32>,
    fonts: Fonts,
    wins: Vec<Window>, // z-order: last = top
    focus: Option<App>,
    mx: i32,
    my: i32,
    buttons: u8,
    drag: Option<(App, i32, i32)>,
    menu_open: bool,
    selected_icon: Option<App>,
    last_click: (u64, i32, i32),
    selected_tid: Option<usize>,
    grid: Box<[u16; COLS * ROWS]>,
    cursor: usize,
    clock: rtc::DateTime,
    blink: bool,
    cpu: Cpu,
    dirty: bool,
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
        selected_icon: None,
        last_click: (0, -100, -100),
        selected_tid: None,
        grid: alloc::vec![0x0720u16; COLS * ROWS].into_boxed_slice().try_into().expect("grid size"),
        cursor: 0,
        clock: rtc::now(),
        blink: true,
        cpu: Cpu { last_ticks: 0, last_idle: 0, per_task: Vec::new(), total: 0, last_sample_us: 0 },
        dirty: true,
    };
    for app in App::ALL {
        let (ww, wh) = app.size();
        let (x, y) = match app {
            App::Terminal => (150, 40),
            App::Tasks => (w as i32 - ww - 40, 70),
            App::About => (w as i32 - ww - 80, 110),
        };
        d.wins.push(Window { app, r: Rect::new(x, y, ww, wh), open: false, minimized: false });
    }
    d.open(App::Terminal);
    let mut input = Input::new();
    let mut frame = 0u64;
    while !crate::shell::finished() {
        while let Some(ev) = input.poll() {
            d.event(ev);
        }
        if frame % 8 == 0 {
            d.tick();
        }
        if let Some(cur) = vga::snapshot(&mut d.grid) {
            d.cursor = cur;
            d.dirty = true;
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

fn wallpaper(w: usize, h: usize) -> Vec<u32> {
    let mut px = alloc::vec![0u32; w * h];
    let (top, bottom) = (0x1D3F72u32, 0x1F7A7A);
    for y in 0..h {
        let t = (y * 256 / h) as u32;
        let row = super::canvas::mix(top, bottom, t);
        for x in 0..w {
            // a soft light from the upper right
            let dx = (w as i64 - x as i64) as u64;
            let dy = y as u64;
            let d2 = (dx * dx + dy * dy) as u64;
            let r2 = (w as u64 * w as u64) / 2;
            let glow = if d2 < r2 { ((r2 - d2) * 40 / r2) as u32 } else { 0 };
            px[y * w + x] = super::canvas::mix(row, 0x9CC9E6, glow);
        }
    }
    px
}

impl Desktop {
    fn w(&self) -> i32 {
        self.fb.width as i32
    }
    fn h(&self) -> i32 {
        self.fb.height as i32
    }

    fn win(&mut self, app: App) -> &mut Window {
        self.wins.iter_mut().find(|w| w.app == app).unwrap()
    }

    fn open(&mut self, app: App) {
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
                        win.r.x = (mx - gx).clamp(-win.r.w + 80, w - 80);
                        win.r.y = (my - gy).clamp(0, h - BAR - TITLE);
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
        let now = timer::now_us();
        let double = now - self.last_click.0 < 450_000 && (x - self.last_click.1).abs() < 6 && (y - self.last_click.2).abs() < 6;
        self.last_click = (now, x, y);

        // Start menu
        if self.menu_open {
            if let Some(item) = self.menu_hit(x, y) {
                self.menu_open = false;
                match item {
                    Menu::Open(app) => self.open(app),
                    Menu::Run(cmd) => {
                        self.open(App::Terminal);
                        super::push_str(cmd);
                        super::push_key(b'\r');
                    }
                    Menu::Shutdown => {
                        self.open(App::Terminal);
                        super::push_str("exit\r");
                    }
                }
                return;
            }
            // A click anywhere else (including on Start again) closes the menu;
            // a click on another window or the desktop still acts normally.
            let on_start = self.start_rect().contains(x, y);
            if !self.menu_rect().contains(x, y) {
                self.menu_open = false;
            }
            if on_start {
                return;
            }
        }
        // Taskbar
        if y >= self.h() - BAR {
            if self.start_rect().contains(x, y) {
                self.menu_open = !self.menu_open;
                return;
            }
            for (i, app) in self.taskbar_apps().into_iter().enumerate() {
                if self.task_button(i).contains(x, y) {
                    let w = self.win(app);
                    if w.minimized {
                        self.open(app);
                    } else if self.focus == Some(app) {
                        self.minimize(app);
                    } else {
                        self.open(app);
                    }
                    return;
                }
            }
            return;
        }
        // Windows, topmost first
        let hit = self.wins.iter().rev().find(|w| w.open && !w.minimized && w.r.contains(x, y)).map(|w| (w.app, w.r));
        if let Some((app, r)) = hit {
            self.open(app); // raise and focus
            let (close, min) = title_buttons(r);
            if close.contains(x, y) {
                self.close(app);
            } else if min.contains(x, y) {
                self.minimize(app);
            } else if y < r.y + TITLE {
                self.drag = Some((app, x - r.x, y - r.y));
            } else if app == App::Tasks {
                self.tasks_click(r, x, y);
            }
            return;
        }
        // Desktop icons
        self.selected_icon = None;
        for (i, app) in App::ALL.into_iter().enumerate() {
            if icon_rect(i).contains(x, y) {
                self.selected_icon = Some(app);
                if double {
                    self.open(app);
                }
            }
        }
    }

    // -------------------------------------------------------------- periodic

    fn tick(&mut self) {
        let now = rtc::now();
        if (now.hour, now.minute) != (self.clock.hour, self.clock.minute) {
            self.dirty = true;
        }
        self.clock = now;
        // Blink the terminal cursor and refresh live numbers about once a second.
        let us = timer::now_us();
        if us - self.cpu.last_sample_us >= 1_000_000 {
            self.sample_cpu(us);
            self.dirty = true;
        }
        // repaint for the terminal's blinking cursor only when it toggles
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

    // -------------------------------------------------------------- layout

    fn start_rect(&self) -> Rect {
        Rect::new(8, self.h() - BAR + 7, 96, BAR - 14)
    }
    fn taskbar_apps(&self) -> Vec<App> {
        self.wins.iter().filter(|w| w.open).map(|w| w.app).collect::<Vec<_>>().into_iter().fold(Vec::new(), |mut v, a| {
            if !v.contains(&a) {
                v.push(a);
            }
            v
        })
    }
    fn task_button(&self, i: usize) -> Rect {
        Rect::new(116 + i as i32 * 172, self.h() - BAR + 7, 164, BAR - 14)
    }
    fn menu_rect(&self) -> Rect {
        Rect::new(8, self.h() - BAR - 8 - 448, 300, 448)
    }
    fn menu_items(&self) -> Vec<(Rect, Menu, &'static str, Rgb)> {
        let m = self.menu_rect();
        let mut v = Vec::new();
        let mut y = m.y + 68;
        for app in App::ALL {
            v.push((Rect::new(m.x + 8, y, m.w - 16, 36), Menu::Open(app), app.title(), app.tile()));
            y += 36;
        }
        y += 34;
        for (label, cmd) in MENU_RUN {
            v.push((Rect::new(m.x + 8, y, m.w - 16, 32), Menu::Run(cmd), label, 0));
            y += 32;
        }
        y += 14;
        v.push((Rect::new(m.x + 8, y, m.w - 16, 36), Menu::Shutdown, "Shut down", RED));
        v
    }
    fn menu_hit(&self, x: i32, y: i32) -> Option<Menu> {
        self.menu_items().into_iter().find(|(r, ..)| r.contains(x, y)).map(|(_, m, ..)| m)
    }

    // -------------------------------------------------------------- task manager

    fn task_rows(r: Rect) -> (i32, i32) {
        (r.y + TITLE + 96, 26) // first row y, row height
    }
    fn end_button(r: Rect) -> Rect {
        Rect::new(r.x + r.w - 132, r.y + r.h - 46, 116, 32)
    }
    fn tasks_click(&mut self, r: Rect, x: i32, y: i32) {
        if Self::end_button(r).contains(x, y) {
            if let Some(tid) = self.selected_tid {
                let name = scheduler::snapshot().into_iter().find(|t| t.tid == tid).map(|t| t.name);
                if !matches!(name, Some("shell" | "desktop") | None) {
                    scheduler::request_cancel(tid);
                }
            }
            return;
        }
        let (y0, rh) = Self::task_rows(r);
        if y >= y0 {
            let i = ((y - y0) / rh) as usize;
            self.selected_tid = scheduler::snapshot().get(i).map(|t| t.tid);
        }
    }

    // -------------------------------------------------------------- drawing

    fn render(&mut self) {
        self.c.px.copy_from_slice(&self.wallpaper);
        self.draw_icons();
        let order: Vec<App> = self.wins.iter().filter(|w| w.open && !w.minimized).map(|w| w.app).collect();
        for app in order {
            let r = self.wins.iter().find(|w| w.app == app).unwrap().r;
            self.draw_window(app, r);
        }
        self.draw_taskbar();
        if self.menu_open {
            self.draw_menu();
        }
        self.draw_pointer();
    }

    fn draw_icons(&mut self) {
        for (i, app) in App::ALL.into_iter().enumerate() {
            let r = icon_rect(i);
            if self.selected_icon == Some(app) {
                self.c.shade(r, 0xFFFFFF, 48);
            }
            let tile = Rect::new(r.x + (r.w - 52) / 2, r.y + 8, 52, 52);
            self.c.shadow(tile, 12);
            self.c.round(tile, 12, app.tile());
            self.app_glyph(app, tile);
            let label = app.title();
            let lw = Canvas::text_w(label);
            let (lx, ly) = (r.x + (r.w - lw) / 2, r.y + 68);
            self.c.text(&self.fonts.bold, lx + 1, ly + 1, label, 0x0A1A2F);
            self.c.text(&self.fonts.bold, lx, ly, label, WHITE);
        }
    }

    /// The small picture inside an app's tile.
    fn app_glyph(&mut self, app: App, t: Rect) {
        let (cx, cy) = (t.x + t.w / 2, t.y + t.h / 2);
        match app {
            App::Terminal => {
                self.c.text(&self.fonts.bold, cx - 10, cy - 8, ">_", 0x7FD1DB);
            }
            App::Tasks => {
                for (i, hgt) in [10, 22, 15, 28].into_iter().enumerate() {
                    let s = t.w / 52;
                    self.c.fill(Rect::new(t.x + 10 * s + i as i32 * 9 * s, t.y + t.h - 10 * s - hgt * s, 6 * s, hgt * s), WHITE);
                }
            }
            App::About => {
                self.c.round(Rect::new(cx - 3, cy - 13, 6, 6), 3, WHITE);
                self.c.fill(Rect::new(cx - 3, cy - 4, 6, 17), WHITE);
            }
        }
    }

    fn draw_window(&mut self, app: App, r: Rect) {
        let focused = self.focus == Some(app);
        self.c.shadow(r, 10);
        self.c.round(r, 10, if app == App::Terminal { TERM_BG } else { WHITE });
        // title bar
        let tb = Rect::new(r.x, r.y, r.w, TITLE);
        self.c.round(Rect::new(r.x, r.y, r.w, TITLE + 10), 10, if focused { TITLE_ON } else { TITLE_OFF });
        self.c.fill(Rect::new(r.x, r.y + TITLE, r.w, 10), if app == App::Terminal { TERM_BG } else { WHITE });
        self.c.fill(Rect::new(r.x, r.y + TITLE, r.w, 1), BORDER);
        let icon = Rect::new(r.x + 10, r.y + 8, 18, 18);
        self.c.round(icon, 5, app.tile());
        self.c.text(&self.fonts.bold, r.x + 36, tb.y + 9, app.title(), if focused { TEXT } else { TEXT_2 });
        let (close, min) = title_buttons(r);
        let hover_close = close.contains(self.mx, self.my);
        if hover_close {
            self.c.round(close, 6, RED);
        }
        self.c.text(&self.fonts.bold, close.x + 9, close.y + 5, "×", if hover_close { WHITE } else { TEXT_2 });
        if min.contains(self.mx, self.my) {
            self.c.round(min, 6, 0xD1D5DB);
        }
        self.c.text(&self.fonts.bold, min.x + 9, min.y + 3, "_", TEXT_2);
        self.c.outline(r, 10, if focused { 0x9CA3AF } else { BORDER });
        let body = Rect::new(r.x + 1, r.y + TITLE + 1, r.w - 2, r.h - TITLE - 2);
        match app {
            App::Terminal => self.draw_terminal(body),
            App::Tasks => self.draw_tasks(r),
            App::About => self.draw_about(body),
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
        // blinking block cursor while the Terminal has focus
        if self.focus == Some(App::Terminal) && self.blink {
            let (row, col) = ((self.cursor / COLS) as i32, (self.cursor % COLS) as i32);
            self.c.shade(Rect::new(x0 + col * 8, y0 + row * 16 + 13, 8, 3), 0xE6EDF3, 220);
        }
    }

    fn draw_tasks(&mut self, r: Rect) {
        let (x0, mut y) = (r.x + 18, r.y + TITLE + 16);
        let policy = scheduler::run_info().map_or(String::new(), |(p, q, ..)| format!("{} scheduler, {} ms time slice", p, q));
        let tasks = scheduler::snapshot();
        self.c.text(&self.fonts.bold, x0, y, "Running programs", TEXT);
        let cpu = format!("CPU {}%", self.cpu.total / 10);
        let cw = Canvas::text_w(&cpu);
        self.c.text(&self.fonts.bold, r.x + r.w - 18 - cw, y, &cpu, TEXT);
        y += 22;
        self.c.text(&self.fonts.regular, x0, y, &format!("{} tasks · {}", tasks.len(), policy), TEXT_2);
        // CPU bar
        let bar = Rect::new(r.x + r.w - 178, y + 4, 160, 8);
        self.c.round(bar, 4, 0xE5E7EB);
        self.c.round(Rect::new(bar.x, bar.y, (bar.w * self.cpu.total as i32 / 1000).max(4), bar.h), 4, ACCENT);
        y += 30;
        // table header
        let cols = [(0, "PID"), (52, "Name"), (210, "Status"), (330, "CPU"), (410, "CPU time")];
        self.c.fill(Rect::new(r.x + 1, y - 6, r.w - 2, 28), 0xF3F4F6);
        for (dx, h) in cols {
            self.c.text(&self.fonts.bold, x0 + dx, y, h, TEXT_2);
        }
        let (row_y, rh) = Self::task_rows(r);
        let max_rows = ((r.h - (row_y - r.y) - 60) / rh).max(0) as usize;
        for (i, t) in tasks.iter().take(max_rows).enumerate() {
            let ry = row_y + i as i32 * rh;
            if self.selected_tid == Some(t.tid) {
                self.c.fill(Rect::new(r.x + 1, ry - 5, r.w - 2, rh), SELECT);
            } else if i % 2 == 1 {
                self.c.fill(Rect::new(r.x + 1, ry - 5, r.w - 2, rh), 0xFAFAFB);
            }
            let share = self.cpu.per_task.iter().find(|p| p.0 == t.tid).map_or(0, |p| p.2);
            let (status, sc) = match t.state {
                TaskState::Running => ("Running", GREEN),
                TaskState::Ready => ("Waiting", ACCENT),
                TaskState::Blocked => ("Sleeping", TEXT_2),
                TaskState::Terminated => ("Finished", MUTED),
            };
            self.c.text(&self.fonts.regular, x0, ry, &format!("{}", t.tid), TEXT_2);
            self.c.text_fit(&self.fonts.bold, x0 + 52, ry, t.name, TEXT, 150);
            self.c.text(&self.fonts.regular, x0 + 210, ry, status, sc);
            self.c.text(&self.fonts.regular, x0 + 330, ry, &format!("{}.{}%", share / 10, share % 10), TEXT);
            let secs = t.cpu_ticks * timer::tick_us() / 1000;
            self.c.text(&self.fonts.regular, x0 + 410, ry, &format!("{}.{:02} s", secs / 1000, secs % 1000 / 10), TEXT);
        }
        // footer: End task
        let eb = Self::end_button(r);
        let sel_name = self.selected_tid.and_then(|tid| tasks.iter().find(|t| t.tid == tid)).map(|t| t.name);
        let can_end = matches!(sel_name, Some(n) if n != "shell" && n != "desktop");
        self.c.round(eb, 8, if can_end { RED } else { 0xE5E7EB });
        self.c.text(&self.fonts.bold, eb.x + 22, eb.y + 8, "End task", if can_end { WHITE } else { MUTED });
        let hint = match sel_name {
            Some("shell" | "desktop") => "The shell and the desktop keep running.",
            Some(_) => "Asks the program to stop (like kill).",
            None => "Select a program to end it.",
        };
        self.c.text(&self.fonts.regular, x0, eb.y + 8, hint, TEXT_2);
    }

    fn draw_about(&mut self, b: Rect) {
        let (x0, mut y) = (b.x + 24, b.y + 22);
        let logo = Rect::new(x0, y, 48, 48);
        self.c.round(logo, 12, ACCENT);
        self.c.text(&self.fonts.bold, logo.x + 20, logo.y + 16, "N", WHITE);
        self.c.text(&self.fonts.bold, x0 + 64, y + 6, "NEXUS", TEXT);
        self.c.text(&self.fonts.regular, x0 + 64, y + 26, &format!("Micro-OS v{} · written in Rust", env!("CARGO_PKG_VERSION")), TEXT_2);
        y += 74;
        let h = memory::stats();
        let (sw, sh) = super::resolution();
        let up = timer::now_us() / 1_000_000;
        let sched = scheduler::run_info().map_or(String::from("-"), |(p, q, pre, ..)| {
            format!("{}, {} ms slice{}", p, q, if pre { ", preemptive" } else { "" })
        });
        let d = self.clock;
        let rows: [(&str, String); 8] = [
            ("Processor", String::from("x86-64, 1 CPU (kernel in ring 0)")),
            ("Started by", String::from("GRUB (Multiboot2)")),
            ("Screen", format!("{} × {}, 32-bit colour", sw, sh)),
            ("Kernel memory", format!("{} KiB of {} MiB in use", h.bytes_in_use / 1024, h.capacity >> 20)),
            ("Scheduler", sched),
            ("Timer", format!("{} ticks per second", 1_000_000 / timer::tick_us())),
            ("Up time", format!("{}:{:02}:{:02}", up / 3600, up / 60 % 60, up % 60)),
            ("Date", format!("{:02}/{:02}/{} {:02}:{:02}", d.day, d.month, d.year, d.hour, d.minute)),
        ];
        for (k, v) in rows.iter() {
            self.c.text(&self.fonts.regular, x0, y, k, TEXT_2);
            self.c.text_fit(&self.fonts.bold, x0 + 140, y, v, TEXT, b.w - 140 - 48);
            if *k == "Kernel memory" {
                let bar = Rect::new(x0 + 140, y + 20, 200, 6);
                self.c.round(bar, 3, 0xE5E7EB);
                let used = (bar.w as usize * h.bytes_in_use / h.capacity.max(1)).max(4) as i32;
                self.c.round(Rect::new(bar.x, bar.y, used, bar.h), 3, GREEN);
                y += 12;
            }
            y += 30;
        }
    }

    fn draw_taskbar(&mut self) {
        let (w, h) = (self.w(), self.h());
        self.c.fill(Rect::new(0, h - BAR, w, BAR), TASKBAR);
        self.c.fill(Rect::new(0, h - BAR, w, 1), TASKBAR_LINE);
        // Start button
        let s = self.start_rect();
        let hover = s.contains(self.mx, self.my) || self.menu_open;
        self.c.round(s, 8, if hover { 0x3B82F6 } else { ACCENT });
        for (dx, dy) in [(0, 0), (9, 0), (0, 9), (9, 9)] {
            self.c.round(Rect::new(s.x + 12 + dx, s.y + 8 + dy, 7, 7), 2, WHITE);
        }
        self.c.text(&self.fonts.bold, s.x + 40, s.y + 9, "Start", WHITE);
        // open windows
        for (i, app) in self.taskbar_apps().into_iter().enumerate() {
            let b = self.task_button(i);
            let active = self.focus == Some(app);
            let minimized = self.wins.iter().any(|w| w.app == app && w.minimized);
            self.c.round(b, 8, if active { 0x374151 } else if b.contains(self.mx, self.my) { 0x2A3342 } else { 0x1F2937 });
            self.c.round(Rect::new(b.x + 10, b.y + 8, 18, 18), 5, app.tile());
            self.c.text_fit(&self.fonts.regular, b.x + 36, b.y + 9, app.title(), if minimized { MUTED } else { WHITE }, b.w - 44);
            if active {
                self.c.round(Rect::new(b.x + b.w / 2 - 12, b.y + b.h - 3, 24, 3), 1, 0x60A5FA);
            }
        }
        // tray: CPU and clock
        let d = self.clock;
        let time = format!("{:02}:{:02}", d.hour, d.minute);
        let date = format!("{:02}/{:02}/{}", d.day, d.month, d.year);
        let tx = w - 16 - Canvas::text_w(&date);
        self.c.text(&self.fonts.bold, tx + (Canvas::text_w(&date) - Canvas::text_w(&time)) / 2, h - BAR + 7, &time, WHITE);
        self.c.text(&self.fonts.regular, tx, h - BAR + 25, &date, MUTED);
        let cpu = format!("CPU {}%", self.cpu.total / 10);
        let cx = tx - 24 - Canvas::text_w(&cpu);
        self.c.text(&self.fonts.regular, cx, h - BAR + 16, &cpu, 0xD1D5DB);
        let bar = Rect::new(cx, h - BAR + 34, Canvas::text_w(&cpu), 3);
        self.c.fill(bar, 0x374151);
        self.c.fill(Rect::new(bar.x, bar.y, (bar.w * self.cpu.total as i32 / 1000).max(1), 3), 0x60A5FA);
    }

    fn draw_menu(&mut self) {
        let m = self.menu_rect();
        self.c.shadow(m, 12);
        self.c.round(m, 12, WHITE);
        self.c.outline(m, 12, BORDER);
        let logo = Rect::new(m.x + 16, m.y + 16, 36, 36);
        self.c.round(logo, 9, ACCENT);
        self.c.text(&self.fonts.bold, logo.x + 14, logo.y + 10, "N", WHITE);
        self.c.text(&self.fonts.bold, m.x + 62, m.y + 18, "NEXUS", TEXT);
        self.c.text(&self.fonts.regular, m.x + 62, m.y + 36, "Micro-OS · Rust kernel", TEXT_2);
        let items = self.menu_items();
        let run_y = items[3].0.y - 26;
        self.c.fill(Rect::new(m.x + 16, run_y - 6, m.w - 32, 1), 0xE5E7EB);
        self.c.text(&self.fonts.bold, m.x + 18, run_y + 2, "Run a program", MUTED);
        for (r, item, label, tile) in items {
            if r.contains(self.mx, self.my) {
                self.c.round(r, 8, 0xEEF2FF);
            }
            match item {
                Menu::Open(_) => {
                    self.c.round(Rect::new(r.x + 8, r.y + 8, 20, 20), 6, tile);
                    self.c.text(&self.fonts.bold, r.x + 38, r.y + 10, label, TEXT);
                }
                Menu::Run(cmd) => {
                    self.c.text(&self.fonts.regular, r.x + 12, r.y + 8, label, TEXT);
                    let cw = Canvas::text_w(cmd.trim_end_matches(" &"));
                    self.c.text(&self.fonts.regular, r.x + r.w - 10 - cw, r.y + 8, cmd.trim_end_matches(" &"), MUTED);
                }
                Menu::Shutdown => {
                    self.c.fill(Rect::new(m.x + 16, r.y - 8, m.w - 32, 1), 0xE5E7EB);
                    self.c.text(&self.fonts.bold, r.x + 12, r.y + 10, label, tile);
                }
            }
        }
    }

    fn draw_pointer(&mut self) {
        // classic arrow, 12 x 19: 1 = outline, 2 = fill
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
}

fn icon_rect(i: usize) -> Rect {
    Rect::new(20, 24 + i as i32 * 104, 112, 92)
}

fn title_buttons(r: Rect) -> (Rect, Rect) {
    let close = Rect::new(r.x + r.w - 38, r.y + 5, 28, 24);
    let min = Rect::new(r.x + r.w - 70, r.y + 5, 28, 24);
    (close, min)
}

