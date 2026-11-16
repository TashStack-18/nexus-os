//! Scheduler Lab: live scheduling measurements and a small version of the R3
//! study ("Fairness vs Throughput in CPU Scheduling") that runs on this very
//! kernel.
//!
//! * **Live numbers** (every 250 ms): dispatches and preemptions per second,
//!   jobs finished per second, and Jain's fairness index over the CPU time the
//!   programs received in the last interval.
//! * **Experiment**: for each of the six R3 policies it switches the real
//!   scheduler, starts a bimodal workload at offered load 1.1 as real kernel
//!   tasks, measures throughput (jobs finished per 100 ms) and the mean Jain
//!   index over 250 ms intervals, then cancels the leftovers and moves on. The
//!   same workload (same seed) is used for every policy, with R3's modelled
//!   0.2 ms context-switch cost.

use alloc::string::String;
use alloc::vec::Vec;

use microos_core::research::{generate, Dist, Job};
use microos_core::sched::{PolicyKind, DEFAULT_PRIO};

use crate::scheduler::{self, BurstInfo, SpawnOpts};
use crate::task::TaskState;
use crate::timer;

/// Length of the measurement window per policy, in ticks (ms).
pub const WINDOW: u64 = 5000;
const INTERVAL: u64 = 250;
const LOAD_PCT: u64 = 110;
const SEED: u64 = 7;
const CS_COST_US: u64 = 200;

pub const POLICIES: [(&str, PolicyKind, u32); 6] = [
    ("FCFS", PolicyKind::Fcfs, 0),
    ("SJF", PolicyKind::Sjf, 0),
    ("SRTF", PolicyKind::Srtf, 0),
    ("HRRN", PolicyKind::Hrrn, 0),
    ("RR q=2", PolicyKind::RoundRobin, 2),
    ("RR q=8", PolicyKind::RoundRobin, 8),
];

fn is_system(name: &str) -> bool {
    matches!(name, "desktop" | "shell" | "monitor")
}

/// Jain's index of `xs` in basis points (10000 = perfectly fair), or None if
/// nobody received any CPU time.
pub fn jain_bp(xs: &[u64]) -> Option<u32> {
    let sum: u64 = xs.iter().sum();
    if xs.is_empty() || sum == 0 {
        return None;
    }
    let sq: u128 = xs.iter().map(|&x| x as u128 * x as u128).sum();
    Some(((sum as u128 * sum as u128 * 10_000) / (xs.len() as u128 * sq)) as u32)
}

#[derive(Clone)]
pub struct Row {
    pub label: &'static str,
    /// Jobs finished per 100 ms, times 100.
    pub thr_x100: u32,
    pub jain_bp: Option<u32>,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Setup,
    Measure,
    Drain,
    Pause,
}

struct Exp {
    idx: usize,
    phase: Phase,
    t0: u64,
    done0: usize,
    jain_sum: u64,
    jain_n: u32,
    prev: Vec<(usize, u64, bool)>,
    next_interval: u64,
}

#[derive(Default, Clone, Copy)]
pub struct Live {
    pub dispatches_s: u32,
    pub preempt_s: u32,
    pub finished_s: u32,
    pub jain_bp: Option<u32>,
}

pub struct Lab {
    pub live: Live,
    prev_cpu: Vec<(usize, u64, bool)>,
    last: (u64, u64, u64, usize, u64), // dispatches, preemptions, ticks, finished, tick of sample
    ring: [u32; 8],
    ring_n: usize,
    ring_pos: usize,
    exp: Option<Exp>,
    pub rows: Vec<Row>,
    pub note: String,
    saved: (PolicyKind, u32),
    seed: u64,
}

impl Lab {
    pub fn new() -> Lab {
        Lab {
            live: Live::default(),
            prev_cpu: Vec::new(),
            last: (0, 0, 0, 0, 0),
            ring: [0; 8],
            ring_n: 0,
            ring_pos: 0,
            exp: None,
            rows: Vec::new(),
            note: String::new(),
            saved: (PolicyKind::RoundRobin, 4),
            seed: 100,
        }
    }

    pub fn running(&self) -> bool {
        self.exp.is_some()
    }

    /// 0..=100 progress of the experiment.
    pub fn progress(&self) -> u32 {
        match &self.exp {
            None => 0,
            Some(e) => ((e.idx as u32 * 100) + if e.phase == Phase::Measure { ((timer::ticks() - e.t0) * 100 / WINDOW).min(100) as u32 } else if e.phase == Phase::Setup { 0 } else { 100 }) / POLICIES.len() as u32,
        }
    }

    pub fn current_label(&self) -> &'static str {
        self.exp.as_ref().map_or("", |e| POLICIES[e.idx.min(POLICIES.len() - 1)].0)
    }

    // ---------------------------------------------------------------- live

    /// Sample the scheduler every ~250 ms (call often; it rate-limits itself).
    pub fn sample(&mut self) {
        let now = timer::ticks();
        if now - self.last.4 < INTERVAL {
            return;
        }
        let Some((_, _, _, stats, _)) = scheduler::run_info() else { return };
        let dt = (now - self.last.4).max(1);
        let finished = scheduler::finished_count();
        if self.last.4 != 0 {
            self.live.dispatches_s = (stats.dispatches.saturating_sub(self.last.0) * 1000 / dt) as u32;
            self.live.preempt_s = (stats.preemptions.saturating_sub(self.last.1) * 1000 / dt) as u32;
            self.live.finished_s = (finished.saturating_sub(self.last.3) as u64 * 1000 / dt) as u32;
        }
        self.last = (stats.dispatches, stats.preemptions, stats.ticks, finished, now);

        // Jain over the programs present for the whole interval.
        let tasks = scheduler::snapshot();
        // A program is "competing" once it has arrived: it is runnable, or it has run.
        let cur: Vec<(usize, u64, bool)> = tasks
            .iter()
            .filter(|t| !is_system(t.name))
            .map(|t| (t.tid, t.cpu_ticks, t.state != TaskState::Blocked || t.cpu_ticks > 0))
            .collect();
        let xs: Vec<u64> = cur
            .iter()
            .filter_map(|&(tid, c, _)| self.prev_cpu.iter().find(|p| p.0 == tid && p.2).map(|p| c.saturating_sub(p.1)))
            .collect();
        if let Some(j) = jain_bp(&xs) {
            self.ring[self.ring_pos] = j;
            self.ring_pos = (self.ring_pos + 1) % self.ring.len();
            self.ring_n = (self.ring_n + 1).min(self.ring.len());
        } else if cur.is_empty() {
            self.ring_n = 0;
        }
        self.live.jain_bp = (self.ring_n > 0).then(|| self.ring[..self.ring_n.min(self.ring.len())].iter().sum::<u32>() / self.ring_n as u32);
        self.prev_cpu = cur;
    }

    /// Start about 20 real jobs from a workload under the current policy.
    pub fn add_jobs(&mut self, dist: Dist) {
        self.seed += 1;
        let jobs = generate(dist, 70, 500, self.seed);
        spawn_jobs(&jobs);
    }

    // ---------------------------------------------------------------- experiment

    pub fn start(&mut self) {
        if self.exp.is_some() {
            return;
        }
        self.rows.clear();
        self.note.clear();
        if let Some((_, q, ..)) = scheduler::run_info() {
            self.saved.1 = q;
        }
        self.saved.0 = current_kind();
        self.exp = Some(Exp { idx: 0, phase: Phase::Setup, t0: timer::ticks(), done0: 0, jain_sum: 0, jain_n: 0, prev: Vec::new(), next_interval: 0 });
    }

    pub fn stop(&mut self) {
        if self.exp.take().is_some() {
            self.cancel_jobs();
            scheduler::switch_policy(self.saved.0, self.saved.1, bursts_for(self.saved.0), 0);
            self.note = String::from("Experiment stopped.");
        }
    }

    fn cancel_jobs(&self) {
        for t in scheduler::snapshot() {
            if t.name == "job" {
                scheduler::request_cancel(t.tid);
            }
        }
    }

    /// Advance the experiment (call every frame).
    pub fn step(&mut self) {
        let now = timer::ticks();
        let Some(mut e) = self.exp.take() else { return };
        match e.phase {
            Phase::Setup => {
                let (_, kind, q) = POLICIES[e.idx];
                scheduler::switch_policy(kind, q.max(1), BurstInfo::Oracle, CS_COST_US);
                let jobs = generate(Dist::Bimodal, LOAD_PCT, WINDOW, SEED);
                spawn_jobs(&jobs);
                e.done0 = scheduler::finished_count();
                e.t0 = timer::ticks();
                e.next_interval = e.t0 + INTERVAL;
                e.jain_sum = 0;
                e.jain_n = 0;
                e.prev.clear();
                e.phase = Phase::Measure;
            }
            Phase::Measure => {
                if now >= e.next_interval {
                    e.next_interval += INTERVAL;
                    let tasks = scheduler::snapshot();
                    // As in R3: only jobs that had arrived before the interval began.
                    let cur: Vec<(usize, u64, bool)> = tasks
                        .iter()
                        .filter(|t| t.name == "job")
                        .map(|t| (t.tid, t.cpu_ticks, t.state != TaskState::Blocked || t.cpu_ticks > 0))
                        .collect();
                    let xs: Vec<u64> = cur
                        .iter()
                        .filter_map(|&(tid, c, _)| e.prev.iter().find(|p| p.0 == tid && p.2).map(|p| c.saturating_sub(p.1)))
                        .collect();
                    if let Some(j) = jain_bp(&xs) {
                        e.jain_sum += j as u64;
                        e.jain_n += 1;
                    }
                    e.prev = cur;
                }
                if now - e.t0 >= WINDOW {
                    let done = scheduler::finished_count().saturating_sub(e.done0) as u32;
                    self.rows.push(Row {
                        label: POLICIES[e.idx].0,
                        thr_x100: (done as u64 * 100 * 100 / WINDOW) as u32,
                        jain_bp: (e.jain_n > 0).then(|| (e.jain_sum / e.jain_n as u64) as u32),
                    });
                    self.cancel_jobs();
                    e.t0 = now;
                    e.phase = Phase::Drain;
                }
            }
            Phase::Drain => {
                let left = scheduler::snapshot().iter().any(|t| t.name == "job");
                if !left || now - e.t0 > 4000 {
                    e.t0 = now;
                    e.phase = Phase::Pause;
                }
            }
            Phase::Pause => {
                if now - e.t0 >= 150 {
                    e.idx += 1;
                    if e.idx >= POLICIES.len() {
                        scheduler::switch_policy(self.saved.0, self.saved.1, bursts_for(self.saved.0), 0);
                        self.note = self.verdict();
                        return; // experiment finished: leave `exp` empty
                    }
                    e.phase = Phase::Setup;
                }
            }
        }
        self.exp = Some(e);
    }

    /// One-line reading of the result, like the paper's conclusion.
    fn verdict(&self) -> String {
        use alloc::format;
        let best = self.rows.iter().max_by_key(|r| r.thr_x100);
        let fair = self.rows.iter().filter(|r| r.jain_bp.is_some()).max_by_key(|r| r.jain_bp.unwrap_or(0));
        match (best, fair) {
            (Some(b), Some(f)) if b.label != f.label => format!(
                "Most jobs finished: {} ({}.{:02} per 100 ms, fairness {}). Fairest: {} (fairness {}). Maximum throughput did not give fair sharing.",
                b.label, b.thr_x100 / 100, b.thr_x100 % 100, fmt_j(b.jain_bp), f.label, fmt_j(f.jain_bp)
            ),
            (Some(b), _) => format!("Most jobs finished: {} ({}.{:02} per 100 ms, fairness {}).", b.label, b.thr_x100 / 100, b.thr_x100 % 100, fmt_j(b.jain_bp)),
            _ => String::from("No result."),
        }
    }
}

pub fn fmt_j(j: Option<u32>) -> String {
    use alloc::format;
    match j {
        Some(v) => format!("{}.{:02}", v / 10_000, v % 10_000 / 100),
        None => String::from("n/a"),
    }
}

pub fn current_kind() -> PolicyKind {
    match scheduler::run_info().map(|r| r.0) {
        Some("PRIO") => PolicyKind::Priority,
        Some("PRIO+AGING") => PolicyKind::PriorityAging,
        Some("MLFQ") => PolicyKind::Mlfq,
        Some("FCFS") => PolicyKind::Fcfs,
        Some("SJF") => PolicyKind::Sjf,
        Some("SRTF") => PolicyKind::Srtf,
        Some("HRRN") => PolicyKind::Hrrn,
        _ => PolicyKind::RoundRobin,
    }
}

pub fn bursts_for(kind: PolicyKind) -> BurstInfo {
    if kind.needs_bursts() {
        BurstInfo::Predicted
    } else {
        BurstInfo::Oracle
    }
}

/// Start each job as a real kernel task that burns exactly its burst of CPU
/// time (and stops early if cancelled).
fn spawn_jobs(jobs: &[Job]) {
    for j in jobs {
        let burst_us = j.burst as u64 * 1000;
        let opts = SpawnOpts { prio: DEFAULT_PRIO, delay_ticks: j.arrival + 1, burst_ticks: j.burst };
        let _ = scheduler::spawn("job", opts, move || {
            let start = scheduler::current_cpu_us();
            while scheduler::current_cpu_us() - start < burst_us {
                if scheduler::cancel_requested() {
                    break;
                }
                scheduler::workload::spin(200);
            }
        });
    }
}
