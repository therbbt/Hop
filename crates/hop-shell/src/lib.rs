//! Local shell `Session`, backed by `portable-pty`. This crate is just the
//! byte source — grid rendering/reflow is xterm.js's job.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::Bytes;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use hop_core::session::{CommandRunner, Session, SessionError, SessionEvent, SessionState};

/// Both fields are optional and empty by default — a bare `{}` (or the
/// `None` sent for an ad-hoc "Local shell" tab) behaves exactly like the
/// old hardcoded `default_shell()` + `$HOME` did.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellConnectOptions {
    /// Overrides `$SHELL` (Unix) / `$COMSPEC` (Windows) when set.
    #[serde(default)]
    pub shell_command: Option<String>,
    /// Arguments passed to `shell_command` — e.g. `["-d", "Ubuntu"]` to
    /// launch a specific WSL distro via `wsl.exe`. Meaningless without
    /// `shell_command` also set, since `CommandBuilder` (unlike a real
    /// shell) never splits a single string into a program and its args.
    #[serde(default)]
    pub shell_args: Option<Vec<String>>,
    /// Overrides `$HOME` as the starting directory when set.
    #[serde(default)]
    pub working_dir: Option<String>,
}

pub struct ShellSession {
    options: ShellConnectOptions,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    child: Option<Box<dyn Child + Send + Sync>>,
}

impl Default for ShellSession {
    fn default() -> Self {
        Self { options: ShellConnectOptions::default(), master: None, writer: None, child: None }
    }
}

impl ShellSession {
    pub fn new(options: ShellConnectOptions) -> Self {
        Self { options, master: None, writer: None, child: None }
    }
}

#[async_trait]
impl Session for ShellSession {
    async fn start(&mut self, events: mpsc::UnboundedSender<SessionEvent>) -> Result<(), SessionError> {
        let _ = events.send(SessionEvent::StateChanged { state: SessionState::Connecting });

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| SessionError::Protocol(format!("failed to open pty: {e}")))?;

        let shell = self.options.shell_command.clone().unwrap_or_else(default_shell);
        let mut cmd = CommandBuilder::new(shell);
        // `CommandBuilder` sets no TERM of its own — left unset, the child
        // shell inherits whatever TERM (if any) hop's own GUI process
        // happens to have, which for an app not launched from a terminal
        // is typically nothing meaningful. An SSH session doesn't have
        // this problem: it explicitly negotiates "xterm-256color" via
        // `request_pty` (see hop-ssh). Without the same value here,
        // programs that check $TERM before deciding whether to colorize —
        // a shell's own prompt script, `ls --color`, `git`, `less`, ncurses
        // apps — can behave differently in a local tab than over SSH for
        // reasons that have nothing to do with the program itself.
        cmd.env("TERM", "xterm-256color");
        for (key, value) in appimage_env_fixups() {
            match value {
                Some(value) => cmd.env(key, value),
                None => cmd.env_remove(key),
            }
        }
        if let Some(args) = &self.options.shell_args {
            cmd.args(args);
        }
        match &self.options.working_dir {
            Some(dir) => cmd.cwd(dir),
            None => {
                if let Ok(home) = std::env::var("HOME") {
                    cmd.cwd(home);
                }
            }
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| SessionError::Protocol(format!("failed to spawn shell: {e}")))?;
        // Drop our copy of the slave end now that the child has it, so the
        // master's reader sees EOF when the child actually exits.
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| SessionError::Protocol(format!("failed to clone pty reader: {e}")))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| SessionError::Protocol(format!("failed to take pty writer: {e}")))?;

        #[cfg(target_os = "linux")]
        if let Some(pid) = child.process_id() {
            let events = events.clone();
            std::thread::spawn(move || watch_cwd(pid, events));
        }

        self.master = Some(pair.master);
        self.writer = Some(Arc::new(Mutex::new(writer)));
        self.child = Some(child);

        let _ = events.send(SessionEvent::StateChanged { state: SessionState::Connected });

        // Blocking reads live on a dedicated OS thread; only the resulting
        // bytes cross back into async land, so an idle shell costs nothing.
        tokio::task::spawn_blocking(move || read_loop(reader, events));

        Ok(())
    }

    async fn write(&mut self, data: Bytes) -> Result<(), SessionError> {
        let writer = self.writer.clone().ok_or(SessionError::Closed)?;
        tokio::task::spawn_blocking(move || {
            let mut w = writer.lock().expect("pty writer mutex poisoned");
            w.write_all(&data)?;
            w.flush()
        })
        .await
        .map_err(|e| SessionError::Protocol(format!("write task panicked: {e}")))??;
        Ok(())
    }

    async fn resize(&mut self, cols: u16, rows: u16) -> Result<(), SessionError> {
        let Some(master) = &self.master else {
            return Err(SessionError::Closed);
        };
        master
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| SessionError::Protocol(format!("resize failed: {e}")))?;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), SessionError> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        self.writer = None;
        self.master = None;
        Ok(())
    }

    /// Runs commands with the same shell as `<shell> -c` - except for a
    /// shell with arguments of its own (e.g. `wsl.exe -d Ubuntu`), where a
    /// trailing `-c` wouldn't mean what it does to a plain shell.
    #[cfg(unix)]
    fn command_runner(&self) -> Option<Arc<dyn CommandRunner>> {
        if self.options.shell_args.as_ref().is_some_and(|a| !a.is_empty()) {
            return None;
        }
        let shell = self.options.shell_command.clone().unwrap_or_else(default_shell);
        Some(Arc::new(LocalRunner { shell }))
    }
}

#[cfg(unix)]
struct LocalRunner {
    shell: String,
}

#[cfg(unix)]
#[async_trait]
impl CommandRunner for LocalRunner {
    async fn run(&self, command: &str, cwd: &str) -> Result<i32, SessionError> {
        let cwd = match (cwd.strip_prefix('~'), std::env::var("HOME")) {
            (Some(rest), Ok(home)) if rest.is_empty() || rest.starts_with('/') => format!("{home}{rest}"),
            _ => cwd.to_string(),
        };
        let mut cmd = tokio::process::Command::new(&self.shell);
        for (key, value) in appimage_env_fixups() {
            match value {
                Some(value) => cmd.env(key, value),
                None => cmd.env_remove(key),
            };
        }
        let status = cmd
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            // The caller times a check out by dropping this future; don't
            // leave the process running behind it.
            .kill_on_drop(true)
            .status()
            .await?;
        // No code means a signal killed it - not a "yes" either way.
        Ok(status.code().unwrap_or(-1))
    }
}

fn read_loop(mut reader: Box<dyn Read + Send>, events: mpsc::UnboundedSender<SessionEvent>) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if events
                    .send(SessionEvent::Data { data: Bytes::copy_from_slice(&buf[..n]) })
                    .is_err()
                {
                    break; // frontend/adapter is gone, stop reading
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    let _ = events.send(SessionEvent::StateChanged { state: SessionState::Disconnected });
    let _ = events.send(SessionEvent::Closed { reason: None });
}

/// Reports the shell's working directory whenever it changes, read from
/// `/proc/<pid>/cwd` - so it follows every `cd` without the shell needing
/// any prompt setup of its own. Polled because nothing announces a `cd`;
/// stops once the shell is gone (the link no longer resolves) or nobody is
/// listening anymore.
#[cfg(target_os = "linux")]
fn watch_cwd(pid: u32, events: mpsc::UnboundedSender<SessionEvent>) {
    let link = format!("/proc/{pid}/cwd");
    let mut last: Option<std::path::PathBuf> = None;
    while let Ok(cwd) = std::fs::read_link(&link) {
        if last.as_ref() != Some(&cwd) {
            let path = cwd.to_string_lossy().into_owned();
            if events.send(SessionEvent::CwdChanged { path }).is_err() {
                break;
            }
            last = Some(cwd);
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

/// Env changes that undo what an AppImage's AppRun did to Hop's own env,
/// for the shells Hop spawns. AppRun (and linuxdeploy's GTK hook) point
/// LD_LIBRARY_PATH, XDG_DATA_DIRS, GTK_PATH, GIO_MODULE_DIR and friends
/// into the AppImage's mount - Hop itself needs that, but a shell that
/// inherits it hands it to everything run there, so e.g. another
/// webkit2gtk app loads Hop's bundled webkit and dies looking for its
/// helper processes. Entries under $APPDIR are dropped from every
/// variable (removing it if nothing's left), plus the vars AppRun sets
/// outright. Empty when not running as an AppImage.
fn appimage_env_fixups() -> Vec<(String, Option<String>)> {
    match std::env::var("APPDIR") {
        Ok(appdir) if !appdir.is_empty() => env_fixups_for(&appdir, std::env::vars()),
        _ => Vec::new(),
    }
}

fn env_fixups_for(appdir: &str, vars: impl Iterator<Item = (String, String)>) -> Vec<(String, Option<String>)> {
    let mut fixups: Vec<(String, Option<String>)> = ["APPDIR", "APPIMAGE", "ARGV0", "OWD", "GTK_THEME"]
        .into_iter()
        .map(|key| (key.to_string(), None))
        .collect();
    for (key, value) in vars {
        if fixups.iter().any(|(k, _)| *k == key) || !value.contains(appdir) {
            continue;
        }
        let kept: Vec<&str> =
            value.split(':').filter(|entry| !entry.is_empty() && !entry.starts_with(appdir)).collect();
        fixups.push((key, if kept.is_empty() { None } else { Some(kept.join(":")) }));
    }
    fixups
}

fn default_shell() -> String {
    #[cfg(unix)]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
    #[cfg(windows)]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appimage_env_fixups_strip_the_mount() {
        let appdir = "/tmp/.mount_hopJEPfbc";
        let vars = [
            ("LD_LIBRARY_PATH", "/tmp/.mount_hopJEPfbc/usr/lib/:/tmp/.mount_hopJEPfbc/lib/"),
            ("XDG_DATA_DIRS", "/tmp/.mount_hopJEPfbc/usr/share/:/usr/share:/usr/local/share"),
            ("GTK_EXE_PREFIX", "/tmp/.mount_hopJEPfbc//usr"),
            ("GST_PLUGIN_SYSTEM_PATH", "/tmp/.mount_hopJEPfbc/usr/lib/gstreamer:"),
            ("GTK_THEME", "Adwaita:dark"),
            ("APPIMAGE", "/home/me/.local/bin/hop"),
            ("HOME", "/home/me"),
        ]
        .map(|(k, v)| (k.to_string(), v.to_string()));
        let fixups = env_fixups_for(appdir, vars.into_iter());
        let get = |key: &str| fixups.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_deref());

        assert_eq!(get("LD_LIBRARY_PATH"), Some(None));
        assert_eq!(get("XDG_DATA_DIRS"), Some(Some("/usr/share:/usr/local/share")));
        assert_eq!(get("GTK_EXE_PREFIX"), Some(None));
        assert_eq!(get("GST_PLUGIN_SYSTEM_PATH"), Some(None));
        assert_eq!(get("GTK_THEME"), Some(None));
        assert_eq!(get("APPIMAGE"), Some(None));
        assert_eq!(get("HOME"), None);
    }
}
