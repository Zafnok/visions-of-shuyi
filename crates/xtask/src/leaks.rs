//! `cargo xtask check-leaks` (ticket 0157, ADR-0118): no tracked file
//! quotes a line of the game's scripts or draws a row of its maps.
//!
//! The lines and rows are read from the game's files as the build embeds
//! them ([`content_layer`]): the private repository's content over
//! `assets/`. Each tracked file is then searched for
//!
//! - any run of [`PHRASE`] words of a line (a caption, a speech, a
//!   narration or a reply), as words: case, punctuation and line breaks
//!   aside, so a line wrapped in a text box of a snapshot is found;
//! - a whole line shorter than that, of at least [`SHORT`] words, as it is
//!   written;
//! - a whole row of a map (its grid, after `---`) of at least [`ROW`]
//!   characters and three kinds of tile.
//!
//! A token (`{n:keeper}`) splits a line: its parts are searched apart, as
//! the screen fills the token in. The files the content's move takes away
//! ([`MOVING`]) and the tickets are not searched. What is found is named by
//! place only, the file and line of the quote and of its source: the
//! check's output never quotes the line itself, as CI's logs are public.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::content_layer::{self, STORY};

/// How many words in a row of a line make a quote: fewer find common
/// phrases ("at the end of the") all over the documents.
pub const PHRASE: usize = 7;

/// The fewest words of a line shorter than [`PHRASE`] that is looked for
/// whole.
pub const SHORT: usize = 4;

/// The fewest characters of a map row that is looked for.
pub const ROW: usize = 12;

/// What is not searched, by path prefix: the tickets (0153 §3.5), and the
/// files tickets 0159 and 0160 move into the private repository, which
/// hold the content until they do. A `README.md` in `assets/` is always
/// searched: the formats stay public. Once 0160 has moved the files, the
/// `assets/` entries name nothing and 0160 removes them.
pub const MOVING: &[&str] = &[
    "tickets/",
    // 0159: the story's documents.
    "docs/story/",
    "assets-src/audio/voice-audition.html",
    // 0160: the game's content.
    "assets/dialogue/",
    "assets/chapters/",
    "assets/battles/",
    "assets/maps/",
    "assets/worldmaps/",
    "assets/cinematics/",
    "assets/openings/",
    "assets/skirmishes/",
];

/// The game's lines and map rows, each with the place it comes from
/// (`assets/dialogue/ch01.dlg:72`).
#[derive(Debug, Default)]
pub struct Sources {
    /// Every run of [`PHRASE`] words of a line.
    phrases: BTreeMap<Vec<String>, String>,
    /// Every line (or part of one between tokens) of [`SHORT`] words or
    /// more but fewer than [`PHRASE`], as written.
    short: BTreeMap<String, String>,
    /// Every map row of [`ROW`] characters or more.
    rows: BTreeMap<String, String>,
}

impl Sources {
    /// Reads the scripts (`dialogue/*.dlg`) and maps (`maps/*.map`) of the
    /// game's files under `root`, the content's over `assets/`'s. A folder
    /// neither has is read as empty.
    pub fn read(root: &Path) -> Result<Self, String> {
        let mut sources = Self::default();
        for (shown, path) in files(root, "dialogue", ".dlg")? {
            let text = read(&path)?;
            sources.add_script(&shown, &text);
        }
        for (shown, path) in files(root, "maps", ".map")? {
            let text = read(&path)?;
            sources.add_map(&shown, &text);
        }
        Ok(sources)
    }

    /// Adds the lines of the script `text`, shown as `shown`.
    pub fn add_script(&mut self, shown: &str, text: &str) {
        for (number, line) in text.lines().enumerate() {
            let Some(said) = said(line) else { continue };
            let from = format!("{shown}:{}", number + 1);
            for part in parts(said) {
                let words = words(part);
                if words.len() >= PHRASE {
                    for run in words.windows(PHRASE) {
                        self.phrases
                            .entry(run.to_vec())
                            .or_insert_with(|| from.clone());
                    }
                } else if words.len() >= SHORT {
                    self.short
                        .entry(part.trim().to_owned())
                        .or_insert_with(|| from.clone());
                }
            }
        }
    }

    /// Adds the rows of the map `text`, shown as `shown`.
    pub fn add_map(&mut self, shown: &str, text: &str) {
        let mut grid = false;
        for (number, line) in text.lines().enumerate() {
            if !grid {
                grid = line.trim_end() == "---";
                continue;
            }
            let row = line.trim_end();
            let kinds: BTreeSet<char> = row.chars().collect();
            if row.chars().count() >= ROW && kinds.len() >= 3 {
                self.rows
                    .entry(row.to_owned())
                    .or_insert_with(|| format!("{shown}:{}", number + 1));
            }
        }
    }

    /// Whether there is nothing to look for (no content at all).
    pub fn is_empty(&self) -> bool {
        self.phrases.is_empty() && self.short.is_empty() && self.rows.is_empty()
    }

    /// Where `text` quotes a line or a map row: each source once, with the
    /// line of `text` (from 1) its first quote starts on, in order.
    pub fn find(&self, text: &str) -> Vec<(usize, String)> {
        let mut first: BTreeMap<String, usize> = BTreeMap::new();
        let mut quoted = |from: &String, line: usize| {
            let at = first.entry(from.clone()).or_insert(line);
            *at = (*at).min(line);
        };
        let tokens = numbered_words(text);
        for run in tokens.windows(PHRASE) {
            let words: Vec<String> = run.iter().map(|(_, w)| w.clone()).collect();
            if let Some(from) = self.phrases.get(&words) {
                quoted(from, run[0].0);
            }
        }
        for (number, line) in text.lines().enumerate() {
            let whole = self.short.iter().chain(&self.rows);
            for (_, from) in whole.filter(|(said, _)| line.contains(said.as_str())) {
                quoted(from, number + 1);
            }
        }
        let mut found: Vec<(usize, String)> =
            first.into_iter().map(|(from, line)| (line, from)).collect();
        found.sort();
        found
    }
}

/// The files directly in `dir` of the game's files under `root` whose
/// name ends in `ext`, the content's over `assets/`'s, as
/// [`content_layer::files_in`] lists them; a folder neither has is empty.
fn files(root: &Path, dir: &str, ext: &str) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let roots: Vec<&str> = STORY
        .into_iter()
        .filter(|layer| root.join(layer).join(dir).is_dir())
        .collect();
    content_layer::files_in(root, &roots, dir, ext)
}

/// The text of the file at `path`.
fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))
}

/// What the script line `line` puts on screen: a caption, a speech, a
/// narration or a reply, or a continuation of one; `None` for a comment,
/// a directive or a blank line.
fn said(line: &str) -> Option<&str> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    if let Some(caption) = line.strip_prefix("@caption ") {
        return Some(caption);
    }
    if line.starts_with('@') {
        return None;
    }
    if let Some(narration) = line.strip_prefix('>') {
        return Some(after_direction(narration.trim_start()));
    }
    if let Some(reply) = line.strip_prefix('*') {
        return Some(reply.split_once(':').map_or(reply, |(_, text)| text));
    }
    Some(speech(line).unwrap_or(line))
}

/// The text of the speech `line` (`keeper[sad] (quietly): text`), or
/// `None` if it isn't one.
fn speech(line: &str) -> Option<&str> {
    let (who, text) = line.split_once(':')?;
    let who = who.split_once(" (").map_or(who, |(who, _)| who);
    let who = who.split_once('[').map_or(who, |(who, _)| who);
    let id = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-');
    (!who.is_empty() && who.chars().all(id)).then_some(text)
}

/// `text` without a leading direction (`(quietly): `).
fn after_direction(text: &str) -> &str {
    text.strip_prefix('(')
        .and_then(|rest| rest.split_once("):"))
        .map_or(text, |(_, after)| after)
}

/// The parts of `said` between its tokens (`{n:keeper}`, `{lead|the exile}`).
fn parts(said: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = said;
    while let Some(open) = rest.find('{') {
        parts.push(&rest[..open]);
        rest = rest[open..].split_once('}').map_or("", |(_, after)| after);
    }
    parts.push(rest);
    parts
}

/// The words of `text`: lowercase runs of letters, digits and apostrophes,
/// without an apostrophe at either end.
fn words(text: &str) -> Vec<String> {
    numbered_words(text).into_iter().map(|(_, w)| w).collect()
}

/// [`words`], each with the line of `text` it is on (from 1).
fn numbered_words(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.replace('\u{2019}', "'").to_lowercase();
        let runs = line.split(|c: char| !(c.is_alphanumeric() || c == '\''));
        for word in runs.map(|w| w.trim_matches('\'')).filter(|w| !w.is_empty()) {
            out.push((number + 1, word.to_owned()));
        }
    }
    out
}

/// Whether the tracked file `path` is searched: not under [`MOVING`], but
/// a `README.md`.
pub fn searched(path: &str) -> bool {
    path.rsplit('/').next() == Some("README.md")
        || !MOVING.iter().any(|prefix| path.starts_with(prefix))
}

/// Each quote in the tracked files of `root` of what `sources` holds, as
/// `path:line quotes source`, in order. A file that isn't text is skipped.
pub fn leaks(root: &Path, tracked: &[String], sources: &Sources) -> Vec<String> {
    let mut found = Vec::new();
    for path in tracked.iter().filter(|path| searched(path)) {
        let Ok(text) = fs::read_to_string(root.join(path)) else {
            continue;
        };
        for (line, from) in sources.find(&text) {
            found.push(format!("{path}:{line} quotes {from}"));
        }
    }
    found
}

/// The tracked files of the repository at `root`.
fn tracked(root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|e| format!("can't run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let list = String::from_utf8_lossy(&out.stdout);
    Ok(list
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect())
}

/// `cargo xtask check-leaks`: 0 when no tracked file of `root` quotes the
/// game's lines or map rows, 1 when one does, 2 when the files can't be
/// read.
pub fn command(root: &Path, args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask check-leaks");
        return 2;
    }
    let sources = match Sources::read(root) {
        Ok(sources) => sources,
        Err(e) => {
            eprintln!("check-leaks: {e}");
            return 2;
        }
    };
    if sources.is_empty() {
        println!("check-leaks: no content here to look for (cargo shuyi private-assets)");
        return 0;
    }
    let tracked = match tracked(root) {
        Ok(tracked) => tracked,
        Err(e) => {
            eprintln!("check-leaks: {e}");
            return 2;
        }
    };
    let found = leaks(root, &tracked, &sources);
    if found.is_empty() {
        println!(
            "check-leaks: OK, {} tracked files quote no line or map row of the game's",
            tracked.iter().filter(|path| searched(path)).count()
        );
        return 0;
    }
    eprintln!(
        "check-leaks: {} quote(s) of the game's content (ADR-0118)",
        found.len()
    );
    for leak in &found {
        eprintln!("  {leak}");
    }
    eprintln!(
        "a public file may name a content id, but never quote a line, draw a real map or \
         describe the plot: point the test at assets-test/, or read the line from the content"
    );
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_layer::tests::repo;

    /// A script of the made-up story the tests look for.
    const SCRIPT: &str = "\
# A comment is never looked for: the lantern swings over the cold water.
@scene made_up
@caption The mill at Underbridge, after dark
@left miller neutral
> The lantern swings over the cold water and nobody says a word.
miller[sad] (quietly): Grind it fine, {n:baker}, the way your mother ground it every winter.
* earnest: Keep the wheel turning.
  miller: You will see the flour come down like snow from the loft above.
@end
";

    /// A map of the made-up story.
    const MAP: &str = "(\n    name: \"Underbridge\",\n)\n---\n..TT~~~~..::..\n....\n";

    fn sources() -> Sources {
        let mut sources = Sources::default();
        sources.add_script("assets/dialogue/made_up.dlg", SCRIPT);
        sources.add_map("assets/maps/made_up.map", MAP);
        sources
    }

    #[test]
    fn a_line_wrapped_in_a_text_box_is_found_and_named_by_its_source() {
        let snapshot = "\
┌──────────────────────────────┐
│ The lantern swings over the  │
│ cold water and nobody says a │
│ word.                        │
└──────────────────────────────┘
";
        assert_eq!(
            sources().find(snapshot),
            [(2, "assets/dialogue/made_up.dlg:5".to_owned())]
        );
    }

    #[test]
    fn a_speech_a_reply_and_a_reaction_are_lines_but_not_a_comment() {
        let sources = sources();
        let speech = "let line = \"the way your mother ground it every winter\";";
        assert_eq!(
            sources.find(speech),
            [(1, "assets/dialogue/made_up.dlg:6".to_owned())]
        );
        let reaction = "see the flour come down like snow";
        assert_eq!(
            sources.find(reaction),
            [(1, "assets/dialogue/made_up.dlg:8".to_owned())]
        );
        // The comment's words are not, nor the speaker's id and direction
        // run into the line.
        assert_eq!(sources.find("a comment is never looked for"), []);
        assert_eq!(sources.find("sad quietly grind it fine the way"), []);
    }

    #[test]
    fn a_short_line_and_a_caption_are_found_whole_as_written() {
        let sources = sources();
        assert_eq!(
            sources.find("> Keep the wheel turning."),
            [(1, "assets/dialogue/made_up.dlg:7".to_owned())]
        );
        assert_eq!(
            sources.find("The mill at Underbridge, after dark"),
            [(1, "assets/dialogue/made_up.dlg:3".to_owned())]
        );
        // A short line in other words, or cased otherwise, is not a quote.
        assert_eq!(sources.find("keep the wheel turning"), []);
        assert_eq!(sources.find("Keep turning the wheel."), []);
    }

    #[test]
    fn a_token_splits_a_line_into_parts_looked_for_apart() {
        // "Grind it fine," is short and the token cuts it from the rest.
        assert_eq!(sources().find("Grind it fine, Bob, the way"), []);
    }

    #[test]
    fn a_map_row_is_found_but_not_a_short_or_plain_one() {
        let sources = sources();
        assert_eq!(
            sources.find("let map = \"..TT~~~~..::..\";"),
            [(1, "assets/maps/made_up.map:5".to_owned())]
        );
        assert_eq!(sources.find("...."), []);
        // The map's header is not a row.
        assert_eq!(sources.find("name: \"Underbridge\","), []);
    }

    #[test]
    fn a_made_up_line_is_not_found() {
        let other = "The lantern swings over the warm water and everybody sings.";
        assert_eq!(sources().find(other), []);
    }

    #[test]
    fn the_moving_files_and_tickets_are_not_searched_but_a_readme_is() {
        for path in [
            "tickets/open/0160-x.md",
            "docs/story/made_up.md",
            "assets/dialogue/ch01.dlg",
            "assets/maps/ch01.map",
        ] {
            assert!(!searched(path), "{path}");
        }
        for path in [
            "assets/made_up/README.md",
            "crates/ui/tests/it/snapshots/it__x.snap",
            "docs/design/made_up.md",
        ] {
            assert!(searched(path), "{path}");
        }
    }

    #[test]
    fn a_leak_in_a_tracked_file_is_found_with_the_contents_files_over_assets() {
        let root = repo(
            "leaks",
            &[
                (
                    "assets/dialogue/made_up.dlg",
                    "@scene a\n> Public words.\n@end\n",
                ),
                ("assets-private/content/dialogue/made_up.dlg", SCRIPT),
                ("assets/maps/made_up.map", MAP),
                (
                    "docs/notes.md",
                    "nobody says a word.\nThe lantern swings over the cold water and\n",
                ),
                (
                    "tickets/open/0001-x.md",
                    "The lantern swings over the cold water and nobody",
                ),
                ("docs/clean.md", "The mill grinds."),
            ],
        );
        let sources = Sources::read(&root).unwrap();
        let tracked: Vec<String> = ["docs/notes.md", "tickets/open/0001-x.md", "docs/clean.md"]
            .map(str::to_owned)
            .to_vec();
        assert_eq!(
            leaks(&root, &tracked, &sources),
            ["docs/notes.md:2 quotes assets-private/content/dialogue/made_up.dlg:5"]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn without_any_content_there_is_nothing_to_look_for() {
        let root = repo("leaks-none", &[("docs/notes.md", "anything")]);
        assert!(Sources::read(&root).unwrap().is_empty());
        assert_eq!(command(&root, &[]), 0);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_command_takes_no_arguments() {
        assert_eq!(command(Path::new("."), &["x".to_owned()]), 2);
    }
}
