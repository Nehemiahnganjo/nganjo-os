// ══════════════════════════════════════════════════════════════════════════
//  panels/files.rs — file manager state (no subprocess, pure std::fs)
// ══════════════════════════════════════════════════════════════════════════

use std::{
    collections::BTreeSet,
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
    process::Command,
};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path:   PathBuf,
    pub is_dir: bool,
    pub size:   u64,
    pub mode:   u32,
}

impl FileEntry {
    pub fn name(&self) -> &str {
        self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
    }

    pub fn perms_str(&self) -> String {
        let m = self.mode;
        let mut s = String::with_capacity(9);
        for shift in [6u32, 3, 0] {
            s.push(if m >> shift & 0o4 != 0 { 'r' } else { '-' });
            s.push(if m >> shift & 0o2 != 0 { 'w' } else { '-' });
            s.push(if m >> shift & 0o1 != 0 { 'x' } else { '-' });
        }
        s
    }
}

// ── Clipboard ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ClipOp { Copy, Cut }

#[derive(Clone, Debug)]
pub struct Clipboard {
    pub op:    ClipOp,
    pub paths: Vec<PathBuf>,
}

// ── Panel ─────────────────────────────────────────────────────────────────

pub struct FilePanel {
    pub path:      PathBuf,
    pub entries:   Vec<FileEntry>,
    pub idx:       usize,
    pub selected:  BTreeSet<usize>,
    pub clipboard: Option<Clipboard>,
}

impl FilePanel {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let mut p = Self {
            path:      home.clone(),
            entries:   vec![],
            idx:       0,
            selected:  BTreeSet::new(),
            clipboard: None,
        };
        p.load(&home.clone());
        p
    }

    pub fn load(&mut self, dir: &PathBuf) {
        self.path     = dir.clone();
        self.entries  = vec![];
        self.idx      = 0;
        self.selected = BTreeSet::new();

        // ".." entry
        if let Some(parent) = dir.parent() {
            self.entries.push(FileEntry {
                path:   parent.to_path_buf(),
                is_dir: true,
                size:   0,
                mode:   0o755,
            });
        }

        if let Ok(rd) = fs::read_dir(dir) {
            let mut items: Vec<FileEntry> = rd
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let meta = e.metadata().ok()?;
                    Some(FileEntry {
                        path:   e.path(),
                        is_dir: meta.is_dir(),
                        size:   meta.len(),
                        mode:   meta.permissions().mode(),
                    })
                })
                .collect();
            items.sort_by(|a, b| {
                b.is_dir.cmp(&a.is_dir)
                    .then(a.name().to_lowercase().cmp(&b.name().to_lowercase()))
            });
            self.entries.extend(items);
        }
    }

    pub fn move_up(&mut self) {
        if !self.entries.is_empty() && self.idx > 0 {
            self.idx -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.idx + 1 < self.entries.len() {
            self.idx += 1;
        }
    }

    pub fn enter(&mut self) {
        if let Some(e) = self.entries.get(self.idx).cloned() {
            if e.is_dir {
                self.load(&e.path);
            } else {
                let _ = Command::new("xdg-open").arg(&e.path).spawn();
            }
        }
    }

    pub fn go_back(&mut self) {
        if let Some(parent) = self.path.parent().map(|p| p.to_path_buf()) {
            self.load(&parent);
        }
    }

    pub fn toggle_select(&mut self) {
        if self.selected.contains(&self.idx) {
            self.selected.remove(&self.idx);
        } else {
            self.selected.insert(self.idx);
        }
        self.move_down();
    }

    pub fn targets(&self) -> Vec<PathBuf> {
        if !self.selected.is_empty() {
            self.selected
                .iter()
                .filter_map(|&i| self.entries.get(i))
                .filter(|e| e.path != self.path)
                .map(|e| e.path.clone())
                .collect()
        } else {
            self.entries
                .get(self.idx)
                .filter(|e| e.path.parent().is_some())  // not root
                .map(|e| vec![e.path.clone()])
                .unwrap_or_default()
        }
    }

    pub fn copy_files(&mut self) {
        let t = self.targets();
        if !t.is_empty() {
            self.clipboard = Some(Clipboard { op: ClipOp::Copy, paths: t });
        }
    }

    pub fn cut_files(&mut self) {
        let t = self.targets();
        if !t.is_empty() {
            self.clipboard = Some(Clipboard { op: ClipOp::Cut, paths: t });
        }
    }

    pub fn paste_files(&mut self) -> anyhow::Result<()> {
        let clip = self.clipboard.clone().ok_or_else(|| anyhow::anyhow!("Clipboard empty"))?;
        for src in &clip.paths {
            let dst = self.path.join(src.file_name().unwrap_or_default());
            if src.is_dir() {
                copy_dir(src, &dst)?;
            } else {
                fs::copy(src, dst)?;
            }
            if matches!(clip.op, ClipOp::Cut) {
                if src.is_dir() { fs::remove_dir_all(src)?; }
                else { fs::remove_file(src)?; }
            }
        }
        if matches!(clip.op, ClipOp::Cut) { self.clipboard = None; }
        self.load(&self.path.clone());
        Ok(())
    }

    pub fn delete_files(&mut self) {
        for p in self.targets() {
            let _ = if p.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) };
        }
        self.load(&self.path.clone());
    }

    pub fn rename_file(&mut self, new_name: &str) {
        if let Some(src) = self.targets().into_iter().next() {
            let dst = self.path.join(new_name);
            let _ = fs::rename(&src, &dst);
        }
        self.load(&self.path.clone());
    }

    pub fn new_folder(&mut self, name: &str) {
        let _ = fs::create_dir(self.path.join(name));
        self.load(&self.path.clone());
    }

    pub fn chmod_file(&mut self, mode_str: &str) {
        if let Ok(mode) = u32::from_str_radix(mode_str, 8) {
            for p in self.targets() {
                let _ = fs::set_permissions(&p, fs::Permissions::from_mode(mode));
            }
        }
    }

    pub fn compress(&mut self, name: &str) {
        let targets = self.targets();
        if targets.is_empty() { return; }
        let out = self.path.join(name);
        if name.ends_with(".zip") {
            // use `zip` command if available
            let args: Vec<_> = targets.iter().map(|p| p.to_string_lossy().into_owned()).collect();
            let _ = Command::new("zip").arg("-r").arg(&out).args(&args).output();
        } else {
            let args: Vec<_> = targets.iter().map(|p| p.to_string_lossy().into_owned()).collect();
            let _ = Command::new("tar").arg("-czf").arg(&out).args(&args).output();
        }
        self.load(&self.path.clone());
    }

    pub fn extract(&mut self) {
        if let Some(p) = self.targets().into_iter().next() {
            let ext = p.to_string_lossy().to_lowercase();
            if ext.ends_with(".tar.gz") || ext.ends_with(".tgz") {
                let _ = Command::new("tar").arg("-xzf").arg(&p).arg("-C").arg(&self.path).output();
            } else if ext.ends_with(".zip") {
                let _ = Command::new("unzip").arg(&p).arg("-d").arg(&self.path).output();
            }
        }
        self.load(&self.path.clone());
    }

    pub fn preview(&self) -> String {
        let targets = self.targets();
        let p = match targets.first() {
            Some(p) => p,
            None    => return "Nothing selected.".into(),
        };
        if p.is_dir() {
            let items: Vec<_> = fs::read_dir(p)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .take(20)
                .map(|e| format!("  {}", e.file_name().to_string_lossy()))
                .collect();
            return format!("{}/\n{}", p.file_name().unwrap_or_default().to_string_lossy(), items.join("\n"));
        }
        match fs::read_to_string(p) {
            Ok(s) => format!("{}\n{}\n{}", p.file_name().unwrap_or_default().to_string_lossy(), "─".repeat(40), &s[..s.len().min(1200)]),
            Err(_) => format!("[Binary file]\n{} bytes", p.metadata().map(|m| m.len()).unwrap_or(0)),
        }
    }

    pub fn open_with(&self, cmd: &str) {
        if let Some(p) = self.targets().into_iter().next() {
            let _ = Command::new(cmd).arg(&p).spawn();
        }
    }

    pub fn current(&self) -> Option<&FileEntry> {
        self.entries.get(self.idx)
    }
}

fn copy_dir(src: &PathBuf, dst: &PathBuf) -> anyhow::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
