// ══════════════════════════════════════════════════════════════════════════
//  ui.rs — ratatui render pass
//  Called every tick; reads shared SystemData snapshot and app state.
// ══════════════════════════════════════════════════════════════════════════

use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    app::{App, NotifKind, Tab, TABS},
    data::bar,
    modal::ModalKind,
};

// ── Palette ───────────────────────────────────────────────────────────────

const C_BG:     Color = Color::Rgb(15,  23,  42);
const C_BG2:    Color = Color::Rgb(2,   6,   23);
const C_BG3:    Color = Color::Rgb(30,  41,  59);
const C_DIM:    Color = Color::Rgb(100, 116, 139);
const C_MUTED:  Color = Color::Rgb(148, 163, 184);
const C_TEXT:   Color = Color::Rgb(226, 232, 240);
const C_BLUE:   Color = Color::Rgb(56,  189, 248);
const C_RED:    Color = Color::Rgb(248, 113, 113);
const C_GREEN:  Color = Color::Rgb(52,  211, 153);
const C_YELLOW: Color = Color::Rgb(251, 191, 36);
const C_PURPLE: Color = Color::Rgb(167, 139, 250);
const C_ORANGE: Color = Color::Rgb(251, 146, 60);
const C_SKY:    Color = Color::Rgb(125, 211, 252);

fn tab_color(tab: Tab) -> Color {
    match tab {
        Tab::Files    => C_BLUE,
        Tab::Procs    => C_RED,
        Tab::Services => C_PURPLE,
        Tab::Packages => C_GREEN,
        Tab::Disks    => C_YELLOW,
        Tab::Users    => C_ORANGE,
    }
}

fn bar_color(pct: f32) -> Color {
    if pct > 80.0 { C_RED } else if pct > 50.0 { C_YELLOW } else { C_GREEN }
}

fn colored_bar(pct: f32, width: usize) -> Vec<Span<'static>> {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty  = width - filled;
    vec![
        Span::styled("█".repeat(filled), Style::default().fg(bar_color(pct))),
        Span::styled("░".repeat(empty),  Style::default().fg(C_BG3)),
    ]
}

// ══════════════════════════════════════════════════════════════════════════
//  Main draw function
// ══════════════════════════════════════════════════════════════════════════

pub fn draw(f: &mut Frame, app: &mut App) {
    // Grab a snapshot of system data
    let sysdata_snapshot = app.sysdata.lock().ok().map(|d| {
        (
            d.cpu_pct, d.mem_pct, d.disk_pct,
            d.uptime_str(), d.mem_str(),
            d.procs.clone(),
            d.disks.clone(),
            d.ifaces.clone(),
        )
    });

    // Push live process/disk data into panels
    if let Some((_, _, _, _, _, ref procs, ref disks, _)) = sysdata_snapshot {
        app.procs.update(procs.clone());
        app.disks.update(disks.clone());
    }

    app.tick_notif();

    let area = f.size();

    // ── top-level vertical split ──────────────────────────────────────────
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(1),  // tab bar
            Constraint::Min(0),     // main body
            Constraint::Length(1),  // notify
        ])
        .split(area);

    draw_header(f, vchunks[0], &sysdata_snapshot);
    draw_tabs(f, vchunks[1], app.active_tab);
    draw_body(f, vchunks[2], app, &sysdata_snapshot);
    draw_notify(f, vchunks[3], app);

    // ── modal overlay ─────────────────────────────────────────────────────
    if app.modal.is_some() {
        draw_modal(f, area, app);
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  Header
// ══════════════════════════════════════════════════════════════════════════

fn draw_header(f: &mut Frame, area: Rect, snap: &Option<(f32,f32,f32,String,String,_,_,_)>) {
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(44)])
        .split(area);

    // brand / clock
    let now = Local::now();
    let brand = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" NG'ANJO OS", Style::default().fg(C_BLUE).add_modifier(Modifier::BOLD)),
            Span::styled(" v2.0", Style::default().fg(C_DIM)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {}", now.format("%H:%M:%S")),
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", now.format("%a %d %b %Y")),
                Style::default().fg(C_DIM),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(C_BG3)));
    f.render_widget(brand, hchunks[0]);

    // mini stats
    if let Some((cpu, mem, disk, _, _, _, _, _)) = snap {
        let mut lines = vec![];
        for (label, pct) in [("CPU", cpu), ("RAM", mem), ("DSK", disk)] {
            let mut spans = vec![
                Span::styled(format!("{} ", label), Style::default().fg(C_DIM)),
            ];
            spans.extend(colored_bar(*pct, 10));
            spans.push(Span::styled(
                format!(" {:3.0}%", pct),
                Style::default().fg(C_TEXT),
            ));
            lines.push(Line::from(spans));
        }
        let stats = Paragraph::new(lines)
            .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT)
                .border_style(Style::default().fg(C_BG3)));
        f.render_widget(stats, hchunks[1]);
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  Tab bar
// ══════════════════════════════════════════════════════════════════════════

fn draw_tabs(f: &mut Frame, area: Rect, active: Tab) {
    let spans: Vec<Span> = TABS.iter().flat_map(|&t| {
        let c = tab_color(t);
        if t == active {
            vec![
                Span::styled(
                    format!("  {}  ", t.label()),
                    Style::default().fg(C_BG2).bg(c).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]
        } else {
            vec![
                Span::styled(
                    format!("  {}  ", t.label()),
                    Style::default().fg(C_DIM),
                ),
                Span::raw(" "),
            ]
        }
    }).collect();

    let tabs = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(C_BG2));
    f.render_widget(tabs, area);
}

// ══════════════════════════════════════════════════════════════════════════
//  Body
// ══════════════════════════════════════════════════════════════════════════

fn draw_body(f: &mut Frame, area: Rect, app: &mut App, snap: &Option<(f32,f32,f32,String,String,_,_,_)>) {
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),  // left sidebar
            Constraint::Min(0),      // main panel
            Constraint::Length(22),  // right help
        ])
        .split(area);

    draw_sidebar(f, hchunks[0], snap);

    match app.active_tab {
        Tab::Files    => draw_files(f, hchunks[1], app),
        Tab::Procs    => draw_procs(f, hchunks[1], app),
        Tab::Services => draw_services(f, hchunks[1], app),
        Tab::Packages => draw_packages(f, hchunks[1], app),
        Tab::Disks    => draw_disks(f, hchunks[1], app),
        Tab::Users    => draw_users(f, hchunks[1], app),
    }

    draw_help(f, hchunks[2], app.active_tab);
}

// ══════════════════════════════════════════════════════════════════════════
//  Sidebar
// ══════════════════════════════════════════════════════════════════════════

fn draw_sidebar(f: &mut Frame, area: Rect, snap: &Option<(f32,f32,f32,String,String,_,_,_)>) {
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // sys stats
    let mut sys_lines = vec![
        Line::from(Span::styled(" SYSTEM", Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD))),
    ];

    if let Some((cpu, mem, disk, uptime, mem_str, _, _, _)) = snap {
        for (label, pct) in [("CPU ", cpu), ("RAM ", mem), ("DISK", disk)] {
            let mut spans = vec![Span::styled(format!("  {} ", label), Style::default().fg(C_DIM))];
            spans.extend(colored_bar(*pct, 10));
            spans.push(Span::styled(format!(" {:3.0}%", pct), Style::default().fg(C_TEXT)));
            sys_lines.push(Line::from(spans));
        }
        sys_lines.push(Line::from(vec![
            Span::styled("  UP   ", Style::default().fg(C_DIM)),
            Span::styled(uptime.clone(), Style::default().fg(C_MUTED)),
        ]));
    }

    let sys_block = Paragraph::new(sys_lines)
        .block(Block::default()
            .borders(Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(C_BG3)));
    f.render_widget(sys_block, vchunks[0]);

    // network
    let mut net_lines = vec![
        Line::from(Span::styled(" NETWORK", Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD))),
    ];

    if let Some((_, _, _, _, _, _, _, ifaces)) = snap {
        for iface in ifaces.iter().take(4) {
            let dot = if iface.up {
                Span::styled("●", Style::default().fg(C_GREEN))
            } else {
                Span::styled("●", Style::default().fg(C_RED))
            };
            net_lines.push(Line::from(vec![
                Span::raw("  "),
                dot,
                Span::styled(format!(" {:<10}", iface.name), Style::default().fg(C_TEXT)),
                Span::styled(format!("{:<16}", iface.ip), Style::default().fg(C_DIM)),
            ]));
        }
        if ifaces.is_empty() {
            net_lines.push(Line::from(Span::styled("  No interfaces", Style::default().fg(C_DIM))));
        }
    }

    let net_block = Paragraph::new(net_lines)
        .block(Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(C_BG3)));
    f.render_widget(net_block, vchunks[1]);
}

// ══════════════════════════════════════════════════════════════════════════
//  FILE PANEL
// ══════════════════════════════════════════════════════════════════════════

fn draw_files(f: &mut Frame, area: Rect, app: &App) {
    let fp = &app.files;
    let path_str = fp.path.to_string_lossy();

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BG3))
        .title(Line::from(vec![
            Span::styled(" FILES", Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD)),
            Span::styled(" [FOCUS]", Style::default().fg(C_BLUE)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    // path
    f.render_widget(
        Paragraph::new(Span::styled(
            format!(" {}", &path_str[..path_str.len().min(50)]),
            Style::default().fg(C_DIM),
        )),
        vchunks[0],
    );

    // entries
    let visible_h = vchunks[1].height as usize;
    let start = if fp.idx >= visible_h { fp.idx - visible_h + 1 } else { 0 };

    let items: Vec<ListItem> = fp.entries
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_h)
        .map(|(i, e)| {
            let is_back = e.path == fp.path;
            let name = if is_back { "..".to_string() } else { e.name().to_string() };
            let name = format!("{:<34}", name);
            let icon = if e.is_dir { " " } else { " " };
            let check = if fp.selected.contains(&i) { "✓ " } else { "  " };

            let style = if i == fp.idx {
                Style::default().fg(C_BLUE).bg(C_BG3).add_modifier(Modifier::BOLD)
            } else if fp.selected.contains(&i) {
                Style::default().fg(C_YELLOW)
            } else if e.is_dir {
                Style::default().fg(C_SKY)
            } else {
                Style::default().fg(C_MUTED)
            };

            ListItem::new(Line::from(Span::styled(
                format!(" {}{}{}", check, icon, name),
                style,
            )))
        })
        .collect();

    f.render_widget(List::new(items), vchunks[1]);

    // footer
    if let Some(e) = fp.current() {
        let info = if e.is_dir {
            format!(" {}  (dir)", e.perms_str())
        } else {
            format!(" {}  {} B", e.perms_str(), e.size)
        };
        if let Some(clip) = &fp.clipboard {
            let op = match clip.op { crate::panels::files::ClipOp::Copy => "copy", crate::panels::files::ClipOp::Cut => "cut" };
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(info, Style::default().fg(C_DIM)),
                    Span::styled(format!("  clip({}): {} item(s)", op, clip.paths.len()), Style::default().fg(C_DIM)),
                ])),
                vchunks[2],
            );
        } else {
            f.render_widget(
                Paragraph::new(Span::styled(info, Style::default().fg(C_DIM))),
                vchunks[2],
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  PROCESS PANEL
// ══════════════════════════════════════════════════════════════════════════

fn draw_procs(f: &mut Frame, area: Rect, app: &App) {
    let pp = &app.procs;

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BG3))
        .title(Line::from(vec![
            Span::styled(" PROCESSES", Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD)),
            Span::styled(" [FOCUS]", Style::default().fg(C_RED)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    f.render_widget(
        Paragraph::new(Span::styled(
            format!("   CPU     MEM    PID   NAME   (sort: {})", pp.sort_label()),
            Style::default().fg(C_DIM),
        )),
        vchunks[0],
    );

    let h = vchunks[1].height as usize;
    let start = if pp.idx >= h { pp.idx - h + 1 } else { 0 };

    let items: Vec<ListItem> = pp.procs
        .iter()
        .enumerate()
        .skip(start)
        .take(h)
        .map(|(i, p)| {
            let cpu_c = if p.cpu > 50.0 { C_RED } else if p.cpu > 10.0 { C_YELLOW } else { C_MUTED };
            let mem_c = if p.mem > 50.0 { C_RED } else if p.mem > 10.0 { C_YELLOW } else { C_MUTED };
            let row_style = if i == pp.idx { Style::default().bg(C_BG3) } else { Style::default() };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {:6.1}%", p.cpu), Style::default().fg(cpu_c).patch(row_style)),
                Span::styled(format!(" {:5.1}%", p.mem), Style::default().fg(mem_c).patch(row_style)),
                Span::styled(format!(" {:6}", p.pid),    Style::default().fg(C_DIM).patch(row_style)),
                Span::styled(format!(" {:<18}", &p.name[..p.name.len().min(18)]), Style::default().fg(C_TEXT).patch(row_style)),
            ]))
        })
        .collect();

    f.render_widget(List::new(items), vchunks[1]);
}

// ══════════════════════════════════════════════════════════════════════════
//  SERVICE PANEL
// ══════════════════════════════════════════════════════════════════════════

fn draw_services(f: &mut Frame, area: Rect, app: &App) {
    let sp = &app.services;

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BG3))
        .title(Span::styled(" SERVICES [FOCUS]", Style::default().fg(C_PURPLE).add_modifier(Modifier::BOLD)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let h = inner.height as usize;
    let start = if sp.idx >= h { sp.idx - h + 1 } else { 0 };

    let items: Vec<ListItem> = sp.services
        .iter()
        .enumerate()
        .skip(start)
        .take(h)
        .map(|(i, s)| {
            let (dot, dot_c, sub_c) = match s.active.as_str() {
                "active"   => ("●", C_GREEN,  C_GREEN),
                "failed"   => ("●", C_RED,    C_RED),
                _          => ("○", C_DIM,    C_DIM),
            };
            let row_style = if i == sp.idx { Style::default().bg(C_BG3) } else { Style::default() };

            ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(dot, Style::default().fg(dot_c).patch(row_style)),
                Span::styled(format!(" {:<11}", s.sub), Style::default().fg(sub_c).patch(row_style)),
                Span::styled(format!("{}", s.name), Style::default().fg(C_TEXT).patch(row_style)),
            ]))
        })
        .collect();

    f.render_widget(List::new(items), inner);
}

// ══════════════════════════════════════════════════════════════════════════
//  PACKAGE PANEL
// ══════════════════════════════════════════════════════════════════════════

fn draw_packages(f: &mut Frame, area: Rect, app: &App) {
    let pp = &app.packages;

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BG3))
        .title(Line::from(vec![
            Span::styled(" PACKAGES", Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({})", pp.pm_name()), Style::default().fg(C_DIM)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Query: ", Style::default().fg(C_DIM)),
            Span::styled(pp.query_label.clone(), Style::default().fg(C_TEXT)),
            Span::styled(format!("  ({})", pp.results.len()), Style::default().fg(C_DIM)),
        ])),
        vchunks[0],
    );

    if pp.results.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("  S = search", Style::default().fg(C_DIM))),
                Line::from(Span::styled("  L = list installed", Style::default().fg(C_DIM))),
            ]),
            vchunks[1],
        );
        return;
    }

    let h = vchunks[1].height as usize;
    let start = if pp.idx >= h { pp.idx - h + 1 } else { 0 };

    let items: Vec<ListItem> = pp.results
        .iter()
        .enumerate()
        .skip(start)
        .take(h)
        .map(|(i, pkg)| {
            let s = if i == pp.idx { Style::default().bg(C_BG3).fg(C_TEXT) } else { Style::default().fg(C_TEXT) };
            ListItem::new(Span::styled(format!("  {}", pkg), s))
        })
        .collect();

    f.render_widget(List::new(items), vchunks[1]);
}

// ══════════════════════════════════════════════════════════════════════════
//  DISK PANEL
// ══════════════════════════════════════════════════════════════════════════

fn draw_disks(f: &mut Frame, area: Rect, app: &App) {
    let dp = &app.disks;

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BG3))
        .title(Span::styled(" DISKS [FOCUS]", Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let items: Vec<ListItem> = dp.disks
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let row_style = if i == dp.idx { Style::default().bg(C_BG3) } else { Style::default() };
            let pct = d.pct as f32;
            let bar_color = if pct > 80.0 { C_RED } else if pct > 50.0 { C_YELLOW } else { C_GREEN };
            let filled = ((pct / 100.0) * 8.0).round() as usize;
            let empty  = 8usize.saturating_sub(filled);

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {:<14}", d.device), Style::default().fg(C_SKY).patch(row_style)),
                Span::styled(format!("{:<16}", d.mount),   Style::default().fg(C_DIM).patch(row_style)),
                Span::styled("█".repeat(filled),           Style::default().fg(bar_color).patch(row_style)),
                Span::styled("░".repeat(empty),            Style::default().fg(C_BG3).patch(row_style)),
                Span::styled(format!(" {:3.0}%", pct),     Style::default().fg(C_TEXT).patch(row_style)),
            ]))
        })
        .collect();

    f.render_widget(List::new(items), vchunks[0]);

    if let Some(d) = dp.current() {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!(" {} / {}  [{}]",
                    bytesize::ByteSize(d.used),
                    bytesize::ByteSize(d.total),
                    d.fstype),
                Style::default().fg(C_DIM),
            )),
            vchunks[1],
        );
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  USER PANEL
// ══════════════════════════════════════════════════════════════════════════

fn draw_users(f: &mut Frame, area: Rect, app: &App) {
    let up = &app.users;

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(C_BG3))
        .title(Span::styled(" USERS [FOCUS]", Style::default().fg(C_ORANGE).add_modifier(Modifier::BOLD)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let items: Vec<ListItem> = up.users
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let uid_c = if u.uid == 0 { C_YELLOW } else { C_MUTED };
            let row_style = if i == up.idx { Style::default().bg(C_BG3) } else { Style::default() };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {:<7}", u.uid),  Style::default().fg(uid_c).patch(row_style)),
                Span::styled(format!("{:<16}", u.name), Style::default().fg(C_TEXT).patch(row_style)),
                Span::styled(u.shell.clone(),           Style::default().fg(C_DIM).patch(row_style)),
            ]))
        })
        .collect();

    f.render_widget(List::new(items), vchunks[0]);

    if let Some(u) = up.current() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(" home: ", Style::default().fg(C_DIM)),
                    Span::styled(u.home.clone(), Style::default().fg(C_SKY)),
                ]),
                Line::from(vec![
                    Span::styled(" groups: ", Style::default().fg(C_DIM)),
                    Span::styled(
                        if u.groups.is_empty() { "—".into() } else { u.groups.clone() },
                        Style::default().fg(C_MUTED),
                    ),
                ]),
            ]),
            vchunks[1],
        );
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  HELP sidebar
// ══════════════════════════════════════════════════════════════════════════

fn draw_help(f: &mut Frame, area: Rect, tab: Tab) {
    let c = tab_color(tab);

    let keys: &[(&str, &str)] = match tab {
        Tab::Files    => &[("↑↓","navigate"),("ENTER","open/enter"),("BSP","go back"),("SPACE","select"),("C","copy"),("X","cut"),("V","paste"),("D","delete"),("R","rename"),("N","new folder"),("P","preview"),("M","chmod"),("Z","compress"),("E","extract"),("O","open with")],
        Tab::Procs    => &[("↑↓","navigate"),("K","kill"),("N","renice"),("T","cycle sort"),("I","/proc info"),("ENTER","summary"),("F5","refresh")],
        Tab::Services => &[("↑↓","navigate"),("1","start"),("2","stop"),("3","restart"),("4","enable"),("5","disable"),("I","status"),("F5","refresh")],
        Tab::Packages => &[("↑↓","navigate"),("S","search"),("L","installed"),("1","install"),("2","remove")],
        Tab::Disks    => &[("↑↓","navigate"),("I","du breakdown"),("K","unmount"),("F5","refresh")],
        Tab::Users    => &[("↑↓","navigate"),("N","add user"),("D","delete"),("K","lock/unlock"),("I","add to group")],
    };

    let mut lines = vec![
        Line::from(Span::styled(tab.label(), Style::default().fg(c).add_modifier(Modifier::BOLD))),
    ];

    for (k, v) in keys {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<7}", k), Style::default().fg(C_DIM)),
            Span::styled(*v, Style::default().fg(C_MUTED)),
        ]));
    }

    lines.push(Line::from(Span::styled("──────", Style::default().fg(C_BG3))));
    lines.push(Line::from(vec![
        Span::styled("TAB    ", Style::default().fg(C_DIM)),
        Span::styled("next panel", Style::default().fg(C_MUTED)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Q      ", Style::default().fg(C_DIM)),
        Span::styled("quit", Style::default().fg(C_MUTED)),
    ]));

    f.render_widget(
        Paragraph::new(lines).block(Block::default()),
        area,
    );
}

// ══════════════════════════════════════════════════════════════════════════
//  NOTIFY bar
// ══════════════════════════════════════════════════════════════════════════

fn draw_notify(f: &mut Frame, area: Rect, app: &App) {
    let (text, color) = match &app.notif {
        Some(n) => {
            let c = match n.kind {
                NotifKind::Ok   => C_GREEN,
                NotifKind::Warn => C_YELLOW,
                NotifKind::Err  => C_RED,
                NotifKind::Info => C_MUTED,
            };
            (n.text.clone(), c)
        }
        None => (
            " Ng'anjo OS v2.0 — More power than any GUI.".into(),
            C_DIM,
        ),
    };

    f.render_widget(
        Paragraph::new(Span::styled(text, Style::default().fg(color)))
            .style(Style::default().bg(C_BG2)),
        area,
    );
}

// ══════════════════════════════════════════════════════════════════════════
//  MODAL
// ══════════════════════════════════════════════════════════════════════════

fn draw_modal(f: &mut Frame, area: Rect, app: &App) {
    let modal = match &app.modal { Some(m) => m, None => return };

    // centered box
    let w = 62u16.min(area.width.saturating_sub(4));
    let h = match modal.kind {
        ModalKind::Input   => 7u16,
        ModalKind::Confirm => 6u16,
        ModalKind::Message => (modal.body.lines().count() as u16 + 5).min(area.height - 4),
    };
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);

    f.render_widget(Clear, rect);

    let title_c = if modal.kind == ModalKind::Confirm { C_YELLOW } else { C_BLUE };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(C_BLUE))
        .style(Style::default().bg(C_BG3))
        .title(Span::styled(format!(" {} ", modal.title), Style::default().fg(title_c).add_modifier(Modifier::BOLD)));

    let inner = block.inner(rect);
    f.render_widget(block, rect);

    match modal.kind {
        ModalKind::Message => {
            let body: Vec<Line> = modal.body.lines()
                .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(C_TEXT))))
                .collect();
            let mut body = body;
            body.push(Line::from(Span::styled("Any key to close", Style::default().fg(C_DIM))));
            f.render_widget(Paragraph::new(body), inner);
        }
        ModalKind::Confirm => {
            f.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(modal.body.clone(), Style::default().fg(C_TEXT))),
                    Line::raw(""),
                    Line::from(Span::styled("ENTER = Yes    ESC = No", Style::default().fg(C_DIM))),
                ]),
                inner,
            );
        }
        ModalKind::Input => {
            let vchunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                .split(inner);

            // input box with cursor
            let input_display = format!("{}_", modal.input);
            f.render_widget(
                Paragraph::new(Span::styled(input_display, Style::default().fg(C_TEXT)))
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(C_BG3))),
                vchunks[1],
            );
            f.render_widget(
                Paragraph::new(Span::styled("ENTER = confirm    ESC = cancel", Style::default().fg(C_DIM))),
                vchunks[2],
            );
        }
    }
}
