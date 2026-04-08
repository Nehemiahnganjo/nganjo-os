// ══════════════════════════════════════════════════════════════════════════
//  panels/users.rs — user manager (reads /etc/passwd + /etc/group)
// ══════════════════════════════════════════════════════════════════════════

use std::{collections::HashMap, fs, process::Command};

#[derive(Clone, Debug)]
pub struct UserEntry {
    pub name:   String,
    pub uid:    u32,
    pub home:   String,
    pub shell:  String,
    pub groups: String,
}

pub struct UserPanel {
    pub idx:   usize,
    pub users: Vec<UserEntry>,
}

impl UserPanel {
    pub fn new() -> Self {
        let mut p = Self { idx: 0, users: vec![] };
        p.load();
        p
    }

    pub fn load(&mut self) {
        // Parse /etc/group into { username → [group names] }
        let mut user_groups: HashMap<String, Vec<String>> = HashMap::new();
        if let Ok(content) = fs::read_to_string("/etc/group") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() < 4 { continue; }
                let gname   = parts[0];
                let members = parts[3];
                for member in members.split(',').filter(|s| !s.is_empty()) {
                    user_groups.entry(member.to_string())
                        .or_default()
                        .push(gname.to_string());
                }
            }
        }

        // Parse /etc/passwd
        self.users = fs::read_to_string("/etc/passwd")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let p: Vec<&str> = line.split(':').collect();
                if p.len() < 7 { return None; }
                let uid: u32 = p[2].parse().ok()?;
                if uid >= 1000 || uid == 0 {
                    let name   = p[0].to_string();
                    let groups = user_groups.get(&name)
                        .map(|gs| gs[..gs.len().min(4)].join(", "))
                        .unwrap_or_default();
                    Some(UserEntry {
                        name,
                        uid,
                        home:  p[5].to_string(),
                        shell: p[6].rsplit('/').next().unwrap_or("?").to_string(),
                        groups,
                    })
                } else {
                    None
                }
            })
            .collect();

        self.users.sort_by_key(|u| u.uid);
        self.idx = self.idx.min(self.users.len().saturating_sub(1));
    }

    pub fn move_up(&mut self)   { if self.idx > 0 { self.idx -= 1; } }
    pub fn move_down(&mut self) { if self.idx + 1 < self.users.len() { self.idx += 1; } }

    pub fn current(&self) -> Option<&UserEntry> {
        self.users.get(self.idx)
    }

    fn pkexec(args: &[&str]) -> String {
        Command::new("pkexec")
            .args(args)
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout).into_owned();
                let e = String::from_utf8_lossy(&o.stderr).into_owned();
                if !s.is_empty() { s } else { e }
            })
            .unwrap_or_else(|e| e.to_string())
    }

    pub fn add_user(&mut self, name: &str) -> String {
        let out = Self::pkexec(&["useradd", "-m", name]);
        self.load();
        if out.is_empty() { format!("Created user {}", name) } else { out }
    }

    pub fn del_user(&mut self, name: &str) -> String {
        let out = Self::pkexec(&["userdel", "-r", name]);
        self.load();
        if out.is_empty() { format!("Deleted user {}", name) } else { out }
    }

    pub fn add_to_group(&mut self, group: &str) -> String {
        let name = match self.current() { Some(u) => u.name.clone(), None => return "Nothing selected".into() };
        let out = Self::pkexec(&["usermod", "-aG", group, &name]);
        self.load();
        if out.is_empty() { format!("Added {} to {}", name, group) } else { out }
    }

    pub fn lock_user(&self) -> String {
        let name = match self.current() { Some(u) => u.name.as_str(), None => return "Nothing selected".into() }.to_string();
        let out = Self::pkexec(&["usermod", "-L", &name]);
        if out.is_empty() { format!("Locked {}", name) } else { out }
    }

    pub fn unlock_user(&self) -> String {
        let name = match self.current() { Some(u) => u.name.as_str(), None => return "Nothing selected".into() }.to_string();
        let out = Self::pkexec(&["usermod", "-U", &name]);
        if out.is_empty() { format!("Unlocked {}", name) } else { out }
    }
}
