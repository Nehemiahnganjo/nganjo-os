// ══════════════════════════════════════════════════════════════════════════
//  panels/disk.rs — disk panel state
// ══════════════════════════════════════════════════════════════════════════

use crate::data::DiskInfo;
use std::process::Command;

pub struct DiskPanel {
    pub idx:   usize,
    pub disks: Vec<DiskInfo>,
}

impl DiskPanel {
    pub fn new() -> Self {
        let mut p = Self { idx: 0, disks: vec![] };
        p.load();
        p
    }

    /// Populate from sysinfo — called by the render loop with a fresh snapshot.
    pub fn update(&mut self, disks: Vec<DiskInfo>) {
        self.disks = disks;
        self.idx = self.idx.min(self.disks.len().saturating_sub(1));
    }

    pub fn load(&mut self) {
        // no-op; data comes via update() from the background thread
    }

    pub fn move_up(&mut self)   { if self.idx > 0 { self.idx -= 1; } }
    pub fn move_down(&mut self) { if self.idx + 1 < self.disks.len() { self.idx += 1; } }

    pub fn current(&self) -> Option<&DiskInfo> {
        self.disks.get(self.idx)
    }

    pub fn du_top(&self, mount: &str) -> String {
        Command::new("sh")
            .arg("-c")
            .arg(format!("du -sh {}/* 2>/dev/null | sort -rh | head -20", mount))
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_else(|_| "No data".into())
    }

    pub fn umount(&self, mount: &str) -> String {
        Command::new("pkexec")
            .arg("umount")
            .arg(mount)
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).into_owned();
                let e = String::from_utf8_lossy(&o.stderr).into_owned();
                if !s.is_empty() { s } else if !e.is_empty() { e } else { format!("Unmounted {}", mount) }
            })
            .unwrap_or_else(|e| e.to_string())
    }
}
