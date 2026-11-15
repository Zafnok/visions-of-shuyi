//! The writer search's trial (ticket 0766): `cargo xtask trial-packet --for
//! <handle>` builds one applicant's packet, and `cargo xtask
//! trial-compare` lays the entries that came back side by side for Nick to
//! judge without names.
//!
//! A packet is a zip of text files and no program (Nick, 2026-11-14:
//! applicants may balk at running an `.exe`): the trial's scenes cut from
//! Chapter 1's script in `dialogue/`, the pages `docs/writer-trial/packet.txt`
//! lists, and the links of the web demo and the web script preview (ticket
//! 0767), where the writer drops the `.dlg` file to see it. Every
//! packet carries its applicant's mark (the first line of its script, its
//! `START-HERE.txt` and its zip's name), so a leaked packet can be traced.
//! Everything it writes is in `dist/trial/`, which is never committed.

use std::collections::BTreeMap;
use std::collections::hash_map::RandomState;
use std::fmt::Write as _;
use std::fs;
use std::hash::BuildHasher;
use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;

/// The scenes an applicant rewrites, in the order they play: everything
/// Chapter 1 shows before and after its battle, and the battle's first
/// turn (Nick, 2026-11-14: "all 3", and the scenes after the battle).
pub const SCENES: &[&str] = &[
    "ch01_intro",
    "ch01_prebattle",
    "ch01_first_turn",
    "ch01_victory",
    "ch01_tbc",
];
/// Where the scenes are cut from, in the private checkout.
const SCRIPT: &str = "assets-private/content/dialogue/ch01.dlg";
/// The names table, in the private checkout, and its public stand-in.
const NAMES: &[&str] = &[
    "assets-private/content/data/names.ron",
    "assets/data/names.ron",
];
/// What else goes in a packet, and where from.
const PACKET_LIST: &str = "docs/writer-trial/packet.txt";
/// The page that opens a packet, with its links still to fill in.
const START_HERE: &str = "docs/writer-trial/start-here.txt";
/// The reading guide at the top of the comparison page.
const READING_GUIDE: &str = "docs/writer-trial/reading-guide.md";
/// The writer's VS Code folder (ticket 0768), which every packet waits for.
const EDITOR: &str = "tools/vscode-dlg/package.json";
/// The story documents' folder in the private checkout. Paths in it are
/// built from it, so no literal names one of its Markdown files (the
/// repository test of the Markdown the code reads would take one for a
/// public file the tools read).
const STORY: &str = "assets-private/story/";
/// Where everything is written; never committed (`.gitignore`'s `/dist/`).
const OUT: &str = "dist/trial";

pub const PACKET_USAGE: &str = "usage: cargo xtask trial-packet --for <handle> --due <date>\n\n\
builds dist/trial/<handle>.zip, the writer trial's packet for one applicant\n\
(ticket 0766): the trial's scenes, the brief, the terms, the story pages and\n\
START-HERE.txt with the links. Text files only, no program.\n\
  --for <handle>  the applicant's mark: letters, digits, - and _\n\
  --due <date>    when their entry is due, as you agreed it\n\
The web demo's and web preview's links and passwords come from\n\
dist/trial/links.txt (one `name = value` a line: demo, demo-password,\n\
preview, preview-password). Anything in dist/trial/extra/ (the signed\n\
agreement, say) goes in too.";

pub const COMPARE_USAGE: &str = "usage: cargo xtask trial-compare\n\n\
reads each entry in dist/trial/entries/<handle>/ (its .dlg files) and writes\n\
dist/trial/compare.html: the current draft and the entries side by side,\n\
scene by scene, named Entry A, B, C... in a random order, each with its\n\
problem count. Which entry is whose is in dist/trial/compare-key.txt: open\n\
it only after you've picked.";

/// The lines of `script` from `@scene <id>` to its `@end`, as written.
pub fn scene<'a>(script: &'a str, id: &str) -> Option<&'a str> {
    let mut start = None;
    let mut at = 0;
    for line in script.split_inclusive('\n') {
        let words: Vec<&str> = line.split_whitespace().collect();
        match (start, words.as_slice()) {
            (None, ["@scene", name]) if *name == id => start = Some(at),
            (Some(from), ["@end"]) => return Some(&script[from..at + line.len()]),
            _ => {}
        }
        at += line.len();
    }
    None
}

/// The first line of every packet's script: whose packet it is.
pub fn mark(handle: &str) -> String {
    format!(
        "# Trial packet for {handle}. Confidential: please don't share it or post it anywhere.\n"
    )
}

/// The trial's script: the mark, then each of [`SCENES`] cut from
/// `script`, a blank line between them.
pub fn trial_script(script: &str, handle: &str) -> Result<String, String> {
    let mut out = mark(handle);
    for id in SCENES {
        let text = scene(script, id).ok_or_else(|| format!("{SCRIPT} has no scene {id}"))?;
        out.push('\n');
        out.push_str(text);
    }
    Ok(out)
}

/// Whether `handle` is a mark a file name can hold.
fn valid_handle(handle: &str) -> bool {
    !handle.is_empty()
        && handle.len() <= 40
        && handle
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// One line of [`PACKET_LIST`]: a packet path, the file it comes from, and
/// the `## ` sections of it to take (all of it when there are none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub to: String,
    pub from: String,
    pub sections: Vec<String>,
}

/// `text`'s first word and what follows it.
fn word(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    (end > 0).then(|| (&text[..end], &text[end..]))
}

/// The lines of [`PACKET_LIST`]'s text.
pub fn read_packet_list(text: &str) -> Result<Vec<Page>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((to, (from, rest))) = word(line).and_then(|(to, after)| Some((to, word(after)?)))
        else {
            return Err(format!(
                "{PACKET_LIST}:{}: expected <packet path> <from> [section | ...]",
                i + 1
            ));
        };
        let sections = rest
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        out.push(Page {
            to: to.to_string(),
            from: from.trim().to_string(),
            sections,
        });
    }
    Ok(out)
}

/// `markdown`'s title line (its first `# ` line) and each `## ` section
/// whose heading starts with one of `sections`, with everything under it
/// to the next `## `. Every section asked for must be there.
pub fn sections(markdown: &str, sections: &[String], file: &str) -> Result<String, String> {
    if sections.is_empty() {
        return Ok(markdown.to_string());
    }
    let mut out = String::new();
    if let Some(title) = markdown.lines().find(|l| l.starts_with("# ")) {
        out.push_str(title);
        out.push_str("\n\n");
    }
    for wanted in sections {
        let mut taking = false;
        let mut found = false;
        for line in markdown.lines() {
            if let Some(heading) = line.strip_prefix("## ") {
                taking = heading.starts_with(wanted.as_str());
                found |= taking;
            }
            if taking {
                out.push_str(line);
                out.push('\n');
            }
        }
        if !found {
            return Err(format!("{file} has no section starting \"## {wanted}\""));
        }
    }
    Ok(out)
}

/// The game's display names by id.
pub fn read_names(text: &str) -> Result<BTreeMap<String, String>, String> {
    ron::from_str(text).map_err(|e| format!("can't read the names table: {e}"))
}

/// The ids `script` names: in `{n:<id>}` and `{N:<id>}`, and on the stage
/// (`@left <id>`, `@right <id>`).
fn named_ids(script: &str) -> Vec<String> {
    let mut ids = std::collections::BTreeSet::new();
    for line in script.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if let ["@left" | "@right", id, ..] = words.as_slice()
            && *id != "clear"
        {
            ids.insert((*id).to_string());
        }
        let mut rest = line;
        while let Some(at) = rest.find("{n:").or_else(|| rest.find("{N:")) {
            rest = &rest[at + 3..];
            if let Some(end) = rest.find('}') {
                ids.insert(rest[..end].to_string());
            }
        }
    }
    ids.into_iter().collect()
}

/// `story/names.md`: what each id the trial's scenes use shows as.
pub fn names_page(script: &str, names: &BTreeMap<String, String>) -> String {
    let mut out = String::from(
        "# The names in the trial's scenes\n\n\
         A script never writes a name: it writes the name's id, so a name can \
         change later without touching a line. On the stage and before a \
         line, a character is their id (`@left retainer`, `retainer: ...`); \
         inside a line, `{n:<id>}` shows the name and `{N:<id>}` the same \
         name with a capital letter. `{lead}` is the name the player chose.\n\n\
         | Id | Shows as |\n| --- | --- |\n",
    );
    if let Some(lead) = names.get("lead") {
        let _ = writeln!(
            out,
            "| `{{lead}}` | the player's choice (by default {lead}) |"
        );
    }
    for id in named_ids(script) {
        let shown = names.get(&id).map_or("(no name yet)", String::as_str);
        let _ = writeln!(out, "| `{id}` | {shown} |");
    }
    out
}

/// `template` with each `{name}` filled from `values`; a name with no
/// value is an error, so no packet goes out with a hole in it.
pub fn fill(template: &str, values: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = template;
    let mut missing = Vec::new();
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let name = &rest[open + 1..open + close];
        match values.get(name) {
            Some(value) => out.push_str(value),
            None => missing.push(name.to_string()),
        }
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    if missing.is_empty() {
        Ok(out)
    } else {
        Err(format!(
            "no value for {}: give --due, and put the links in {OUT}/links.txt (trial-packet --help)",
            missing.join(", ")
        ))
    }
}

/// `name = value` lines; `#` starts a comment line.
pub fn read_links(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(k, v)| !k.is_empty() && !v.is_empty())
        .collect()
}

/// The problems the game's checks find in a trial script (`story-check`'s,
/// with only some of the game's scenes there: ticket 0764), one a line.
pub fn problems(script: &str) -> Vec<String> {
    let loaded = trpg_content::load_with_some_scripts(&[("ch01.dlg", script)]);
    match loaded.content {
        Ok(_) => Vec::new(),
        Err(errors) => errors.0.iter().map(ToString::to_string).collect(),
    }
}

/// The files of one packet, by their path in it.
pub type Files = BTreeMap<String, Vec<u8>>;

/// Writes `zip_path`: `files`, under the folder `root`.
pub fn write_packet(files: &Files, root: &str, zip_path: &Path) -> Result<(), String> {
    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let out = fs::File::create(zip_path).map_err(|e| format!("{}: {e}", zip_path.display()))?;
    let mut zip = zip::ZipWriter::new(out);
    let options = SimpleFileOptions::default();
    let err = |e: &dyn std::fmt::Display| format!("{}: {e}", zip_path.display());
    for (name, bytes) in files {
        zip.start_file(format!("{root}/{name}"), options)
            .map_err(|e| err(&e))?;
        zip.write_all(bytes).map_err(|e| err(&e))?;
    }
    zip.finish().map_err(|e| err(&e))?;
    Ok(())
}

/// What `trial-packet` reads, gathered so a test can give its own.
pub struct PacketInputs {
    pub handle: String,
    /// Chapter 1's script.
    pub script: String,
    pub names: BTreeMap<String, String>,
    /// The pages [`PACKET_LIST`] names, already cut, by packet path.
    pub pages: Files,
    pub start_here: String,
    /// The links and the applicant's due date.
    pub values: BTreeMap<String, String>,
}

/// A packet's own files, checked: the trial script must have no problem.
pub fn packet_files(
    inputs: &PacketInputs,
    check: &dyn Fn(&str) -> Vec<String>,
) -> Result<Files, String> {
    let script = trial_script(&inputs.script, &inputs.handle)?;
    let found = check(&script);
    if !found.is_empty() {
        return Err(format!(
            "the trial script has {} problem(s), so no packet was made:\n{}",
            found.len(),
            found.join("\n")
        ));
    }
    let mut values = inputs.values.clone();
    values.insert("handle".into(), inputs.handle.clone());
    let mut files = inputs.pages.clone();
    files.insert(
        "START-HERE.txt".into(),
        fill(&inputs.start_here, &values)?.into_bytes(),
    );
    files.insert(
        "story/names.md".into(),
        names_page(&script, &inputs.names).into_bytes(),
    );
    files.insert("dialogue/ch01.dlg".into(), script.into_bytes());
    Ok(files)
}

fn read(root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(root.join(path)).map_err(|e| format!("can't read {path}: {e}"))
}

/// Where each file of the game's checkout is in a packet: every page's
/// source, the names table's page (the packet writes its own,
/// `story/names.md`) and Chapter 1's script.
pub fn packet_paths(list: &[Page]) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = list
        .iter()
        .map(|p| (p.from.clone(), p.to.clone()))
        .collect();
    out.insert(format!("{STORY}names.md"), "story/names.md".into());
    for script in [SCRIPT, "assets/dialogue/ch01.dlg"] {
        out.insert(script.into(), "dialogue/ch01.dlg".into());
    }
    out
}

/// Whether `page`'s file paths are rewritten for the packet: a Markdown
/// page cut from the story's documents. The trial's own pages and the VS
/// Code folder are written for the packet already.
fn rewrites_paths(page: &Page) -> bool {
    Path::new(&page.to).extension().is_some_and(|e| e == "md")
        && page.from.starts_with("assets-private/")
}

/// `path` with its `.` and `..` parts resolved.
fn normalized(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}

/// Where `mention` (a path written in `from`, relative to its folder or to
/// the checkout's root) is in the packet, if it is.
fn in_packet(mention: &str, from: &str, moved: &BTreeMap<String, String>) -> Option<String> {
    let folder = from.rsplit_once('/').map_or("", |(dir, _)| dir);
    [format!("{folder}/{mention}"), mention.to_string()]
        .iter()
        .find_map(|candidate| moved.get(&normalized(candidate)).cloned())
}

/// The link from the page at `page` to the packet file `to`, both from the
/// packet's root.
fn relative(page: &str, to: &str) -> String {
    let folders: Vec<&str> = page.split('/').collect();
    let folders = &folders[..folders.len() - 1];
    let target: Vec<&str> = to.split('/').collect();
    // Only the folders the two paths don't share are climbed out of.
    let shared = folders
        .iter()
        .zip(&target[..target.len() - 1])
        .take_while(|(a, b)| a == b)
        .count();
    let mut out = "../".repeat(folders.len() - shared);
    out.push_str(&target[shared..].join("/"));
    out
}

/// Whether `text` names a file: a word, a dot and `md`, `dlg` or `ron`.
fn names_a_file(text: &str) -> bool {
    let text = text.trim_matches('`');
    let Some((stem, ext)) = text.split('#').next().unwrap_or("").rsplit_once('.') else {
        return false;
    };
    ["md", "dlg", "ron"].contains(&ext)
        && stem.ends_with(|c: char| c.is_alphanumeric() || c == '_' || c == '-')
        && !text.contains(' ')
}

/// What a packet page says where the story documents said a path: links
/// and `code` mentions of a file the packet has show its packet path (from
/// the packet's root, so the folder is in the text too), and a link works
/// from the page; a file the packet doesn't have is marked so, its link
/// taken out. `from` is the page's source and `page` its packet path.
pub fn packet_refs(text: &str, from: &str, page: &str, moved: &BTreeMap<String, String>) -> String {
    // A fenced block (an example script) stays as written.
    let mut out = String::new();
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            out.push_str(line);
        } else if fenced {
            out.push_str(line);
        } else {
            out.push_str(&line_refs(line, from, page, moved));
        }
    }
    out
}

/// [`packet_refs`] on one line outside a fenced block.
fn line_refs(text: &str, from: &str, page: &str, moved: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(['[', '`']) {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        if rest.starts_with('[')
            && let Some(close) = rest.find("](")
            && let Some(end) = rest[close..].find(')')
            && !rest[1..close].contains(']')
        {
            let label = &rest[1..close];
            let target = &rest[close + 2..close + end];
            rest = &rest[close + end + 1..];
            if target.contains("://") || target.starts_with('#') {
                let _ = write!(out, "[{label}]({target})");
                continue;
            }
            let (file, anchor) = target.split_once('#').map_or((target, ""), |(f, a)| (f, a));
            let anchor = if anchor.is_empty() {
                String::new()
            } else {
                format!("#{anchor}")
            };
            match in_packet(file, from, moved) {
                Some(to) if names_a_file(label) => {
                    let _ = write!(out, "[`{to}`]({}{anchor})", relative(page, &to));
                }
                Some(to) => {
                    let _ = write!(out, "[{label}]({}{anchor}) (`{to}`)", relative(page, &to));
                }
                None => {
                    let _ = write!(out, "{label} (not in this packet)");
                }
            }
            continue;
        }
        if rest.starts_with('`')
            && let Some(end) = rest[1..].find('`')
        {
            let code = &rest[1..=end];
            rest = &rest[end + 2..];
            let file = code.split('#').next().unwrap_or(code);
            if !names_a_file(code) {
                let _ = write!(out, "`{code}`");
            } else if let Some(to) = in_packet(file, from, moved) {
                let _ = write!(out, "`{to}`");
            } else {
                let _ = write!(out, "`{code}` (not in this packet)");
            }
            continue;
        }
        out.push_str(&rest[..1]);
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

/// The pages of [`PACKET_LIST`], cut, by packet path; lines with the same
/// path are joined.
fn pages(root: &Path) -> Result<Files, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let list = read_packet_list(&read(root, PACKET_LIST)?)?;
    let moved = packet_paths(&list);
    for page in list {
        let mut text = sections(&read(root, &page.from)?, &page.sections, &page.from)?;
        if rewrites_paths(&page) {
            text = packet_refs(&text, &page.from, &page.to, &moved);
        }
        let joined = out.entry(page.to).or_default();
        if !joined.is_empty() {
            joined.push_str("\n---\n\n");
        }
        joined.push_str(&text);
    }
    Ok(out.into_iter().map(|(k, v)| (k, v.into_bytes())).collect())
}

/// The files in `dir`, by `extra/<name>` (the signed agreement, say).
fn extras(dir: &Path) -> Result<Files, String> {
    let mut out = Files::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(out);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            out.insert(name, bytes);
        }
    }
    Ok(out)
}

fn names(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let path = NAMES
        .iter()
        .find(|p| root.join(p).is_file())
        .ok_or("no names table")?;
    read_names(&read(root, path)?)
}

/// `trial-packet`'s arguments: `--name value` pairs.
fn flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let name = flag
            .strip_prefix("--")
            .ok_or_else(|| format!("unexpected {flag}"))?;
        let value = it.next().ok_or_else(|| format!("{flag} needs a value"))?;
        out.insert(name.to_string(), value.clone());
    }
    Ok(out)
}

/// `cargo xtask trial-packet`.
pub fn packet_command(root: &Path, args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{PACKET_USAGE}");
        return 0;
    }
    match build_packet(root, args) {
        Ok(path) => {
            println!("wrote {}", path.display());
            0
        }
        Err(e) => {
            eprintln!("trial-packet: {e}");
            1
        }
    }
}

fn build_packet(root: &Path, args: &[String]) -> Result<PathBuf, String> {
    // Nick, 2026-11-15: no packet before writers have the editor's colors
    // and indenting (ticket 0768): the format breaks easily by hand.
    if !root.join(EDITOR).is_file() {
        return Err(format!(
            "no {EDITOR}: ticket 0768 (the .dlg colors and indenting for VS Code) comes before the first packet"
        ));
    }
    let flags = flags(args)?;
    let handle = flags.get("for").ok_or(PACKET_USAGE)?.clone();
    if !valid_handle(&handle) {
        return Err(format!(
            "{handle:?}: a handle is letters, digits, - and _ only"
        ));
    }
    if let Some(unknown) = flags.keys().find(|k| !["for", "due"].contains(&k.as_str())) {
        return Err(format!("unknown option --{unknown}"));
    }
    let mut values = fs::read_to_string(root.join(OUT).join("links.txt"))
        .map(|t| read_links(&t))
        .unwrap_or_default();
    if let Some(due) = flags.get("due") {
        values.insert("due".into(), due.clone());
    }
    let inputs = PacketInputs {
        handle: handle.clone(),
        script: read(root, SCRIPT)
            .map_err(|e| format!("{e} (run `cargo shuyi private-assets`)"))?,
        names: names(root)?,
        pages: pages(root)?,
        start_here: read(root, START_HERE)?,
        values,
    };
    let mut files = packet_files(&inputs, &problems)?;
    for (name, bytes) in extras(&root.join(OUT).join("extra"))? {
        files.insert(name, bytes);
    }
    let zip = root.join(OUT).join(format!("{handle}.zip"));
    write_packet(&files, &format!("visions-of-shuyi-trial-{handle}"), &zip)?;
    Ok(zip)
}

// ---------------------------------------------------------------------------
// The comparison
// ---------------------------------------------------------------------------

/// Letters for `handles` in an order `seed` shuffles: (letter, handle).
pub fn letters(handles: &[String], seed: u64) -> Vec<(String, String)> {
    let mut order: Vec<String> = handles.to_vec();
    order.sort();
    // A small xorshift: the order only needs to be one Nick can't guess.
    let mut state = seed | 1;
    for i in (1..order.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = usize::try_from(state % (i as u64 + 1)).unwrap_or(0);
        order.swap(i, j);
    }
    order
        .into_iter()
        .enumerate()
        .map(|(i, handle)| (letter(i), handle))
        .collect()
}

/// `A`, `B`, … `Z`, `AA`, `AB`, …
fn letter(i: usize) -> String {
    let a = b'A';
    let n = u8::try_from(i % 26).unwrap_or(0);
    if i < 26 {
        char::from(a + n).to_string()
    } else {
        format!("{}{}", letter(i / 26 - 1), char::from(a + n))
    }
}

/// A scene as the page shows it: no comment lines (they can carry a name),
/// the names written out.
pub fn readable(scene: &str, names: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for line in scene.lines().filter(|l| !l.trim_start().starts_with('#')) {
        out.push_str(&with_names(line, names));
        out.push('\n');
    }
    out
}

/// `line` with `{n:<id>}`, `{N:<id>}` and `{lead}` written out.
fn with_names(line: &str, names: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let token = &rest[open + 1..open + close];
        let name = match token.split_once(':') {
            Some(("n", id)) => names.get(id).cloned(),
            Some(("N", id)) => names.get(id).map(|n| capitalized(n)),
            _ if token == "lead" => names.get("lead").cloned(),
            _ => None,
        };
        out.push_str(&name.unwrap_or_else(|| format!("{{{token}}}")));
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    out
}

fn capitalized(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// A very small Markdown: `#` headings, `- ` items, `> ` quotes and
/// paragraphs, for the reading guide.
fn markdown_html(text: &str) -> String {
    let mut out = String::new();
    for block in text.split("\n\n").map(str::trim).filter(|b| !b.is_empty()) {
        if let Some(h) = block.strip_prefix("# ") {
            let _ = writeln!(out, "<h1>{}</h1>", escape(h));
        } else if let Some(h) = block.strip_prefix("## ") {
            let _ = writeln!(out, "<h2>{}</h2>", escape(h));
        } else if block.starts_with("- ") {
            out.push_str("<ul>");
            for item in block.split("\n- ") {
                let item = item.trim_start_matches("- ").replace('\n', " ");
                let _ = write!(out, "<li>{}</li>", escape(&item));
            }
            out.push_str("</ul>\n");
        } else if block.starts_with("> ") {
            let quote: Vec<&str> = block
                .lines()
                .map(|l| l.trim_start_matches('>').trim())
                .collect();
            let _ = writeln!(out, "<blockquote>{}</blockquote>", escape(&quote.join(" ")));
        } else {
            let _ = writeln!(out, "<p>{}</p>", escape(&block.replace('\n', " ")));
        }
    }
    out
}

/// One column of the page: its title, its whole script and its problems.
pub struct Column {
    pub title: String,
    pub script: String,
    pub problems: usize,
}

/// The comparison page: the guide, then each of [`SCENES`] with a column
/// per entry.
pub fn compare_page(guide: &str, columns: &[Column], names: &BTreeMap<String, String>) -> String {
    let mut out = String::from(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <title>Writer trial: the entries</title><style>\
         body{font-family:Georgia,serif;margin:2em;max-width:none}\
         .guide{max-width:46em}\
         .row{display:flex;gap:1em;overflow-x:auto;align-items:flex-start}\
         .col{flex:0 0 32em;border:1px solid #ccc;padding:0 1em}\
         .col h3{margin:.6em 0 .2em}.count{color:#555;margin:0 0 .6em}\
         pre{white-space:pre-wrap;font-family:Georgia,serif;font-size:15px;line-height:1.45}\
         blockquote{border-left:3px solid #aaa;margin-left:0;padding-left:1em;color:#333}\
         </style></head><body>\n<div class=\"guide\">\n",
    );
    out.push_str(&markdown_html(guide));
    out.push_str(
        "<p><em>Comment lines are hidden, and names are written out as the game shows them \
         (the lead is the default name).</em></p>\n</div>\n",
    );
    for id in SCENES {
        let _ = writeln!(out, "<h2>{id}</h2>\n<div class=\"row\">");
        for column in columns {
            let text = scene(&column.script, id).map_or_else(
                || "(this entry has no such scene)".to_string(),
                |s| readable(s, names),
            );
            let problems = match column.problems {
                0 => "no problem".to_string(),
                1 => "1 problem".to_string(),
                n => format!("{n} problems"),
            };
            let _ = writeln!(
                out,
                "<div class=\"col\"><h3>{}</h3><p class=\"count\">{problems} in the whole entry</p><pre>{}</pre></div>",
                escape(&column.title),
                escape(&text)
            );
        }
        out.push_str("</div>\n");
    }
    out.push_str("</body></html>\n");
    out
}

/// An entry's `.dlg` files, joined, without the packet's mark line.
fn entry_script(dir: &Path) -> Result<String, String> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "dlg"))
        .collect();
    files.sort();
    let mut out = String::new();
    for file in files {
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        for line in text
            .lines()
            .filter(|l| !l.starts_with("# Trial packet for "))
        {
            out.push_str(line);
            out.push('\n');
        }
    }
    Ok(out)
}

/// The entries in `dir`, by handle (the folder's name).
fn entries(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let read = fs::read_dir(dir).map_err(|e| {
        format!(
            "{}: {e} (put each entry in {OUT}/entries/<handle>/)",
            dir.display()
        )
    })?;
    let mut out = Vec::new();
    for entry in read.flatten() {
        if entry.path().is_dir() {
            let handle = entry.file_name().to_string_lossy().to_string();
            out.push((handle, entry_script(&entry.path())?));
        }
    }
    out.sort();
    Ok(out)
}

/// The page and the key for `entries` (handle, script), the draft first.
pub fn comparison(
    guide: &str,
    draft: &str,
    entries: &[(String, String)],
    names: &BTreeMap<String, String>,
    seed: u64,
    check: &dyn Fn(&str) -> Vec<String>,
) -> (String, String) {
    let handles: Vec<String> = entries.iter().map(|(h, _)| h.clone()).collect();
    let lettered = letters(&handles, seed);
    let mut columns = vec![Column {
        title: "The current draft (AI)".into(),
        script: draft.to_string(),
        problems: check(draft).len(),
    }];
    let mut key = String::from("# Which entry is whose. Open it only after you've picked.\n");
    for (letter, handle) in &lettered {
        let script = entries
            .iter()
            .find(|(h, _)| h == handle)
            .map(|(_, s)| s.clone())
            .unwrap_or_default();
        columns.push(Column {
            title: format!("Entry {letter}"),
            problems: check(&script).len(),
            script,
        });
        let _ = writeln!(key, "Entry {letter}: {handle}");
    }
    (compare_page(guide, &columns, names), key)
}

/// `cargo xtask trial-compare`.
pub fn compare_command(root: &Path, args: &[String]) -> u8 {
    if !args.is_empty() {
        println!("{COMPARE_USAGE}");
        return u8::from(!args.iter().any(|a| a == "--help" || a == "-h")) * 2;
    }
    match build_comparison(root) {
        Ok(page) => {
            println!(
                "wrote {} (the key is beside it: open it after you pick)",
                page.display()
            );
            0
        }
        Err(e) => {
            eprintln!("trial-compare: {e}");
            1
        }
    }
}

fn build_comparison(root: &Path) -> Result<PathBuf, String> {
    let out = root.join(OUT);
    let entries = entries(&out.join("entries"))?;
    if entries.is_empty() {
        return Err(format!("no entries in {OUT}/entries/"));
    }
    let script =
        read(root, SCRIPT).map_err(|e| format!("{e} (run `cargo shuyi private-assets`)"))?;
    let draft = trial_script(&script, "the draft")?;
    let seed = RandomState::new().hash_one(std::time::SystemTime::now());
    let (page, key) = comparison(
        &read(root, READING_GUIDE)?,
        &draft,
        &entries,
        &names(root)?,
        seed,
        &problems,
    );
    for (handle, _) in &entries {
        if page.to_lowercase().contains(&handle.to_lowercase()) {
            eprintln!(
                "trial-compare: the page has the text {handle:?} in it (an entry's name, or a word that happens to match it)"
            );
        }
    }
    let page_path = out.join("compare.html");
    fs::write(&page_path, page).map_err(|e| format!("{}: {e}", page_path.display()))?;
    fs::write(out.join("compare-key.txt"), key)
        .map_err(|e| format!("{OUT}/compare-key.txt: {e}"))?;
    Ok(page_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT_TEXT: &str = "# The file's header.\n\n\
        # Above the first scene.\n\
        @scene ch01_intro\n> Morning.\n# a note inside\nretainer: Kid. {n:place.harrowby}.\n@end\n\n\
        @scene ch01_prebattle\n@left retainer neutral\nretainer: Swords.\n@end\n\
        @scene ch01_first_turn\n> Go.\n@end\n\
        @scene ch01_boss_engage\n> Not in the trial.\n@end\n\
        @scene ch01_victory\n> Won.\n@end\n\
        @scene ch01_tbc\n@left rival neutral\n> Night.\n@end\n";

    fn names() -> BTreeMap<String, String> {
        [
            ("lead", "Ellery"),
            ("retainer", "Hollis Marr"),
            ("rival", "Dace Marr"),
            ("place.harrowby", "Harrowby"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    /// A folder of its own under the system's temporary folder, deleted on
    /// drop.
    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("trpg-trial-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_scene_is_cut_at_its_scene_and_end_lines_as_written() {
        assert_eq!(
            scene(SCRIPT_TEXT, "ch01_intro"),
            Some(
                "@scene ch01_intro\n> Morning.\n# a note inside\nretainer: Kid. {n:place.harrowby}.\n@end\n"
            )
        );
        assert_eq!(scene(SCRIPT_TEXT, "ch01_tbc").unwrap().lines().count(), 4);
        assert_eq!(scene(SCRIPT_TEXT, "ch01"), None, "an id is matched whole");
        assert_eq!(scene(SCRIPT_TEXT, "nowhere"), None);
    }

    #[test]
    fn the_trial_script_is_the_mark_and_exactly_the_chosen_scenes() {
        let script = trial_script(SCRIPT_TEXT, "test").unwrap();
        assert_eq!(script.lines().next().unwrap(), mark("test").trim_end());
        let scenes: Vec<&str> = script
            .lines()
            .filter_map(|l| l.strip_prefix("@scene "))
            .collect();
        assert_eq!(scenes, SCENES);
        assert!(!script.contains("Not in the trial"));
        assert!(!script.contains("The file's header"));
        assert!(trial_script("@scene ch01_intro\n@end\n", "x").is_err());
    }

    #[test]
    fn a_handle_is_a_plain_file_name() {
        assert!(valid_handle("u_Writer-42"));
        for bad in ["", "a b", "../x", "x.zip", "é"] {
            assert!(!valid_handle(bad), "{bad:?}");
        }
    }

    #[test]
    fn the_packet_list_reads_paths_sources_and_sections() {
        let list = "# comment\n\nbrief.md  docs/writer-trial/brief.md\n\
                    story/world.md  bible.md  Premise | Rules of magic\n";
        assert_eq!(
            read_packet_list(list).unwrap(),
            vec![
                Page {
                    to: "brief.md".into(),
                    from: "docs/writer-trial/brief.md".into(),
                    sections: vec![]
                },
                Page {
                    to: "story/world.md".into(),
                    from: "bible.md".into(),
                    sections: vec!["Premise".into(), "Rules of magic".into()],
                },
            ]
        );
        assert!(read_packet_list("lonely\n").is_err());
    }

    #[test]
    fn sections_keep_the_title_and_the_sections_asked_for() {
        let doc =
            "# Bible\n\nintro\n\n## Premise\np\n### Sub\ns\n## History\nsecret\n## Glossary\ng\n";
        let got = sections(doc, &["Premise".into(), "Glossary".into()], "b.md").unwrap();
        assert_eq!(
            got,
            "# Bible\n\n## Premise\np\n### Sub\ns\n## Glossary\ng\n"
        );
        assert!(!got.contains("secret"));
        assert!(sections(doc, &["Nope".into()], "b.md").is_err());
        assert_eq!(sections(doc, &[], "b.md").unwrap(), doc);
    }

    #[test]
    fn the_names_page_lists_what_the_scenes_name() {
        let page = names_page(&trial_script(SCRIPT_TEXT, "t").unwrap(), &names());
        assert!(page.contains("| `retainer` | Hollis Marr |"));
        assert!(page.contains("| `rival` | Dace Marr |"));
        assert!(page.contains("| `place.harrowby` | Harrowby |"));
        assert!(page.contains("by default Ellery"));
    }

    #[test]
    fn a_template_with_a_missing_value_is_refused() {
        let mut values = BTreeMap::new();
        values.insert("handle".to_string(), "test".to_string());
        assert_eq!(fill("For {handle}.", &values).unwrap(), "For test.");
        let err = fill("{handle} {preview} {due}", &values).unwrap_err();
        assert!(err.contains("preview, due"), "{err}");
        assert_eq!(
            read_links("# x\ndemo = https://a\nempty =\n"),
            BTreeMap::from([("demo".to_string(), "https://a".to_string())])
        );
    }

    fn inputs() -> PacketInputs {
        PacketInputs {
            handle: "test".into(),
            script: SCRIPT_TEXT.into(),
            names: names(),
            pages: Files::from([("brief.md".to_string(), b"# Brief\n".to_vec())]),
            start_here: "Packet for {handle}. Preview: {preview}\n".into(),
            values: BTreeMap::from([("preview".to_string(), "https://preview".to_string())]),
        }
    }

    #[test]
    fn a_script_with_problems_makes_no_packet() {
        let err = packet_files(&inputs(), &|_| vec!["ch01.dlg:3: bad".into()]).unwrap_err();
        assert!(err.contains("ch01.dlg:3: bad"));
    }

    fn zip_text(path: &Path, name: &str) -> String {
        use std::io::Read as _;
        let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        let mut out = String::new();
        zip.by_name(name).unwrap().read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn a_packet_holds_the_marked_scenes_and_no_program() {
        let dir = Folder::new("packet");
        let files = packet_files(&inputs(), &|_| Vec::new()).unwrap();
        let out = dir.0.join("test.zip");
        write_packet(&files, "trial-test", &out).unwrap();
        let mut names: Vec<String> = zip::ZipArchive::new(fs::File::open(&out).unwrap())
            .unwrap()
            .file_names()
            .map(|n| n.unwrap().to_string())
            .collect();
        names.sort();
        assert_eq!(
            names,
            [
                "trial-test/START-HERE.txt",
                "trial-test/brief.md",
                "trial-test/dialogue/ch01.dlg",
                "trial-test/story/names.md",
            ]
        );
        let script = zip_text(&out, "trial-test/dialogue/ch01.dlg");
        assert!(script.starts_with(&mark("test")));
        assert_eq!(script, trial_script(SCRIPT_TEXT, "test").unwrap());
        assert_eq!(
            zip_text(&out, "trial-test/START-HERE.txt"),
            "Packet for test. Preview: https://preview\n"
        );
    }

    /// The real script, when the private checkout is here (CI's jobs that
    /// load content check it out): the trial's scenes pass the game's
    /// checks, so every packet's would.
    #[test]
    fn the_real_trial_script_has_no_problem() {
        let root = crate::repo_root();
        let Ok(script) = fs::read_to_string(root.join(SCRIPT)) else {
            eprintln!("no {SCRIPT}: skipped (run `cargo shuyi private-assets`)");
            return;
        };
        let trial = trial_script(&script, "test").unwrap();
        assert_eq!(problems(&trial), Vec::<String>::new());
        // And the pages the list names are all there to cut.
        pages(&root).unwrap();
    }

    fn moved() -> BTreeMap<String, String> {
        packet_paths(&[
            Page {
                to: "cheat-sheet.md".into(),
                from: format!("{STORY}cheat-sheet.md"),
                sections: vec![],
            },
            Page {
                to: "story/2-the-arc.md".into(),
                from: format!("{STORY}outline.md"),
                sections: vec![],
            },
            Page {
                to: "story/3-chapter-1.md".into(),
                from: format!("{STORY}chapters/ch01.md"),
                sections: vec![],
            },
        ])
    }

    #[test]
    fn links_point_into_the_packet_and_show_the_folder() {
        let m = moved();
        let cheat = &format!("{STORY}cheat-sheet.md");
        assert_eq!(
            packet_refs("See [`names.md`](names.md).\n", cheat, "cheat-sheet.md", &m),
            "See [`story/names.md`](story/names.md).\n"
        );
        let ch01 = &format!("{STORY}chapters/ch01.md");
        assert_eq!(
            packet_refs(
                "[the outline](../outline.md#act-1)",
                ch01,
                "story/3-chapter-1.md",
                &m
            ),
            "[the outline](2-the-arc.md#act-1) (`story/2-the-arc.md`)"
        );
        assert_eq!(
            packet_refs(
                "`../names.md` and `assets/dialogue/ch01.dlg`",
                ch01,
                "story/3-chapter-1.md",
                &m
            ),
            "`story/names.md` and `dialogue/ch01.dlg`"
        );
    }

    #[test]
    fn a_file_outside_the_packet_is_marked_and_unlinked() {
        let m = moved();
        let cheat = &format!("{STORY}cheat-sheet.md");
        assert_eq!(
            packet_refs(
                "the [writer's guide](writers-guide.md)",
                cheat,
                "cheat-sheet.md",
                &m
            ),
            "the writer's guide (not in this packet)"
        );
        assert_eq!(
            packet_refs(
                "`turn-structure.md`, `@scene`, `.dlg`",
                cheat,
                "cheat-sheet.md",
                &m
            ),
            "`turn-structure.md` (not in this packet), `@scene`, `.dlg`"
        );
        let web = "[VS Code](https://code.visualstudio.com/)";
        assert_eq!(packet_refs(web, cheat, "cheat-sheet.md", &m), web);
        let fenced = "```dlg\n# see `names.md`\n```\n";
        assert_eq!(packet_refs(fenced, cheat, "cheat-sheet.md", &m), fenced);
    }

    #[test]
    fn letters_shuffle_and_cover_every_entry() {
        let handles: Vec<String> = ["ann", "bo", "cy", "di"].map(String::from).to_vec();
        let a = letters(&handles, 1);
        assert_eq!(
            a.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>(),
            ["A", "B", "C", "D"]
        );
        let mut whose: Vec<&str> = a.iter().map(|(_, h)| h.as_str()).collect();
        whose.sort_unstable();
        assert_eq!(whose, ["ann", "bo", "cy", "di"]);
        let orders: std::collections::BTreeSet<Vec<String>> = (0..20)
            .map(|seed| {
                letters(&handles, seed)
                    .into_iter()
                    .map(|(_, h)| h)
                    .collect()
            })
            .collect();
        assert!(orders.len() > 1, "the seed changes the order");
        assert_eq!(letter(25), "Z");
        assert_eq!(letter(26), "AA");
    }

    #[test]
    fn the_comparison_shows_entries_side_by_side_with_no_handle() {
        let entry = |line: &str| {
            format!(
                "{}# by secretname_one\n@scene ch01_intro\nretainer: {line} {{n:place.harrowby}}.\n@end\n",
                mark("secretname_one")
            )
        };
        let entries = vec![
            ("secretname_one".to_string(), entry("First entry's line")),
            ("secretname_two".to_string(), entry("Second entry's line")),
        ];
        let draft = trial_script(SCRIPT_TEXT, "the draft").unwrap();
        let (page, key) = comparison("# Guide\n\n- Look.", &draft, &entries, &names(), 7, &|s| {
            vec![String::new(); usize::from(s.contains("Second"))]
        });
        assert!(!page.contains("secretname"), "no handle on the page");
        assert!(page.contains("Entry A") && page.contains("Entry B"));
        assert!(page.contains("The current draft (AI)"));
        assert!(page.contains("First entry&#39;s line") || page.contains("First entry's line"));
        assert!(page.contains("Harrowby"), "names are written out");
        assert!(page.contains("1 problem in the whole entry"));
        assert!(page.contains("(this entry has no such scene)"));
        assert!(page.find("ch01_intro") < page.find("ch01_tbc"));
        assert!(key.contains("secretname_one") && key.contains("secretname_two"));
    }

    #[test]
    fn an_entry_folder_is_read_without_its_mark() {
        let dir = Folder::new("entries");
        let one = dir.0.join("someone");
        fs::create_dir_all(&one).unwrap();
        fs::write(
            one.join("ch01.dlg"),
            format!("{}@scene ch01_intro\n> Hi.\n@end\n", mark("someone")),
        )
        .unwrap();
        fs::write(one.join("notes.txt"), "not a script").unwrap();
        let got = entries(&dir.0).unwrap();
        assert_eq!(
            got,
            vec![(
                "someone".to_string(),
                "@scene ch01_intro\n> Hi.\n@end\n".to_string()
            )]
        );
    }

    /// The VS Code folder for `.dlg` scripts (ticket 0768): its JSON
    /// parses, `package.json` names files that are there, and the packet
    /// list copies every file of it whole, under `vscode-dlg/`.
    #[test]
    fn the_vscode_folder_parses_and_the_packet_copies_all_of_it() {
        const DIR: &str = "tools/vscode-dlg";
        let root = crate::repo_root();
        let mut files = Vec::new();
        let mut todo = vec![root.join(DIR)];
        while let Some(dir) = todo.pop() {
            for entry in fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    todo.push(path);
                } else {
                    let rel = path.strip_prefix(root.join(DIR)).unwrap();
                    files.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        files.sort();
        let json = |rel: &str| -> serde_json::Value {
            serde_json::from_str(&fs::read_to_string(root.join(DIR).join(rel)).unwrap())
                .unwrap_or_else(|e| panic!("{DIR}/{rel}: {e}"))
        };
        let is_json = |f: &&String| {
            Path::new(f.as_str())
                .extension()
                .is_some_and(|e| e == "json")
        };
        for rel in files.iter().filter(is_json) {
            json(rel);
        }
        let package = json("package.json");
        let contributes = &package["contributes"];
        let named = [
            &contributes["languages"][0]["configuration"],
            &contributes["grammars"][0]["path"],
        ];
        for path in named {
            let path = path.as_str().expect("a path").trim_start_matches("./");
            assert!(files.iter().any(|f| f == path), "package.json names {path}");
        }
        assert!(package.get("main").is_none(), "no code");

        let list = read_packet_list(&fs::read_to_string(root.join(PACKET_LIST)).unwrap()).unwrap();
        let mut listed: Vec<String> = list
            .iter()
            .filter_map(|page| {
                let rel = page.to.strip_prefix("vscode-dlg/")?;
                assert_eq!(
                    page.from,
                    format!("{DIR}/{rel}"),
                    "{} copied as itself",
                    page.to
                );
                assert!(page.sections.is_empty(), "{} copied whole", page.to);
                Some(rel.to_string())
            })
            .collect();
        listed.sort();
        assert_eq!(listed, files, "packet.txt lists exactly the folder's files");
    }
}
