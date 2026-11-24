//! The bots' own recorded wins (ticket 0531, ADR-0130): for each battle a
//! later battle's army plays, [`LINES`] winning playthroughs, one per
//! *line*. A line is one player's way through the story: its win of a
//! battle is recorded with the army its wins of the battles before leave,
//! so line 3 of Chapter 2 starts where line 3 of Chapter 1 ended. The bots
//! field every line's army ([`arrival`](crate::arrival)), each try one of
//! them ([`line_of`]), so no single recording's luck sets a chapter's
//! army (Nick, 2026-11-21: "Several playthroughs").
//!
//! The files are `crates/bots/wins/<battle>.ron`, written by `cargo xtask
//! bot-wins` and compiled in by the build script: a battle's wins are found
//! by its id. They are the bots' own, apart from `trpg-ui`'s replay
//! fixtures, so re-recording one of those moves no army.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use trpg_core::Command;

use crate::record::Recorded;

/// How many lines the bots field: each battle's wins hold this many. Nick
/// picked "about 8 recorded wins per chapter", then 24 when two sets of 8
/// still left Chapter 2's Normal 14 points apart (2026-11-22).
pub const LINES: usize = 24;

/// Every `crates/bots/wins/<battle>.ron`, as (battle, text), by battle.
const FILES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/wins.rs"));

/// Each battle's recorded wins, by battle id: line 1's first.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Wins(BTreeMap<String, Vec<Vec<Command>>>);

impl Wins {
    /// The wins compiled in, `crates/bots/wins/`.
    pub fn embedded() -> Result<&'static Wins, String> {
        static WINS: OnceLock<Result<Wins, String>> = OnceLock::new();
        let wins = WINS.get_or_init(|| {
            let mut wins = Wins::default();
            for (battle, text) in FILES {
                wins.insert(*battle, parse(battle, text)?);
            }
            Ok(wins)
        });
        wins.as_ref().map_err(Clone::clone)
    }

    /// Battle `battle`'s wins become `lines`.
    pub fn insert(&mut self, battle: impl Into<String>, lines: Vec<Vec<Command>>) {
        self.0.insert(battle.into(), lines);
    }

    /// Line `line` (from 0) of battle `battle` becomes `commands`; a line
    /// before it that the battle hasn't got is left empty.
    pub fn set(&mut self, battle: &str, line: usize, commands: Vec<Command>) {
        let lines = self.0.entry(battle.to_owned()).or_default();
        if lines.len() <= line {
            lines.resize(line + 1, Vec::new());
        }
        lines[line] = commands;
    }

    /// How many lines battle `battle` has: 0 when it has none.
    pub fn lines(&self, battle: &str) -> usize {
        self.0.get(battle).map_or(0, Vec::len)
    }

    /// The player's commands of line `line` (from 0) of battle `battle`.
    pub fn line(&self, battle: &str, line: usize) -> Result<&[Command], String> {
        let lines = self.0.get(battle).ok_or_else(|| {
            format!(
                "no recorded wins of \"{battle}\": `cargo xtask bot-wins {battle}` records them"
            )
        })?;
        let commands = lines.get(line).ok_or_else(|| {
            format!(
                "\"{battle}\" has {} recorded lines, not {}: `cargo xtask bot-wins {battle}`",
                lines.len(),
                line + 1
            )
        })?;
        Ok(commands)
    }
}

/// The line (from 0) a try with seed `seed` fields, of `lines`: `(seed − 1)
/// mod lines`, so with 24 lines tries 1 to 24 field lines 1 to 24, try 25
/// line 1 again, and `--seed 17 --runs 1` plays try 17's army again.
pub fn line_of(seed: u64, lines: usize) -> usize {
    let lines = u64::try_from(lines.max(1)).unwrap_or(1);
    usize::try_from(seed.wrapping_sub(1) % lines).unwrap_or(0)
}

/// The lines of file text `text`, battle `battle`'s.
pub fn parse(battle: &str, text: &str) -> Result<Vec<Vec<Command>>, String> {
    ron::from_str(text).map_err(|e| format!("crates/bots/wins/{battle}.ron: {e}"))
}

/// The text of battle `battle`'s file: its lines, each with the seed its
/// search was given and how it ended, turn by turn.
pub fn file_text(battle: &str, lines: &[(u64, Recorded)]) -> Result<String, String> {
    let mut out = vec![
        format!("// The bots' recorded wins of battle {battle} (ticket 0531): one per line, the"),
        "// player's commands phase by phase, each from the army that line's wins of the"
            .to_owned(),
        "// battles before leave (docs/playtesting.md, *The army*). Written by".to_owned(),
        "// `cargo xtask bot-wins`: never edit it by hand.".to_owned(),
        "[".to_owned(),
    ];
    for (i, (seed, recorded)) in lines.iter().enumerate() {
        out.push(format!(
            "    // Line {} (seed {seed}): {}",
            i + 1,
            recorded.summary
        ));
        out.push("    [".to_owned());
        for (turn, commands) in recorded.phases.iter().enumerate() {
            out.push(format!("        // Turn {}", turn + 1));
            for command in commands {
                let text = ron::to_string(command).map_err(|e| e.to_string())?;
                out.push(format!("        {text},"));
            }
        }
        out.push("    ],".to_owned());
    }
    out.extend(["]".to_owned(), String::new()]);
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests;
