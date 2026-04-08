// ══════════════════════════════════════════════════════════════════════════
//  data.rs — all system queries via `sysinfo` (zero subprocess overhead)
//  Refreshed every 2 s on a background thread; UI reads a snapshot under
//  a Mutex so the render loop is never blocked.
// ══════════════════════════════════════════════════════════════════════════

use bytesize::ByteSize;
use chrono::{DateTime, Local};
use sysinfo::{
    CpuExt, DiskExt, NetworkExt, NetworksExt, ProcessExt, System, SystemExt, UserExt,
};

// ── Process snapshot ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ProcInfo {
    pub pid:    u32,
    pub name:   String,
    pub cpu:    f32,   // percent
    pub mem:    f32,   // percent
    pub status: String,
    pub user:   String,
}

// ── Network interface snapshot ────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct NetIface {
    pub name:      String,
    pub ip:        String,
    pub up:        bool,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
}

// ── Disk snapshot ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub device:    String,
    pub mount:     String,
    pub fstype:    String,
    pub total:     u64,
    pub used:      u64,
    pub available: u64,
    pub pct:       f64,
}

impl DiskInfo {
    pub fn pct_bar(&self, width: usize) -> String {
        bar(self.pct as f32, width)
    }
}

// ── Main snapshot struct ──────────────────────────────────────────────────

pub struct SystemData {
    sys:      System,
    pub ts:   DateTime<Local>,

    pub cpu_pct:  f32,
    pub mem_pct:  f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk_pct: f32,

    pub uptime_secs: u64,

    pub procs:  Vec<ProcInfo>,
    pub disks:  Vec<DiskInfo>,
    pub ifaces: Vec<NetIface>,
}

impl SystemData {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let mut d = Self {
            sys,
            ts: Local::now(),
            cpu_pct: 0.0,
            mem_pct: 0.0,
            mem_used: 0,
            mem_total: 0,
            disk_pct: 0.0,
            uptime_secs: 0,
            procs: vec![],
            disks: vec![],
            ifaces: vec![],
        };
        d.refresh();
        d
    }

    /// Called every 2 s from the background thread.
    pub fn refresh(&mut self) {
        self.sys.refresh_all();
        self.ts = Local::now();

        // CPU — average across all logical cores
        let cpus = self.sys.cpus();
        self.cpu_pct = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };

        // Memory
        self.mem_total = self.sys.total_memory();
        self.mem_used  = self.sys.used_memory();
        self.mem_pct   = if self.mem_total > 0 {
            self.mem_used as f32 / self.mem_total as f32 * 100.0
        } else {
            0.0
        };

        // Uptime
        self.uptime_secs = System::uptime();

        // Disks
        self.disks = self.sys.disks().iter().map(|d| {
            let total = d.total_space();
            let avail = d.available_space();
            let used  = total.saturating_sub(avail);
            let pct   = if total > 0 { used as f64 / total as f64 * 100.0 } else { 0.0 };
            // root disk for header bar
            if d.mount_point().to_string_lossy() == "/" {
                self.disk_pct = pct as f32;
            }
            DiskInfo {
                device: d.name().to_string_lossy().to_string(),
                mount:  d.mount_point().to_string_lossy().to_string(),
                fstype: d.file_system().to_string_lossy().to_string(),
                total, used, available: avail, pct,
            }
        }).collect();

        // Processes
        let total_mem = self.mem_total as f32;
        let mut procs: Vec<ProcInfo> = self.sys.processes().iter().map(|(pid, p)| {
            ProcInfo {
                pid:    pid.as_u32(),
                name:   p.name().to_string(),
                cpu:    p.cpu_usage(),
                mem:    if total_mem > 0.0 { p.memory() as f32 / total_mem * 100.0 } else { 0.0 },
                status: format!("{:?}", p.status()),
                user:   p.user_id()
                         .and_then(|uid| self.sys.get_user_by_id(uid))
                         .map(|u| u.name().to_string())
                         .unwrap_or_default(),
            }
        }).collect();
        procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(200);
        self.procs = procs;

        // Network
        self.ifaces = self.sys.networks().iter().map(|(name, net)| {
            NetIface {
                name:       name.clone(),
                ip:         String::new(), // sysinfo doesn't expose IP; use `if_addrs` crate for that
                up:         net.received() > 0 || net.transmitted() > 0,
                bytes_sent: net.total_transmitted(),
                bytes_recv: net.total_received(),
            }
        }).filter(|i| i.name != "lo").take(4).collect();
    }

    pub fn uptime_str(&self) -> String {
        let h = self.uptime_secs / 3600;
        let m = (self.uptime_secs % 3600) / 60;
        format!("{}h {}m", h, m)
    }

    pub fn mem_str(&self) -> String {
        format!(
            "{} / {}",
            ByteSize(self.mem_used),
            ByteSize(self.mem_total)
        )
    }
}

// ── Shared bar helper (also used by UI) ──────────────────────────────────

pub fn bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty  = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
