//! What the Chapter 1 demo embeds of `assets/` (ticket 0914, ADR-0117):
//! the rules of `crates/content/demo/bundle.txt`, the demo edition's
//! content root, and the cut its files go through.
//!
//! `build.rs` includes this file and writes the demo's table with it
//! (`demo_files.rs`): every file of `assets/` but those the list leaves
//! out, each RON file without its comments, and the files the list cuts
//! holding only the entries it keeps. Nothing here reads a file: the
//! build script hands the texts in, so the crate's tests check the same
//! code.
//!
//! The list's lines (blank lines and `#` comments aside):
//!
//! - `chapters/` leaves out a folder and everything below it;
//!   `maps/ch02.map` a file.
//! - `+ maps/ch01.map` keeps a file a folder's line left out.
//! - `keep data/names.ron: lead retainer` keeps only these entries of the
//!   file's top-level map or list; `keep data/characters.ron characters:
//!   lead` of its field `characters`. An entry is kept by its key (a map's
//!   key, a list's string, or a struct's `id`), and so are the keys made
//!   from it with a `.` (`retainer` keeps `retainer.first`). Several lines
//!   for one file and field add up.
//! - `swap chapters/ch01.ron: next: Some("ch02") => next: None` replaces
//!   text that is in the file once.
//!
//! A line that matches nothing stops the build, so the list can't go
//! stale without anyone noticing.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

/// One rule of the list, with its line number for errors.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    line: usize,
    path: String,
}

impl Rule {
    /// Whether the rule names `path`: the file, or a folder above it.
    fn matches(&self, path: &str) -> bool {
        if self.path.ends_with('/') {
            path.starts_with(&self.path)
        } else {
            path == self.path
        }
    }
}

/// The parsed list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    leave_out: Vec<Rule>,
    keep_back: Vec<Rule>,
    /// By file, then field (`None` for the top level): the keys kept.
    keeps: BTreeMap<String, BTreeMap<Option<String>, BTreeSet<String>>>,
    /// By file: the text replaced and its replacement.
    swaps: BTreeMap<String, Vec<(String, String)>>,
}

impl Plan {
    /// Reads the list `text`.
    ///
    /// # Errors
    ///
    /// A line it can't read, with its number.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut plan = Self::default();
        for (n, line) in text.lines().enumerate() {
            let (line_no, line) = (n + 1, line.trim());
            let err = |what: &str| format!("line {line_no}: {what}: `{line}`");
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(path) = line.strip_prefix('+') {
                let path = path.trim().to_owned();
                plan.keep_back.push(Rule {
                    line: line_no,
                    path,
                });
            } else if let Some(rest) = line.strip_prefix("keep ") {
                let (head, ids) = rest.split_once(':').ok_or_else(|| err("no `:`"))?;
                let mut head = head.split_whitespace();
                let path = head.next().ok_or_else(|| err("no file"))?.to_owned();
                let field = head.next().map(str::to_owned);
                if head.next().is_some() {
                    return Err(err("a file and one field at most"));
                }
                let kept = plan.keeps.entry(path).or_default().entry(field);
                kept.or_default()
                    .extend(ids.split_whitespace().map(str::to_owned));
            } else if let Some(rest) = line.strip_prefix("swap ") {
                let (path, swap) = rest.split_once(':').ok_or_else(|| err("no `:`"))?;
                let (old, new) = swap.split_once("=>").ok_or_else(|| err("no `=>`"))?;
                let entry = plan.swaps.entry(path.trim().to_owned()).or_default();
                entry.push((old.trim().to_owned(), new.trim().to_owned()));
            } else if line.contains(char::is_whitespace) {
                return Err(err("not a path, `+`, `keep` or `swap`"));
            } else {
                plan.leave_out.push(Rule {
                    line: line_no,
                    path: line.to_owned(),
                });
            }
        }
        Ok(plan)
    }

    /// The paths of `files` (all of `assets/`, sorted) the demo leaves out,
    /// sorted.
    ///
    /// # Errors
    ///
    /// A rule that names no file of `files`, a `+` line that keeps no file
    /// a rule left out, and a `keep` or `swap` of a file the demo doesn't
    /// hold.
    pub fn left_out(&self, files: &[String]) -> Result<Vec<String>, String> {
        let unmatched =
            |rule: &Rule, what: &str| format!("line {}: `{}` {what}", rule.line, rule.path);
        for rule in &self.leave_out {
            if !files.iter().any(|f| rule.matches(f)) {
                return Err(unmatched(rule, "is no file under assets/"));
            }
        }
        for rule in &self.keep_back {
            let left = files
                .iter()
                .any(|f| rule.matches(f) && self.leave_out.iter().any(|r| r.matches(f)));
            if !left {
                return Err(unmatched(rule, "keeps no file a line above leaves out"));
            }
        }
        let out: Vec<String> = files
            .iter()
            .filter(|f| self.leaves_out(f))
            .cloned()
            .collect();
        let named = self.keeps.keys().chain(self.swaps.keys());
        for path in named {
            if out.contains(path) || !files.contains(path) {
                return Err(format!("`{path}` is cut, but the demo doesn't hold it"));
            }
        }
        Ok(out)
    }

    /// Whether the demo leaves out the file at `path`.
    pub fn leaves_out(&self, path: &str) -> bool {
        self.leave_out.iter().any(|r| r.matches(path))
            && !self.keep_back.iter().any(|r| r.matches(path))
    }

    /// The demo's copy of the file at `path` holding `text`: without its
    /// comments if it is RON, then cut as the list says. `None` when the
    /// copy is the file as it is.
    ///
    /// # Errors
    ///
    /// When the file can't be cut: RON it can't read, a field or a kept key
    /// it lacks, a swap's text not in it exactly once.
    pub fn rewrite(&self, path: &str, text: &str) -> Result<Option<String>, String> {
        let is_ron = path
            .rsplit_once('.')
            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("ron"));
        let cuts = self.keeps.get(path);
        let swaps = self.swaps.get(path);
        if !is_ron && cuts.is_none() && swaps.is_none() {
            return Ok(None);
        }
        let err = |e: String| format!("{path}: {e}");
        let mut out = if is_ron {
            strip_comments(text).map_err(err)?
        } else {
            text.to_owned()
        };
        for (field, keep) in cuts.into_iter().flatten() {
            out = keep_entries(&out, field.as_deref(), keep).map_err(err)?;
        }
        for (old, new) in swaps.into_iter().flatten() {
            let times = out.matches(old.as_str()).count();
            if times != 1 {
                return Err(err(format!("`{old}` is there {times} times, not once")));
            }
            out = out.replacen(old.as_str(), new, 1);
        }
        Ok((out != text).then_some(out))
    }
}

/// What a run of RON text is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Code,
    /// A string or a character, its quotes included.
    Quoted,
    Comment,
}

/// `text` cut into runs of code, quoted text and comments. A run of code
/// may be empty (between two others, or at the end).
fn runs(text: &str) -> Result<Vec<(Kind, Range<usize>)>, String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let (mut i, mut start) = (0, 0);
    while i < bytes.len() {
        let rest = &bytes[i..];
        let run = if rest.starts_with(b"//") {
            let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
            Some((Kind::Comment, i + end))
        } else if rest.starts_with(b"/*") {
            Some((Kind::Comment, block_end(text, i)?))
        } else if matches!(rest[0], b'"' | b'\'') {
            Some((Kind::Quoted, quoted_end(text, i)?))
        } else {
            None
        };
        if let Some((kind, end)) = run {
            out.push((Kind::Code, start..i));
            out.push((kind, i..end));
            (i, start) = (end, end);
        } else {
            i += 1;
        }
    }
    out.push((Kind::Code, start..bytes.len()));
    Ok(out)
}

/// The line of byte `at` of `text`, for errors.
fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

/// The end of the block comment opening at `at` (nested ones count): the
/// byte after its `*/`.
fn block_end(text: &str, at: usize) -> Result<usize, String> {
    let mut depth = 0usize;
    // The pair after a `/*` or a `*/` shares its middle byte: not a mark.
    let mut skip = false;
    for (i, pair) in text.as_bytes()[at..].windows(2).enumerate() {
        if skip {
            skip = false;
            continue;
        }
        match pair {
            b"/*" => depth += 1,
            b"*/" => depth -= 1,
            _ => continue,
        }
        if depth == 0 {
            return Ok(at + i + 2);
        }
        skip = true;
    }
    Err(format!(
        "line {}: a comment never closes",
        line_of(text, at)
    ))
}

/// The end of the string or character opening at `at`, past its closing
/// quote.
fn quoted_end(text: &str, at: usize) -> Result<usize, String> {
    let bytes = text.as_bytes();
    let quote = bytes[at];
    let mut i = at + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b if b == quote => return Ok(i + 1),
            _ => i += 1,
        }
    }
    Err(format!("line {}: a quote never closes", line_of(text, at)))
}

/// `text` (RON) without its comments: each is gone but the line breaks in
/// it, so a line keeps its number.
///
/// # Errors
///
/// A string, a character or a comment that never closes.
pub fn strip_comments(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    for (kind, range) in runs(text)? {
        let run = &text[range];
        if kind == Kind::Comment {
            out.extend(run.chars().filter(|&c| c == '\n'));
        } else {
            out.push_str(run);
        }
    }
    Ok(out)
}

/// Where each byte of a RON text is code, and how deep in brackets: a
/// bracket is at the depth around it, what is inside one deeper.
struct Shape {
    code: Vec<bool>,
    depth: Vec<usize>,
}

impl Shape {
    fn of(text: &str) -> Result<Self, String> {
        let bytes = text.as_bytes();
        let mut code = vec![false; bytes.len()];
        for (kind, range) in runs(text)? {
            if kind == Kind::Code {
                code[range].fill(true);
            }
        }
        let (mut depth, mut d) = (Vec::with_capacity(bytes.len()), 0usize);
        for (i, &b) in bytes.iter().enumerate() {
            if code[i] && matches!(b, b')' | b']' | b'}') {
                d = d.checked_sub(1).ok_or_else(|| {
                    format!("line {}: a bracket closes nothing", line_of(text, i))
                })?;
            }
            depth.push(d);
            if code[i] && matches!(b, b'(' | b'[' | b'{') {
                d += 1;
            }
        }
        Ok(Self { code, depth })
    }

    /// Whether byte `i` of `text` is the code `word` standing alone at
    /// depth `depth`.
    fn word_at(&self, text: &str, i: usize, word: &str, depth: usize) -> bool {
        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let bytes = text.as_bytes();
        let end = i + word.len();
        self.code[i]
            && self.depth[i] == depth
            && text[i..].starts_with(word)
            && (i == 0 || !ident(bytes[i - 1]))
            && bytes.get(end).is_none_or(|&b| !ident(b))
    }

    /// The first byte after `from` that isn't white space.
    fn next_code(text: &str, from: usize) -> Option<usize> {
        text[from..]
            .find(|c: char| !c.is_whitespace())
            .map(|n| from + n)
    }

    /// Where the value of the field `word` at depth `depth` starts, the
    /// field's name in `span`: the byte after `word:` and its space.
    fn field(&self, text: &str, word: &str, depth: usize, span: Range<usize>) -> Option<usize> {
        span.into_iter().find_map(|i| {
            if !self.word_at(text, i, word, depth) {
                return None;
            }
            let colon = Self::next_code(text, i + word.len())?;
            (text.as_bytes()[colon] == b':')
                .then(|| Self::next_code(text, colon + 1))
                .flatten()
        })
    }

    /// The bracket that closes the one opening at `open`.
    fn close_of(&self, text: &str, open: usize) -> Option<usize> {
        let bytes = text.as_bytes();
        // `open` itself opens: never the one that closes.
        (open..bytes.len()).find(|&j| {
            self.code[j]
                && self.depth[j] == self.depth[open]
                && matches!(bytes[j], b')' | b']' | b'}')
        })
    }
}

/// The text of the quoted string starting at `at`, without its quotes.
fn string_at(text: &str, at: usize) -> Option<&str> {
    if text.as_bytes().get(at) != Some(&b'"') {
        return None;
    }
    let end = quoted_end(text, at).ok()?;
    Some(&text[at + 1..end - 1])
}

/// `text` (RON, without comments) holding only the entries of its
/// top-level map or list (`field` `None`), or of the top-level struct's
/// field `field`, whose key is in `keep` or is one of those and a `.`
/// (`retainer.first`). An entry's key is a map's key, a list's string, or
/// a struct's `id`.
///
/// # Errors
///
/// No such map, list or field; an entry with no key; a key of `keep` that
/// is no entry's.
pub fn keep_entries(
    text: &str,
    field: Option<&str>,
    keep: &BTreeSet<String>,
) -> Result<String, String> {
    let shape = Shape::of(text)?;
    let bytes = text.as_bytes();
    let opening = |i: usize| shape.code[i] && matches!(bytes[i], b'(' | b'[' | b'{');
    let open = match field {
        None => (0..bytes.len()).find(|&i| opening(i) && shape.depth[i] == 0),
        Some(field) => shape
            .field(text, field, 1, 0..bytes.len())
            .filter(|&i| opening(i)),
    }
    .ok_or_else(|| format!("no {}", field.map_or("top-level map or list", |f| f)))?;
    let close = shape
        .close_of(text, open)
        .ok_or_else(|| format!("line {}: a bracket never closes", line_of(text, open)))?;
    let inside = shape.depth[open] + 1;
    let mut entries = Vec::new();
    let mut start = open + 1;
    // `open` is a bracket, never a comma: the entries start after it.
    for i in open..=close {
        if i == close || (shape.code[i] && bytes[i] == b',' && shape.depth[i] == inside) {
            let (lead, trimmed) = (
                text[start..i].len() - text[start..i].trim_start().len(),
                text[start..i].trim(),
            );
            if !trimmed.is_empty() {
                entries.push(start + lead..start + lead + trimmed.len());
            }
            start = i + 1;
        }
    }
    let mut found = BTreeSet::new();
    let mut kept = Vec::new();
    for entry in entries {
        let key = string_at(text, entry.start)
            .or_else(|| {
                let id = shape.field(text, "id", inside + 1, entry.clone())?;
                string_at(text, id)
            })
            .ok_or_else(|| format!("line {}: an entry with no key", line_of(text, entry.start)))?;
        let by = keep.iter().find(|k| {
            key == k.as_str()
                || key
                    .strip_prefix(k.as_str())
                    .is_some_and(|r| r.starts_with('.'))
        });
        if let Some(by) = by {
            found.insert(by.clone());
            kept.push(&text[entry]);
        }
    }
    if let Some(missing) = keep.iter().find(|k| !found.contains(*k)) {
        return Err(format!("keeps `{missing}`, which is no entry"));
    }
    let mut out = String::from(&text[..=open]);
    out.push('\n');
    for entry in kept {
        out.push_str(entry);
        out.push_str(",\n");
    }
    out.push_str(&text[close..]);
    Ok(out)
}

#[cfg(test)]
#[path = "demo_bundle/tests.rs"]
mod tests;
