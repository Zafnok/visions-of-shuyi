//! `cargo xtask bot-wins` (ticket 0531): records the bots' own wins,
//! `crates/bots/wins/<battle>.ron`, the lines the bots' armies come from
//! (`docs/playtesting.md`, *The army*). Each line is recorded on a thread
//! of its own, battle after battle in story order, each from the army that
//! line's wins before it leave.

use std::path::Path;

use trpg_bots::{LINES, Recorded, Wins, file_text, record, start_of, won_on_the_way};
use trpg_content::Content;
use trpg_core::{BattleState, GameMode, LeadGender, LeadProfile};

use crate::playtest::LEAD_NAME;

const USAGE: &str = "usage: cargo xtask bot-wins (<battle-id> | --all) [options]

Records the bots' own winning playthroughs, crates/bots/wins/<battle>.ron:
for every battle a later battle's army plays (each chapter's guards, then
its story battle), one per line. A line is one player's way through the
story: its win of a battle is recorded from the army its wins of the
battles before leave. The bots field every line's army, a try each
(docs/playtesting.md, *The army*).

  <battle-id>     that battle and every one after it on the way (their armies
                  change with it); the ones before keep their wins
  --all           every battle on the way

options:
  --lines <n>     lines to record (default 24)
  --seed <n>      the search's seed for line 1; line i gets seed + i - 1
                  (default 1)
  --out <dir>     where to write the files (default crates/bots/wins), for
                  example a scratch folder to compare two sets of seeds";

/// What to record.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    /// The first battle to record: `None` for `--all`.
    from: Option<String>,
    lines: usize,
    seed: u64,
    out: Option<String>,
}

/// Runs the command in repository `root`: 0 when every line won every
/// battle, 1 when one didn't, 2 for a usage error.
pub fn command(root: &Path, args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return 0;
    }
    let options = match parse(args) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return 2;
        }
    };
    let run = || -> Result<String, String> {
        let content = trpg_content::load_embedded().map_err(|e| format!("content: {e}"))?;
        let embedded = Wins::embedded()?;
        let files = record_lines(&content, embedded, &options)?;
        let out = options.out.as_deref().unwrap_or("crates/bots/wins");
        let dir = root.join(out);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut said = Vec::new();
        for (battle, text) in files {
            let path = dir.join(format!("{battle}.ron"));
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            said.push(format!("wrote {out}/{battle}.ron"));
        }
        Ok(said.join("\n"))
    };
    crate::report(run())
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        from: None,
        lines: LINES,
        seed: 1,
        out: None,
    };
    let mut all = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match arg.as_str() {
            "--all" => all = true,
            "--lines" => {
                let n = value("--lines")?;
                options.lines = n
                    .parse()
                    .map_err(|_| format!("--lines {n}: not a number"))?;
            }
            "--seed" => {
                let n = value("--seed")?;
                options.seed = n.parse().map_err(|_| format!("--seed {n}: not a number"))?;
            }
            "--out" => options.out = Some(value("--out")?),
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            battle if options.from.is_none() => options.from = Some(battle.to_owned()),
            extra => return Err(format!("one battle at a time, not also {extra}")),
        }
    }
    if options.lines == 0 {
        return Err("--lines 0 records nothing".to_owned());
    }
    match (all, options.from.is_some()) {
        (true, true) => Err("a battle or --all, not both".to_owned()),
        (false, false) => Err("name the battle to record, or --all".to_owned()),
        _ => Ok(options),
    }
}

/// The files of the battles `options` records, as (battle, text), in story
/// order: each line played on a thread of its own, its battles before
/// `options.from` won by `embedded`'s lines.
fn record_lines(
    content: &Content,
    embedded: &Wins,
    options: &Options,
) -> Result<Vec<(String, String)>, String> {
    let on_the_way = won_on_the_way(content);
    let start = first_to_record(&on_the_way, options.from.as_deref())?;
    let battles = &on_the_way[start..];
    let lines: Vec<Result<Vec<(u64, Recorded)>, String>> = std::thread::scope(|scope| {
        let threads: Vec<_> = (0..options.lines)
            .map(|line| {
                let seed = line_seed(options.seed, line);
                scope.spawn(move || record_line(content, embedded, battles, line, seed))
            })
            .collect();
        threads
            .into_iter()
            .map(|t| {
                t.join()
                    .unwrap_or_else(|_| Err("a line's thread panicked".to_owned()))
            })
            .collect()
    });
    let lines = lines.into_iter().collect::<Result<Vec<_>, String>>()?;
    battles
        .iter()
        .enumerate()
        .map(|(i, battle)| {
            let of_battle: Vec<(u64, Recorded)> = lines.iter().map(|l| l[i].clone()).collect();
            Ok((battle.clone(), file_text(battle, &of_battle)?))
        })
        .collect()
}

/// How many seeds a line's search tries at one battle before the line
/// fails: a search can wander past the turn cap (Chapter 4 did, 3 times in
/// 16 lines).
const SEARCHES: u64 = 3;

/// How far apart a battle's next search seed is from the one before: past
/// any other line's seed.
const SEED_STEP: u64 = 1000;

/// Where in `on_the_way` (the battles later armies play, in story order)
/// recording starts: at battle `from`, or the first.
fn first_to_record(on_the_way: &[String], from: Option<&str>) -> Result<usize, String> {
    let Some(battle) = from else {
        return Ok(0);
    };
    on_the_way.iter().position(|b| b == battle).ok_or_else(|| {
        format!(
            "no later battle's army plays \"{battle}\" (those that do: {})",
            on_the_way.join(", ")
        )
    })
}

/// The search's seed of line `line` (from 0) when line 1's is `first`.
fn line_seed(first: u64, line: usize) -> u64 {
    first + u64::try_from(line).unwrap_or(0)
}

/// The seed of a line's `attempt`th search (from 0) at a battle, its first
/// `seed`: [`SEED_STEP`] apart.
fn search_seed(seed: u64, attempt: u64) -> u64 {
    seed + attempt * SEED_STEP
}

/// Line `line`'s wins of `battles`, in order, its search seeded `seed`:
/// each from the army its wins before leave, the ones before the first of
/// `battles` `embedded`'s. A search that finds no win is tried again with
/// `seed + 1000`, then `seed + 2000`; the file says which seed won.
fn record_line(
    content: &Content,
    embedded: &Wins,
    battles: &[String],
    line: usize,
    seed: u64,
) -> Result<Vec<(u64, Recorded)>, String> {
    let mut wins = embedded.clone();
    let lead = LeadProfile::new(LEAD_NAME, LeadGender::Male);
    let mut recorded = Vec::new();
    for battle in battles {
        let def = content.battles.get(battle);
        let def = def.ok_or_else(|| format!("no battle \"{battle}\""))?;
        let setup = start_of(content, def, GameMode::Classic, lead.clone(), &wins, line)?;
        let setup = setup.ok_or_else(|| format!("no story chapter fights \"{battle}\""))?;
        let mut failed = Vec::new();
        let won = (0..SEARCHES).find_map(|i| {
            let seed = search_seed(seed, i);
            match record(BattleState::new(setup.clone()).0, content, seed, &|_| true) {
                Ok(won) => Some((seed, won)),
                Err(e) => {
                    eprintln!("line {} · {battle} · seed {seed}: {e}", line + 1);
                    failed.push(format!("seed {seed}: {e}"));
                    None
                }
            }
        });
        let Some((seed, won)) = won else {
            return Err(format!(
                "line {} of {battle}: {}",
                line + 1,
                failed.join("; ")
            ));
        };
        eprintln!("line {} · {battle} · {}", line + 1, won.summary);
        wins.set(battle, line, won.commands());
        recorded.push((seed, won));
    }
    Ok(recorded)
}

#[cfg(test)]
mod tests;
