# Ng'anjo OS — Power TUI v2.0 (Rust)

> The terminal interface that beats any GUI.  
> Rewritten in Rust for near-zero idle CPU, sub-millisecond UI, and a single ~5 MB binary.

## Stack

| Concern       | Crate              | Replaces (Python)         |
|---------------|--------------------|---------------------------|
| TUI rendering | `ratatui 0.26`     | `textual`                 |
| Terminal I/O  | `crossterm 0.27`   | `textual` / `curtsies`    |
| System info   | `sysinfo 0.30`     | `psutil`                  |
| Async runtime | `tokio`            | `asyncio`                 |
| Date/time     | `chrono`           | `datetime`                |
| Byte sizes    | `bytesize`         | manual formatting         |
| Error handling| `anyhow`           | bare `Exception`          |

## Build

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build (release = optimised, stripped, ~5 MB)
git clone <repo>
cd nganjo-os
cargo build --release

# Run
./target/release/nganjo
```

## Performance vs Python version

| Metric            | Python (Textual)  | Rust (ratatui)   |
|-------------------|-------------------|------------------|
| Idle CPU          | ~2–5 %            | < 0.1 %          |
| Memory            | ~60–120 MB        | ~4–8 MB          |
| Binary size       | ~50 MB + venv     | ~5 MB (stripped) |
| Startup time      | ~0.8 s            | ~20 ms           |
| Render latency    | ~16 ms            | < 1 ms           |

## Architecture

```
src/
├── main.rs          Entry point, terminal setup, event loop
├── app.rs           App state, keyboard dispatch
├── ui.rs            ratatui render pass (all widgets)
├── data.rs          sysinfo wrapper — background refresh thread
├── modal.rs         Typed modal dialogs (confirm / input / message)
└── panels/
    ├── files.rs     File manager (std::fs — no subprocess)
    ├── procs.rs     Process list (reads live data from sysinfo)
    ├── services.rs  systemd via systemctl subprocess
    ├── packages.rs  pacman / apt / dnf / zypper auto-detect
    ├── disk.rs      Disk usage (sysinfo + du subprocess)
    └── users.rs     /etc/passwd + /etc/group parser
```

## Key design decisions

**Background thread + Mutex snapshot**  
`sysinfo` refresh is done on a separate OS thread every 2 s. The UI thread
locks the `Arc<Mutex<SystemData>>` only to clone the snapshot, so the render
loop never blocks waiting on kernel I/O.

**No unsafe code**  
`kill(2)` is issued via `Command::new("kill")` rather than `libc::kill()`.
This keeps the codebase safe Rust throughout.

**Zero-copy where possible**  
`ratatui` builds `Line`/`Span` trees from borrowed data; heap allocations per
frame are minimal (only the `ListItem` vec).

**Typed modals**  
`Modal` uses boxed `FnOnce` closures that capture whatever state they need,
avoiding a giant `match` on modal IDs.

## Keybindings (same as Python version)

| Key       | Action                         |
|-----------|--------------------------------|
| Tab       | Next panel                     |
| Shift+Tab | Previous panel                 |
| ↑ / ↓    | Navigate list                  |
| q         | Quit                           |
| **Files** |                                |
| Enter     | Open / enter directory         |
| Backspace | Go up                          |
| Space     | Toggle selection               |
| c / x     | Copy / cut                     |
| v         | Paste                          |
| d         | Delete (confirm)               |
| r         | Rename                         |
| n         | New folder                     |
| p         | Preview                        |
| m         | chmod                          |
| z / e     | Compress / extract             |
| o         | Open with…                     |
| **Procs** |                                |
| k         | Kill (SIGKILL, confirm)        |
| n         | Renice                         |
| t         | Cycle sort (cpu/mem/pid/name)  |
| i         | /proc/PID/status               |
| F5        | Force refresh                  |
| **Services** |                           |
| 1–5       | start/stop/restart/enable/disable |
| i         | systemctl status               |
| **Packages** |                           |
| s         | Search                         |
| l         | List installed                 |
| 1 / 2     | Install / remove               |
| **Disks** |                                |
| i         | du breakdown                   |
| k         | Unmount                        |
| **Users** |                                |
| n / d     | Add / delete user              |
| k         | Lock / unlock                  |
| i         | Add to group                   |
