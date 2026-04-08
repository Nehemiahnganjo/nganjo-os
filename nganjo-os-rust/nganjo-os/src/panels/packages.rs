// ══════════════════════════════════════════════════════════════════════════
//  panels/packages.rs — distro-agnostic package manager frontend
// ══════════════════════════════════════════════════════════════════════════

use std::process::Command;

struct Pm {
    name:    &'static str,
    search:  &'static str,
    install: &'static str,
    remove:  &'static str,
    list:    &'static str,
}

const PMS: &[Pm] = &[
    Pm { name: "pacman", search: "pacman -Ss {q}",                  install: "pkexec pacman -S --noconfirm {pkg}", remove: "pkexec pacman -R --noconfirm {pkg}", list: "pacman -Q" },
    Pm { name: "apt",    search: "apt search {q} 2>/dev/null",       install: "pkexec apt install -y {pkg}",        remove: "pkexec apt remove -y {pkg}",         list: "apt list --installed 2>/dev/null" },
    Pm { name: "dnf",    search: "dnf search {q} 2>/dev/null",       install: "pkexec dnf install -y {pkg}",        remove: "pkexec dnf remove -y {pkg}",         list: "dnf list installed" },
    Pm { name: "zypper", search: "zypper search {q}",                install: "pkexec zypper install -y {pkg}",     remove: "pkexec zypper remove -y {pkg}",      list: "zypper packages --installed-only" },
];

fn detect_pm() -> Option<&'static Pm> {
    PMS.iter().find(|pm| which::which(pm.name).is_ok())
}

pub struct PackagePanel {
    pm:              Option<&'static Pm>,
    pub idx:         usize,
    pub results:     Vec<String>,
    pub query_label: String,
}

impl PackagePanel {
    pub fn new() -> Self {
        Self {
            pm:          detect_pm(),
            idx:         0,
            results:     vec![],
            query_label: "—".into(),
        }
    }

    pub fn move_up(&mut self)   { if self.idx > 0 { self.idx -= 1; } }
    pub fn move_down(&mut self) { if self.idx + 1 < self.results.len() { self.idx += 1; } }

    pub fn pm_name(&self) -> &str {
        self.pm.map(|p| p.name).unwrap_or("none")
    }

    pub fn search(&mut self, q: &str) {
        let pm = match self.pm { Some(p) => p, None => return };
        self.query_label = q.to_string();
        let cmd = pm.search.replace("{q}", q);
        let out = sh(&cmd);
        self.results = out
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with(' '))
            .filter_map(|l| l.split_whitespace().next().map(String::from))
            .take(60)
            .collect();
        self.idx = 0;
    }

    pub fn list_installed(&mut self) {
        let pm = match self.pm { Some(p) => p, None => return };
        self.query_label = "[installed]".into();
        let out = sh(pm.list);
        self.results = out
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with("Listing"))
            .filter_map(|l| l.split_whitespace().next().map(String::from))
            .take(100)
            .collect();
        self.idx = 0;
    }

    pub fn current_pkg(&self) -> Option<&str> {
        self.results.get(self.idx).map(|s| {
            // strip repo prefix (e.g. "core/vim" → "vim")
            s.split('/').last().and_then(|t| t.split_whitespace().next()).unwrap_or(s.as_str())
        })
    }

    pub fn install(&self, pkg: &str) -> String {
        let pm = match self.pm { Some(p) => p, None => return "No package manager".into() };
        sh(&pm.install.replace("{pkg}", pkg))
    }

    pub fn remove(&self, pkg: &str) -> String {
        let pm = match self.pm { Some(p) => p, None => return "No package manager".into() };
        sh(&pm.remove.replace("{pkg}", pkg))
    }
}

fn sh(cmd: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}
