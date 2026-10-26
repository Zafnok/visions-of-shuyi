//! A battle file whose enemy list the autobalance changes (ticket 0509),
//! written back over the file's own text. Only the enemy list changes:
//! the rest of the file, comments too, stays as written. An enemy that
//! changes keeps its own text, only the changed fields rewritten; one
//! added is a copy of its original's text on its own line after it; one
//! removed takes the comment lines just above it with it.
use crate::prelude::*;

use alloc::sync::Arc;
use core::fmt::Write as _;
use core::ops::Range;

use trpg_core::{BattleDef, ItemId, Level, LoadoutDef};

use super::{BattleRefs, RawEnemy, from_source};
use crate::error::ContentError;
use crate::ron_loader::parse_ron;

/// One enemy entry, as the file writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Its generic template, if it is one.
    pub template: Option<String>,
    /// Its `level`, if the file gives one.
    pub level: Option<Level>,
    /// Its tile.
    pub pos: (i32, i32),
    /// Its `loadout`, if the file gives one.
    pub loadout: Option<LoadoutDef>,
    /// A generic enemy the story doesn't name: a template with no `id`,
    /// no `name`, not a boss and not a green unit.
    pub unnamed: bool,
}

/// One enemy of the changed list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foe {
    /// The entry it is, or is a copy of: its index in the file's list.
    pub entry: usize,
    /// Added: a copy of the entry.
    pub copy: bool,
    /// Its generic template.
    pub template: Option<String>,
    /// Its level, `None` for its template's.
    pub level: Option<Level>,
    /// Its tile.
    pub pos: (i32, i32),
    /// Its loadout, `None` for its template's.
    pub loadout: Option<LoadoutDef>,
}

/// A battle file, its enemy list changed by [`foes`](Self::foes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleText {
    file: Arc<File>,
    /// The enemy list, in order: as authored, [`BattleText::parse`] gives
    /// one per entry.
    pub foes: Vec<Foe>,
}

/// What [`BattleText`] keeps of the file as authored.
#[derive(Debug, PartialEq, Eq)]
struct File {
    /// Its human-facing name.
    path: String,
    /// The battle's id (the file stem).
    stem: String,
    source: String,
    /// Each enemy entry's text, `(` to `)`.
    spans: Vec<Range<usize>>,
    entries: Vec<Entry>,
}

impl BattleText {
    /// Battle file `source` (named `path`, for battle `stem`), with its
    /// enemy list as authored.
    pub fn parse(path: &str, stem: &str, source: &str) -> Result<Self, ContentError> {
        let spans = enemy_spans(source);
        let entries = spans
            .iter()
            .map(|span| parse_ron::<RawEnemy>(path, &source[span.clone()]).map(|e| entry(&e)))
            .collect::<Result<Vec<_>, _>>()?;
        let foes = entries
            .iter()
            .enumerate()
            .map(|(i, e)| Foe {
                entry: i,
                copy: false,
                template: e.template.clone(),
                level: e.level,
                pos: e.pos,
                loadout: e.loadout.clone(),
            })
            .collect();
        let file = File {
            path: path.to_owned(),
            stem: stem.to_owned(),
            source: source.to_owned(),
            spans,
            entries,
        };
        Ok(Self {
            file: Arc::new(file),
            foes,
        })
    }

    /// The file's enemy entries as authored.
    pub fn entries(&self) -> &[Entry] {
        &self.file.entries
    }

    /// The battle as the content check loads it.
    pub fn load(&self, refs: &BattleRefs<'_>) -> Result<BattleDef, Vec<ContentError>> {
        from_source(&self.file.path, &self.file.stem, &self.text(), refs)
    }

    /// The file's text with the changed enemy list.
    pub fn text(&self) -> String {
        let File { source, spans, .. } = &*self.file;
        let (Some(first), Some(last)) = (spans.first(), spans.last()) else {
            return source.clone();
        };
        let mut out = source[..first.start].to_owned();
        for (k, foe) in self.foes.iter().enumerate() {
            if k > 0 {
                match foe.entry {
                    e if !foe.copy && e > 0 => out += &source[spans[e - 1].end..spans[e].start],
                    e => {
                        out += ",\n";
                        out += indent(source, spans[e].start);
                    }
                }
            }
            out += &self.foe_text(foe);
        }
        let tail = &source[last.end..];
        out += if self.foes.is_empty() {
            tail.trim_start_matches(',')
        } else {
            tail
        };
        out
    }

    /// `foe`'s entry text: its entry's, the fields it changed rewritten.
    fn foe_text(&self, foe: &Foe) -> String {
        let File { source, spans, .. } = &*self.file;
        let authored = &self.file.entries[foe.entry];
        let mut text = source[spans[foe.entry].clone()].to_owned();
        if foe.template != authored.template
            && let Some(t) = &foe.template
        {
            text = set_field(&text, "template", &format!("\"{t}\""));
        }
        if foe.level != authored.level
            && let Some(level) = foe.level
        {
            text = set_field(&text, "level", &format!("Some({level})"));
        }
        if foe.pos != authored.pos {
            text = set_field(&text, "pos", &format!("({}, {})", foe.pos.0, foe.pos.1));
        }
        if foe.loadout != authored.loadout
            && let Some(loadout) = &foe.loadout
        {
            text = set_field(
                &text,
                "loadout",
                &format!("Some({})", loadout_text(loadout)),
            );
        }
        text
    }
}

fn entry(e: &RawEnemy) -> Entry {
    Entry {
        template: e.template.clone(),
        level: e.level,
        pos: e.pos,
        loadout: e.loadout.as_ref().map(super::RawLoadout::to_def),
        unnamed: e.template.is_some()
            && e.id.is_none()
            && e.name.is_none()
            && !e.boss
            && e.faction == trpg_core::Faction::Enemy,
    }
}

/// `(weapons: ["a"], armour: Some("b"))`, as a battle file writes a loadout.
fn loadout_text(loadout: &LoadoutDef) -> String {
    let quoted = |id: &ItemId| format!("\"{}\"", id.0);
    let weapons: Vec<String> = loadout.weapons.iter().map(quoted).collect();
    let mut text = format!("(weapons: [{}]", weapons.join(", "));
    for (key, item) in [
        ("armour", &loadout.armour),
        ("accessory", &loadout.accessory),
    ] {
        if let Some(item) = item {
            let _ = write!(text, ", {key}: Some({})", quoted(item));
        }
    }
    text + ")"
}

/// The whitespace before byte `at` on its line.
fn indent(source: &str, at: usize) -> &str {
    let line = source[..at].rfind('\n').map_or(0, |n| n + 1);
    let lead = &source[line..at];
    &lead[..lead.len() - lead.trim_start().len()]
}

/// `source`'s bytes with every string's contents and every comment
/// blanked, so brackets and names in them don't count. One pass, byte by
/// byte.
fn code(source: &str) -> Vec<u8> {
    let b = source.as_bytes();
    let mut out = b.to_vec();
    let mut state = In::Code;
    for (i, &c) in b.iter().enumerate() {
        let (blank, next) = match (state, c, b.get(i + 1)) {
            (In::Code, b'"', _) => (false, In::Str),
            (In::Code, b'/', Some(b'/')) => (true, In::Line),
            (In::Code, b'/', Some(b'*')) => (true, In::BlockOpen),
            // A string's quotes and a line comment's end stay.
            (In::Code, ..) | (In::Str, b'"', _) | (In::Line, b'\n', _) => (false, In::Code),
            (In::Str, b'\\', _) => (true, In::Escape),
            (In::Str | In::Escape, ..) => (true, In::Str),
            (In::Line, ..) => (true, In::Line),
            (In::BlockStar, b'/', _) => (true, In::Code),
            (In::Block | In::BlockStar, b'*', _) => (true, In::BlockStar),
            (In::BlockOpen | In::Block | In::BlockStar, ..) => (true, In::Block),
        };
        if blank {
            out[i] = b' ';
        }
        state = next;
    }
    out
}

/// Where [`code`] is in the text.
#[derive(Debug, Clone, Copy)]
enum In {
    Code,
    /// Inside a string.
    Str,
    /// After a `\` in a string.
    Escape,
    /// In a `//` comment.
    Line,
    /// On the `*` that opens a `/*` comment.
    BlockOpen,
    /// In a `/* */` comment.
    Block,
    /// In a `/* */` comment, after a `*`.
    BlockStar,
}

/// Where field `key` is in `code`, at bracket depth `depth`: the byte
/// after its `:`.
fn field_at(code: &[u8], key: &str, depth: i32) -> Option<usize> {
    let mut d = 0;
    for i in 0..code.len() {
        match code[i] {
            b'(' | b'[' | b'{' => d += 1,
            b')' | b']' | b'}' => d -= 1,
            _ if d == depth
                && code[i..].starts_with(key.as_bytes())
                && (i == 0 || !is_name(code[i - 1])) =>
            {
                let rest = &code[i + key.len()..];
                let colon = rest.iter().position(|c| !c.is_ascii_whitespace())?;
                if rest[colon] == b':' {
                    return Some(i + key.len() + colon + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_name(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The span of each entry of the root's `enemies: [...]`: none if it has
/// no such list.
fn enemy_spans(source: &str) -> Vec<Range<usize>> {
    let code = code(source);
    let Some(after) = field_at(&code, "enemies", 1) else {
        return Vec::new();
    };
    let Some(open) = code[after..].iter().position(|&c| c == b'[') else {
        return Vec::new();
    };
    let mut spans = Vec::new();
    let (mut depth, mut start) = (0, 0);
    for (i, &c) in code.iter().enumerate().skip(after + open + 1) {
        match c {
            b'(' | b'[' | b'{' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            b')' | b']' | b'}' if depth == 0 => break,
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    spans.push(start..i + 1);
                }
            }
            _ => {}
        }
    }
    spans
}

/// Entry `text` (`(` to `)`) with field `key` set to `value`: its value
/// replaced, or the field added at the end.
fn set_field(text: &str, key: &str, value: &str) -> String {
    let code = code(text);
    if let Some(colon) = field_at(&code, key, 1) {
        let start = colon + (text[colon..].len() - text[colon..].trim_start().len());
        let mut depth = 0;
        let end = (start..code.len())
            .find(|&i| match code[i] {
                b'(' | b'[' | b'{' => {
                    depth += 1;
                    false
                }
                b')' | b']' | b'}' if depth > 0 => {
                    depth -= 1;
                    false
                }
                b')' | b']' | b'}' | b',' => depth == 0,
                _ => false,
            })
            .unwrap_or(code.len());
        let end = start + text[start..end].trim_end().len();
        return format!("{}{value}{}", &text[..start], &text[end..]);
    }
    let close = text.len() - 1;
    let body = text[..close].trim_end();
    let sep = match body.as_bytes().last() {
        Some(b'(') => "",
        Some(b',') => " ",
        _ => ", ",
    };
    let comma = if sep == " " { "," } else { "" };
    format!("{body}{sep}{key}: {value}{comma}{}", &text[body.len()..])
}

#[cfg(test)]
mod tests;
