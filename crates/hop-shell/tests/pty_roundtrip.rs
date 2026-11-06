use std::time::Duration;

use bytes::Bytes;
use hop_core::session::{Session, SessionEvent, SessionState};
use hop_shell::{ShellConnectOptions, ShellSession};
use tokio::sync::mpsc;
use tokio::time::timeout;

async fn next_data(rx: &mut mpsc::UnboundedReceiver<SessionEvent>) -> String {
    loop {
        let event = timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("timed out waiting for session event")
            .expect("session event stream ended unexpectedly");
        if let SessionEvent::Data { data } = event {
            return String::from_utf8_lossy(&data).to_string();
        }
    }
}

async fn wait_for_state(rx: &mut mpsc::UnboundedReceiver<SessionEvent>, want: SessionState) {
    loop {
        let event = timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("timed out waiting for state")
            .expect("session event stream ended unexpectedly");
        if let SessionEvent::StateChanged { state } = event {
            if state == want {
                return;
            }
        }
    }
}

/// Proves the byte pipe end to end without any GUI involved: spawn a real
/// shell behind the pty, round-trip a command through write() -> stdout ->
/// Data events, and confirm resize() doesn't error (SIGWINCH plumbing).
#[tokio::test]
async fn shell_session_echoes_written_command() {
    let mut session = ShellSession::default();
    let (tx, mut rx) = mpsc::unbounded_channel();

    session.start(tx).await.expect("session failed to start");
    wait_for_state(&mut rx, SessionState::Connected).await;

    session
        .write(Bytes::from_static(b"echo HOP_PTY_OK\n"))
        .await
        .expect("write failed");

    let mut seen = String::new();
    let saw_marker = timeout(Duration::from_secs(5), async {
        loop {
            seen.push_str(&next_data(&mut rx).await);
            if seen.contains("HOP_PTY_OK") {
                return;
            }
        }
    })
    .await;
    assert!(saw_marker.is_ok(), "never saw echoed marker in pty output: {seen:?}");

    session.resize(120, 40).await.expect("resize failed");

    session.shutdown().await.expect("shutdown failed");
}

/// Proves `ShellConnectOptions` actually reaches the spawned process instead
/// of being silently ignored — a saved terminal preset's whole point is
/// that its shell_command/working_dir take effect.
#[tokio::test]
async fn shell_session_honors_shell_command_and_working_dir_options() {
    let tmp = std::env::temp_dir();
    let options = ShellConnectOptions {
        shell_command: Some("/bin/sh".to_string()),
        shell_args: None,
        working_dir: Some(tmp.to_string_lossy().to_string()),
    };
    let mut session = ShellSession::new(options);
    let (tx, mut rx) = mpsc::unbounded_channel();

    session.start(tx).await.expect("session failed to start");
    wait_for_state(&mut rx, SessionState::Connected).await;

    session.write(Bytes::from_static(b"pwd\n")).await.expect("write failed");

    let mut seen = String::new();
    let canonical_tmp = tmp.canonicalize().unwrap_or(tmp).to_string_lossy().to_string();
    let saw_cwd = timeout(Duration::from_secs(5), async {
        loop {
            seen.push_str(&next_data(&mut rx).await);
            if seen.contains(&canonical_tmp) {
                return;
            }
        }
    })
    .await;
    assert!(saw_cwd.is_ok(), "expected pwd output to contain {canonical_tmp:?}, got: {seen:?}");

    session.shutdown().await.expect("shutdown failed");
}

/// A `cd` in the shell is reported as a CwdChanged event (read from /proc,
/// so Linux only), with no prompt setup needed on the shell's side.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn shell_session_reports_cwd_changes() {
    let dir = std::env::temp_dir().canonicalize().expect("temp dir");
    let mut session = ShellSession::new(ShellConnectOptions { shell_command: Some("/bin/sh".to_string()), ..Default::default() });
    let (tx, mut rx) = mpsc::unbounded_channel();
    session.start(tx).await.expect("session failed to start");
    wait_for_state(&mut rx, SessionState::Connected).await;

    session
        .write(Bytes::from(format!("cd '{}'\n", dir.display())))
        .await
        .expect("write failed");

    let want = dir.to_string_lossy().into_owned();
    timeout(Duration::from_secs(5), async {
        loop {
            match rx.recv().await.expect("session event stream ended unexpectedly") {
                SessionEvent::CwdChanged { path } if path == want => return,
                _ => {}
            }
        }
    })
    .await
    .expect("never saw a CwdChanged event for the new directory");

    session.shutdown().await.expect("shutdown failed");
}

/// The command runner runs out of band, in the given directory, and hands
/// back the exit code - what a folder action's status check relies on.
#[cfg(unix)]
#[tokio::test]
async fn shell_session_runs_commands_out_of_band() {
    let dir = std::env::temp_dir().join(format!("hop-runner-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("marker"), "").expect("write marker");
    let cwd = dir.to_string_lossy().into_owned();

    let mut session = ShellSession::new(ShellConnectOptions { shell_command: Some("/bin/sh".to_string()), ..Default::default() });
    let (tx, mut rx) = mpsc::unbounded_channel();
    session.start(tx).await.expect("session failed to start");
    wait_for_state(&mut rx, SessionState::Connected).await;

    let runner = session.command_runner().expect("a local shell should have a command runner");
    assert_eq!(runner.run("test -f marker", &cwd).await.expect("run failed"), 0);
    assert_eq!(runner.run("test -f missing", &cwd).await.expect("run failed"), 1);
    assert_eq!(runner.run("exit 7", &cwd).await.expect("run failed"), 7);

    session.shutdown().await.expect("shutdown failed");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A shell launched with arguments of its own (e.g. `wsl.exe -d Ubuntu`)
/// gets no runner: `<shell> <args> -c ...` wouldn't mean what it does to a
/// plain shell.
#[tokio::test]
async fn shell_with_args_has_no_command_runner() {
    let session = ShellSession::new(ShellConnectOptions {
        shell_command: Some("/bin/sh".to_string()),
        shell_args: Some(vec!["-l".to_string()]),
        ..Default::default()
    });
    assert!(session.command_runner().is_none());
}
