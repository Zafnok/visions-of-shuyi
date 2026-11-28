//! End-to-end tests that run the real `xtask` binary as a subprocess, so
//! `main` itself (which reads the process's actual argv) is exercised.

use std::process::Command;

fn xtask() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
}

#[test]
fn no_command_prints_usage_and_exits_2() {
    let output = xtask()
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("usage: cargo xtask"));
}

#[test]
fn ticket_lint_on_the_real_repo_exits_0() {
    let output = xtask()
        .arg("ticket-lint")
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("ticket-lint: OK"));
}

// `--offline`: `--check` alone asks GitHub about the ledger's last section,
// which CI's own step does with a token, and no test reaches the network.
#[test]
fn engine_pin_check_on_the_real_repo_exits_0() {
    let output = xtask()
        .args(["engine-pin", "--check", "--offline"])
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("engine-pin: OK, at "));
}

/// A command that moved to the game's tool program still answers by its
/// old name (ADR-0129): xtask runs it in `trpg-tools`, which cargo builds
/// first.
#[test]
fn a_moved_command_answers_through_xtask() {
    let output = xtask()
        .args(["playtest", "--help"])
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("usage: cargo xtask playtest"), "{stdout}");
}

/// A file in the system's temporary folder, unique to this test run.
fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("xtask-cli-{name}-{}", std::process::id()))
}

/// Runs `ticket-next 08` reading `prs` as the open PRs' files, and answers
/// its exit code, stdout and stderr.
fn ticket_next_with(prs: &str, name: &str) -> (Option<i32>, String, String) {
    let file = temp_path(name);
    std::fs::write(&file, prs).unwrap_or_else(|e| panic!("write {}: {e}", file.display()));
    let output = xtask()
        .args(["ticket-next", "08", "--pr-list"])
        .arg(&file)
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    let _ = std::fs::remove_file(&file);
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

/// Ticket 0131: a number an open PR adds is taken, and the PR is named. The
/// PR list is a fixture, so no test reaches the network.
#[test]
fn ticket_next_skips_the_number_an_open_pr_adds() {
    let (code, local, said) = ticket_next_with("[]", "no-prs");
    assert_eq!(code, Some(0), "{said}");
    assert!(local.len() == 4 && local.starts_with("08"), "{local}");
    assert_eq!(said, "");

    let prs = format!(
        r#"[{{"number": 402, "files": [{{"path": "tickets/open/{local}-x.md", "changeType": "ADDED"}}]}}]"#
    );
    let (code, next, said) = ticket_next_with(&prs, "one-pr");
    assert_eq!(code, Some(0), "{said}");
    let expected = local
        .parse::<u16>()
        .unwrap_or_else(|e| panic!("{local}: {e}"))
        + 1;
    assert_eq!(next, format!("{expected:04}"));
    assert_eq!(said, format!("{local} is taken by PR #402"));
}

/// Without `gh` on `PATH` the answer is the local one, with a warning.
#[test]
fn ticket_next_without_gh_warns_and_exits_0() {
    let empty = temp_path("empty-path");
    std::fs::create_dir_all(&empty).unwrap_or_else(|e| panic!("mkdir {}: {e}", empty.display()));
    let output = xtask()
        .args(["ticket-next", "08"])
        .env("PATH", &empty)
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    let _ = std::fs::remove_dir_all(&empty);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    let (_, local, _) = ticket_next_with("[]", "local");
    assert_eq!(stdout.trim(), local);
    assert!(
        stderr.contains("warning: open PRs not checked (can't run gh"),
        "{stderr}"
    );
}
