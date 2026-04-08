// ══════════════════════════════════════════════════════════════════════════
//  app.rs — central application state & keyboard dispatch
// ══════════════════════════════════════════════════════════════════════════

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    data::SystemData,
    modal::{Modal, ModalKind},
    panels::{
        disk::DiskPanel,
        files::FilePanel,
        packages::PackagePanel,
        procs::ProcPanel,
        services::ServicePanel,
        users::UserPanel,
    },
};

// ── Tab ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Files,
    Procs,
    Services,
    Packages,
    Disks,
    Users,
}

pub const TABS: &[Tab] = &[
    Tab::Files,
    Tab::Procs,
    Tab::Services,
    Tab::Packages,
    Tab::Disks,
    Tab::Users,
];

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Tab::Files    => "FILES",
            Tab::Procs    => "PROCS",
            Tab::Services => "SERVICES",
            Tab::Packages => "PACKAGES",
            Tab::Disks    => "DISKS",
            Tab::Users    => "USERS",
        }
    }
}

// ── Notification ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Notification {
    pub text:  String,
    pub kind:  NotifKind,
    pub ticks: u8,   // decremented each render; cleared at 0
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NotifKind { Info, Ok, Warn, Err }

// ── App ───────────────────────────────────────────────────────────────────

pub struct App {
    pub sysdata:     Arc<Mutex<SystemData>>,
    pub active_tab:  Tab,
    pub should_quit: bool,
    pub notif:       Option<Notification>,
    pub modal:       Option<Modal>,

    // panels (each owns its own cursor / selection state)
    pub files:    FilePanel,
    pub procs:    ProcPanel,
    pub services: ServicePanel,
    pub packages: PackagePanel,
    pub disks:    DiskPanel,
    pub users:    UserPanel,
}

impl App {
    pub fn new(sysdata: Arc<Mutex<SystemData>>) -> Self {
        Self {
            sysdata,
            active_tab:  Tab::Files,
            should_quit: false,
            notif:       None,
            modal:       None,
            files:    FilePanel::new(),
            procs:    ProcPanel::new(),
            services: ServicePanel::new(),
            packages: PackagePanel::new(),
            disks:    DiskPanel::new(),
            users:    UserPanel::new(),
        }
    }

    // ── notify helper ─────────────────────────────────────────────────────

    pub fn notify(&mut self, text: impl Into<String>, kind: NotifKind) {
        self.notif = Some(Notification {
            text:  text.into(),
            kind,
            ticks: 12,  // ~3 s at 250 ms tick
        });
    }

    pub fn tick_notif(&mut self) {
        if let Some(n) = &mut self.notif {
            if n.ticks == 0 { self.notif = None; }
            else { n.ticks -= 1; }
        }
    }

    // ── key dispatch ──────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent) {
        // ── modal eats keys first ─────────────────────────────────────────
        if self.modal.is_some() {
            self.handle_modal_key(key);
            return;
        }

        match key.code {
            // tab switching
            KeyCode::Tab => {
                let i = TABS.iter().position(|&t| t == self.active_tab).unwrap_or(0);
                self.active_tab = TABS[(i + 1) % TABS.len()];
            }
            KeyCode::BackTab => {
                let i = TABS.iter().position(|&t| t == self.active_tab).unwrap_or(0);
                self.active_tab = TABS[(i + TABS.len() - 1) % TABS.len()];
            }

            // navigation
            KeyCode::Up   => self.nav_up(),
            KeyCode::Down => self.nav_down(),

            // action keys
            KeyCode::Enter     => self.act_enter(),
            KeyCode::Backspace => self.act_back(),
            KeyCode::Char(' ') => self.act_space(),

            KeyCode::Char('c') => self.act_copy(),
            KeyCode::Char('x') => self.act_cut(),
            KeyCode::Char('v') => self.act_paste(),
            KeyCode::Char('d') => self.act_delete(),
            KeyCode::Char('r') => self.act_rename(),
            KeyCode::Char('n') => self.act_new(),
            KeyCode::Char('p') => self.act_preview(),
            KeyCode::Char('m') => self.act_chmod(),
            KeyCode::Char('z') => self.act_compress(),
            KeyCode::Char('e') => self.act_extract(),
            KeyCode::Char('o') => self.act_open_with(),
            KeyCode::Char('k') => self.act_kill(),
            KeyCode::Char('i') => self.act_info(),
            KeyCode::Char('s') => self.act_search(),
            KeyCode::Char('t') => self.act_sort(),
            KeyCode::Char('l') => self.act_list(),
            KeyCode::F(5)      => self.act_refresh(),

            // number keys (services / packages)
            KeyCode::Char(c @ '1'..='5') => self.act_number(c),

            _ => {}
        }
    }

    // ── modal key handler ─────────────────────────────────────────────────

    fn handle_modal_key(&mut self, key: KeyEvent) {
        let modal = match &mut self.modal {
            Some(m) => m,
            None    => return,
        };

        match modal.kind {
            ModalKind::Message => {
                // any key closes
                self.modal = None;
            }
            ModalKind::Confirm => {
                match key.code {
                    KeyCode::Enter => {
                        let cb = self.modal.take().and_then(|m| m.confirm_cb);
                        if let Some(cb) = cb { cb(self); }
                    }
                    KeyCode::Esc => { self.modal = None; }
                    _ => {}
                }
            }
            ModalKind::Input => {
                match key.code {
                    KeyCode::Char(c) => { modal.input.push(c); }
                    KeyCode::Backspace => { modal.input.pop(); }
                    KeyCode::Enter => {
                        let val = modal.input.clone();
                        let cb  = self.modal.take().and_then(|m| m.input_cb);
                        if let Some(cb) = cb { cb(self, val); }
                    }
                    KeyCode::Esc => { self.modal = None; }
                    _ => {}
                }
            }
        }
    }

    // ── navigation ────────────────────────────────────────────────────────

    fn nav_up(&mut self) {
        match self.active_tab {
            Tab::Files    => self.files.move_up(),
            Tab::Procs    => self.procs.move_up(),
            Tab::Services => self.services.move_up(),
            Tab::Packages => self.packages.move_up(),
            Tab::Disks    => self.disks.move_up(),
            Tab::Users    => self.users.move_up(),
        }
    }

    fn nav_down(&mut self) {
        match self.active_tab {
            Tab::Files    => self.files.move_down(),
            Tab::Procs    => self.procs.move_down(),
            Tab::Services => self.services.move_down(),
            Tab::Packages => self.packages.move_down(),
            Tab::Disks    => self.disks.move_down(),
            Tab::Users    => self.users.move_down(),
        }
    }

    // ── actions ───────────────────────────────────────────────────────────

    fn act_enter(&mut self) {
        match self.active_tab {
            Tab::Files => self.files.enter(),
            Tab::Procs => {
                if let Some(p) = self.procs.current() {
                    self.notify(
                        format!("PID {}  {}  CPU {:.1}%  MEM {:.1}%",
                            p.pid, p.name, p.cpu, p.mem),
                        NotifKind::Info,
                    );
                }
            }
            _ => {}
        }
    }

    fn act_back(&mut self) {
        if self.active_tab == Tab::Files { self.files.go_back(); }
    }

    fn act_space(&mut self) {
        if self.active_tab == Tab::Files { self.files.toggle_select(); }
    }

    fn act_copy(&mut self) {
        if self.active_tab == Tab::Files {
            self.files.copy_files();
            self.notify("Copied to clipboard", NotifKind::Ok);
        }
    }

    fn act_cut(&mut self) {
        if self.active_tab == Tab::Files {
            self.files.cut_files();
            self.notify("Cut to clipboard", NotifKind::Warn);
        }
    }

    fn act_paste(&mut self) {
        if self.active_tab == Tab::Files {
            match self.files.paste_files() {
                Ok(_)  => self.notify("Pasted successfully", NotifKind::Ok),
                Err(e) => self.notify(e.to_string(), NotifKind::Err),
            }
        }
    }

    fn act_delete(&mut self) {
        match self.active_tab {
            Tab::Files => {
                let names: Vec<String> = self.files
                    .targets()
                    .iter()
                    .take(3)
                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into())
                    .collect();
                if !names.is_empty() {
                    self.modal = Some(Modal::confirm(
                        format!("Delete: {}?", names.join(", ")),
                        |app| {
                            app.files.delete_files();
                            app.notify("Deleted", NotifKind::Warn);
                        },
                    ));
                }
            }
            Tab::Users => {
                if let Some(u) = self.users.current().cloned() {
                    let name = u.name.clone();
                    self.modal = Some(Modal::confirm(
                        format!("Delete user: {}?", name),
                        move |app| {
                            let msg = app.users.del_user(&name);
                            app.notify(msg, NotifKind::Warn);
                        },
                    ));
                }
            }
            _ => {}
        }
    }

    fn act_rename(&mut self) {
        if self.active_tab == Tab::Files {
            let cur = self.files.targets().into_iter().next()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
            if let Some(cur) = cur {
                self.modal = Some(Modal::input(
                    "Rename to:",
                    cur,
                    |app, val| {
                        app.files.rename_file(&val);
                        app.notify(format!("Renamed to {val}"), NotifKind::Ok);
                    },
                ));
            }
        }
    }

    fn act_new(&mut self) {
        match self.active_tab {
            Tab::Files => {
                self.modal = Some(Modal::input("New folder name:", "", |app, val| {
                    app.files.new_folder(&val);
                    app.notify(format!("Created {val}"), NotifKind::Ok);
                }));
            }
            Tab::Users => {
                self.modal = Some(Modal::input("New username:", "", |app, val| {
                    let msg = app.users.add_user(&val);
                    app.notify(msg, NotifKind::Ok);
                }));
            }
            _ => {}
        }
    }

    fn act_preview(&mut self) {
        if self.active_tab == Tab::Files {
            let text = self.files.preview();
            self.modal = Some(Modal::message("Preview", text));
        }
    }

    fn act_chmod(&mut self) {
        if self.active_tab == Tab::Files {
            self.modal = Some(Modal::input("Chmod (octal, e.g. 755):", "", |app, val| {
                app.files.chmod_file(&val);
                app.notify(format!("chmod {val} applied"), NotifKind::Ok);
            }));
        }
    }

    fn act_compress(&mut self) {
        if self.active_tab == Tab::Files {
            self.modal = Some(Modal::input(
                "Archive name (e.g. backup.tar.gz or backup.zip):", "",
                |app, val| {
                    app.files.compress(&val);
                    app.notify(format!("Compressed to {val}"), NotifKind::Ok);
                },
            ));
        }
    }

    fn act_extract(&mut self) {
        if self.active_tab == Tab::Files {
            self.files.extract();
            self.notify("Extracted", NotifKind::Ok);
        }
    }

    fn act_open_with(&mut self) {
        if self.active_tab == Tab::Files {
            self.modal = Some(Modal::input("Open with (command):", "", |app, val| {
                app.files.open_with(&val);
                app.notify(format!("Opened with {val}"), NotifKind::Info);
            }));
        }
    }

    fn act_kill(&mut self) {
        match self.active_tab {
            Tab::Procs => {
                if let Some(p) = self.procs.current().cloned() {
                    let label = format!("Kill PID {} ({})?", p.pid, p.name);
                    self.modal = Some(Modal::confirm(label, move |app| {
                        app.procs.kill_current();
                        app.notify(format!("Killed {}", p.name), NotifKind::Warn);
                    }));
                }
            }
            Tab::Disks => {
                if let Some(d) = self.disks.current().cloned() {
                    let mount = d.mount.clone();
                    self.modal = Some(Modal::confirm(
                        format!("Unmount {}?", mount),
                        move |app| {
                            let msg = app.disks.umount(&mount);
                            app.notify(msg, NotifKind::Warn);
                        },
                    ));
                }
            }
            Tab::Users => {
                self.modal = Some(Modal::input("lock or unlock?", "", |app, val| {
                    let msg = if val.trim() == "lock" {
                        app.users.lock_user()
                    } else {
                        app.users.unlock_user()
                    };
                    app.notify(msg, NotifKind::Warn);
                }));
            }
            _ => {}
        }
    }

    fn act_info(&mut self) {
        match self.active_tab {
            Tab::Services => {
                if let Some(s) = self.services.current().cloned() {
                    let body = self.services.status_of(&s.name);
                    self.modal = Some(Modal::message(
                        format!("Service: {}", s.name),
                        body,
                    ));
                }
            }
            Tab::Disks => {
                if let Some(d) = self.disks.current().cloned() {
                    let body = self.disks.du_top(&d.mount);
                    self.modal = Some(Modal::message(
                        format!("Disk usage: {}", d.mount),
                        body,
                    ));
                }
            }
            Tab::Procs => {
                if let Some(p) = self.procs.current().cloned() {
                    let body = self.procs.proc_info(p.pid);
                    self.modal = Some(Modal::message(
                        format!("Process: {}", p.name),
                        body,
                    ));
                }
            }
            Tab::Users => {
                self.modal = Some(Modal::input("Add to group:", "", |app, g| {
                    let msg = app.users.add_to_group(&g);
                    app.notify(msg, NotifKind::Ok);
                }));
            }
            _ => {}
        }
    }

    fn act_search(&mut self) {
        if self.active_tab == Tab::Packages {
            self.modal = Some(Modal::input("Search packages:", "", |app, q| {
                app.packages.search(&q);
                let n = app.packages.results.len();
                app.notify(format!("Found {n} packages"), NotifKind::Ok);
            }));
        }
    }

    fn act_sort(&mut self) {
        if self.active_tab == Tab::Procs { self.procs.cycle_sort(); }
    }

    fn act_list(&mut self) {
        if self.active_tab == Tab::Packages {
            self.packages.list_installed();
            self.notify("Listed installed packages", NotifKind::Ok);
        }
    }

    fn act_refresh(&mut self) {
        match self.active_tab {
            Tab::Services => self.services.load(),
            Tab::Disks    => self.disks.load(),
            Tab::Users    => self.users.load(),
            _ => {}
        }
        self.notify("Refreshed", NotifKind::Ok);
    }

    fn act_number(&mut self, c: char) {
        match self.active_tab {
            Tab::Services => {
                if let Some(s) = self.services.current().cloned() {
                    let action = match c {
                        '1' => "start", '2' => "stop", '3' => "restart",
                        '4' => "enable", '5' => "disable", _ => return,
                    };
                    let label = format!("{} service: {}?", action, s.name);
                    let name  = s.name.clone();
                    self.modal = Some(Modal::confirm(label, move |app| {
                        let msg = app.services.do_action(&name, action);
                        app.notify(msg, NotifKind::Ok);
                    }));
                }
            }
            Tab::Packages => {
                match c {
                    '1' => {
                        if let Some(pkg) = self.packages.current_pkg().map(String::from) {
                            self.modal = Some(Modal::confirm(
                                format!("Install: {}?", pkg),
                                move |app| {
                                    let msg = app.packages.install(&pkg);
                                    app.notify(msg, NotifKind::Ok);
                                },
                            ));
                        }
                    }
                    '2' => {
                        if let Some(pkg) = self.packages.current_pkg().map(String::from) {
                            self.modal = Some(Modal::confirm(
                                format!("Remove: {}?", pkg),
                                move |app| {
                                    let msg = app.packages.remove(&pkg);
                                    app.notify(msg, NotifKind::Warn);
                                },
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Tab::Procs if c == 'n' => {
                self.modal = Some(Modal::input("Renice value (-20 to 19):", "", |app, v| {
                    app.procs.renice(&v);
                    app.notify(format!("Reniced to {v}"), NotifKind::Ok);
                }));
            }
            _ => {}
        }
    }
}
