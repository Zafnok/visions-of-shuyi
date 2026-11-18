//! End-to-end tests that run the real `trpg-tools` binary as a subprocess,
//! so `main` itself (which reads the process's actual argv) is exercised.

use std::process::Command;

fn tools() -> Command {
    Command::new(env!("CARGO_BIN_EXE_trpg-tools"))
}

#[test]
fn no_command_prints_usage_and_exits_2() {
    let output = tools()
        .output()
        .unwrap_or_else(|e| panic!("spawn trpg-tools: {e}"));
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("usage: cargo xtask"));
}

#[test]
fn private_assets_with_bad_arguments_prints_usage_and_exits_2() {
    // Only bad arguments: a real run would fetch from the private repository
    // into this checkout's `assets-private/`.
    for bad in [&["--bogus"][..], &["--library", "--pin"]] {
        let output = tools()
            .arg("private-assets")
            .args(bad)
            .output()
            .unwrap_or_else(|e| panic!("spawn trpg-tools: {e}"));
        assert_eq!(output.status.code(), Some(2), "{bad:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).trim(),
            "usage: cargo shuyi private-assets [--library | --pin]"
        );
    }
}

#[test]
fn playtest_prints_the_report_and_exits_0() {
    let history =
        std::env::temp_dir().join(format!("trpg-tools-cli-playtest-{}", std::process::id()));
    let output = tools()
        .args(["playtest", "quick", "--runs", "3", "--history"])
        .arg(&history)
        .output()
        .unwrap_or_else(|e| panic!("spawn trpg-tools: {e}"));
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first =
        "quick · Normal tier · baseline · turn cap 60\narmy · fresh from the character data · ";
    assert!(stdout.starts_with(first), "{stdout}");
    assert!(
        stdout.contains(" · gold 3000\n3 tries (seeds 1–3)\n"),
        "{stdout}"
    );
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
fn shuyi_runs_the_engine_s_tools_with_the_game_s() {
    // `cargo shuyi` runs `trpg-tools shuyi` (tickets 1131, 0188): with no
    // command, the tools' usage, named after the project's `command`.
    let output = tools()
        .arg("shuyi")
        .output()
        .unwrap_or_else(|e| panic!("spawn trpg-tools: {e}"));
    assert_eq!(output.status.code(), Some(2));
    let said = String::from_utf8_lossy(&output.stdout);
    assert!(said.contains("cargo shuyi"), "{said}");
    assert!(said.contains("frame-png"), "{said}");
}

#[test]
fn autobalance_brings_the_too_hard_battle_into_its_bands() {
    // The engine's command on the game's plan (ticket 0509), without
    // `--apply`: the game's tests write the file into a temp folder.
    let output = tools()
        .args(["shuyi", "autobalance", "--stage", "test_too_hard"])
        .args([
            "--tries", "20", "--quick", "10", "--budget", "40", "--turns", "20",
        ])
        .output()
        .unwrap_or_else(|e| panic!("spawn trpg-tools: {e}"));
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
