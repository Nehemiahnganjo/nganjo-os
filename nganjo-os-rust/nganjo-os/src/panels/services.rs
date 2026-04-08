// ══════════════════════════════════════════════════════════════════════════
//  panels/services.rs — systemd service manager
// ══════════════════════════════════════════════════════════════════════════

use std::process::Command;

#[derive(Clone, Debug)]
pub struct ServiceEntry {
    pub name:   String,
    pub active: String,
    pub sub:    String,
}

pub struct ServicePanel {
    pub idx:      usize,
    pub services: Vec<ServiceEntry>,
}

impl ServicePanel {
    pub fn new() -> Self {
        let mut p = Self { idx: 0, services: vec![] };
        p.load();
        p
    }

    pub fn load(&mut self) {
        let out = Command::new("systemctl")
            .args(["list-units", "--type=service", "--all",
                   "--no-pager", "--no-legend", "--plain"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();

        self.services = out
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    Some(ServiceEntry {
                        name:   parts[0].trim_end_matches(".service").to_string(),
                        active: parts[2].to_string(),
                        sub:    parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .take(80)
            .collect();

        self.idx = self.idx.min(self.services.len().saturating_sub(1));
    }

    pub fn move_up(&mut self)   { if self.idx > 0 { self.idx -= 1; } }
    pub fn move_down(&mut self) { if self.idx + 1 < self.services.len() { self.idx += 1; } }

    pub fn current(&self) -> Option<&ServiceEntry> {
        self.services.get(self.idx)
    }

    pub fn do_action(&mut self, name: &str, action: &str) -> String {
        let out = Command::new("systemctl")
            .arg(action)
            .arg(format!("{}.service", name))
            .output();
        self.load();
        match out {
            Ok(o) => {
                let s = String::from_utf8_lossy(&o.stdout).into_owned();
                let e = String::from_utf8_lossy(&o.stderr).into_owned();
                if !s.is_empty() { s } else if !e.is_empty() { e } else { format!("{action} {name}: done") }
            }
            Err(e) => e.to_string(),
        }
    }

    pub fn status_of(&self, name: &str) -> String {
        Command::new("systemctl")
            .args(["status", &format!("{}.service", name), "--no-pager"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().take(25).collect::<Vec<_>>().join("\n"))
            .unwrap_or_else(|e| e.to_string())
    }
}
