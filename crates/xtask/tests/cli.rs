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
