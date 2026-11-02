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

#[test]
fn private_assets_with_bad_arguments_prints_usage_and_exits_2() {
    // Only bad arguments: a real run would fetch from the private repository
    // into this checkout's `assets-private/`.
    for bad in [&["--bogus"][..], &["--library", "--pin"]] {
        let output = xtask()
            .arg("private-assets")
            .args(bad)
            .output()
            .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
        assert_eq!(output.status.code(), Some(2), "{bad:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).trim(),
            "usage: cargo shuyi private-assets [--library | --pin]"
        );
    }
}

#[test]
fn playtest_prints_the_report_and_exits_0() {
    let history = std::env::temp_dir().join(format!("xtask-cli-playtest-{}", std::process::id()));
    let output = xtask()
        .args(["playtest", "quick", "--runs", "3", "--history"])
        .arg(&history)
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first = "quick · Normal tier · baseline · turn cap 60\n3 tries (seeds 1–3)\n";
    assert!(stdout.starts_with(first), "{stdout}");
    assert!(stdout.contains("\nbaseline · Classic · won "), "{stdout}");
    assert!(stdout.contains("\n\ntry 1 · baseline · "), "{stdout}");
    assert!(stdout.contains("\ntry 3 · baseline · "), "{stdout}");
    assert!(
        stdout.ends_with('\n') && !stdout.ends_with("\n\n"),
        "{stdout}"
    );
    let _ = std::fs::remove_dir_all(&history);
}

#[test]
fn engine_pin_check_on_the_real_repo_exits_0() {
    let output = xtask()
        .args(["engine-pin", "--check"])
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

#[test]
fn shuyi_runs_the_engine_s_tools_with_the_game_s() {
    // `cargo shuyi` is `cargo xtask shuyi` (ticket 1131): with no command,
    // the tools' usage, named after the project's `command`.
    let output = xtask()
        .arg("shuyi")
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    assert_eq!(output.status.code(), Some(2));
    let said = String::from_utf8_lossy(&output.stdout);
    assert!(said.contains("cargo shuyi"), "{said}");
    assert!(said.contains("frame-png"), "{said}");
}

#[test]
fn autobalance_brings_the_too_hard_battle_into_its_bands() {
    // The engine's command on the game's plan (ticket 0509), without
    // `--apply`: the game's tests write the file into a temp folder.
    let output = xtask()
        .args(["shuyi", "autobalance", "--stage", "test_too_hard"])
        .args([
            "--tries", "20", "--quick", "10", "--budget", "40", "--turns", "20",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn xtask: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(
        stdout.starts_with("test_too_hard · Normal tier · SideQuest\n"),
        "{stdout}"
    );
    // Its pack's Potion heals 20 (0443), so -5 levels is enough.
    assert!(stdout.contains("every unnamed enemy -5 levels"), "{stdout}");
    assert!(stdout.contains("\nafter: 20 tries"), "{stdout}");
}
