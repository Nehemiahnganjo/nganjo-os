// ══════════════════════════════════════════════════════════════════════════
//  panels/procs.rs — process manager state
//  Reads live data from the shared SystemData snapshot.
// ══════════════════════════════════════════════════════════════════════════

use crate::data::ProcInfo;
use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortBy { Cpu, Mem, Pid, Name }

pub struct ProcPanel {
    pub idx:     usize,
    pub procs:   Vec<ProcInfo>,
    pub sort_by: SortBy,
}

impl ProcPanel {
    pub fn new() -> Self {
        Self { idx: 0, procs: vec![], sort_by: SortBy::Cpu }
    }

    /// Called by the UI render pass with a fresh snapshot.
    pub fn update(&mut self, mut procs: Vec<ProcInfo>) {
        match self.sort_by {
            SortBy::Cpu  => procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap()),
            SortBy::Mem  => procs.sort_by(|a, b| b.mem.partial_cmp(&a.mem).unwrap()),
            SortBy::Pid  => procs.sort_by_key(|p| p.pid),
            SortBy::Name => procs.sort_by(|a, b| a.name.cmp(&b.name)),
        }
        self.procs = procs;
        self.idx = self.idx.min(self.procs.len().saturating_sub(1));
    }

    pub fn move_up(&mut self)   { if self.idx > 0 { self.idx -= 1; } }
    pub fn move_down(&mut self) { if self.idx + 1 < self.procs.len() { self.idx += 1; } }

    pub fn current(&self) -> Option<&ProcInfo> {
        self.procs.get(self.idx)
    }

    pub fn kill_current(&mut self) {
        if let Some(p) = self.procs.get(self.idx) {
            // SIGKILL via `kill` — avoids unsafe signal code
            let _ = Command::new("kill").arg("-9").arg(p.pid.to_string()).output();
            self.procs.remove(self.idx);
            self.idx = self.idx.min(self.procs.len().saturating_sub(1));
        }
    }

    pub fn renice(&self, val: &str) {
        if let Some(p) = self.procs.get(self.idx) {
            let _ = Command::new("renice").arg(val).arg("-p").arg(p.pid.to_string()).output();
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Cpu  => SortBy::Mem,
            SortBy::Mem  => SortBy::Pid,
            SortBy::Pid  => SortBy::Name,
            SortBy::Name => SortBy::Cpu,
        };
    }

    pub fn sort_label(&self) -> &'static str {
        match self.sort_by {
            SortBy::Cpu  => "cpu",
            SortBy::Mem  => "mem",
            SortBy::Pid  => "pid",
            SortBy::Name => "name",
        }
    }

    pub fn proc_info(&self, pid: u32) -> String {
        let path = format!("/proc/{}/status", pid);
        std::fs::read_to_string(&path)
            .map(|s| s.lines().take(20).collect::<Vec<_>>().join("\n"))
            .unwrap_or_else(|_| format!("PID {} — no /proc info", pid))
    }
}
