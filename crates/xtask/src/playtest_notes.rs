//! What a done ticket leaves for the Act 1 playtest (ticket 1064), under one
//! heading of its `## Completion notes` (ticket 1256): the section reader
//! `ticket-lint` checks with, and `cargo xtask playtest-notes`, which lists
//! every done ticket's section for 1064's checklist.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// The heading, under `## Completion notes`, of what is left to judge by
/// playing.
pub const HEADING: &str = "### For the Act 1 playtest (1064)";

/// A line of the notes that says the ticket leaves nothing to play, and why.
pub const NOTHING: &str = "Nothing for the playtest:";

/// The ticket types that change play, whose done tickets need the heading or
/// the `Nothing for the playtest:` line.
pub const GAMEPLAY_TYPES: &[&str] = &["feature", "content", "tuning", "bug"];

/// The text of a ticket's `## Completion notes`, up to the next `## `
/// heading, or `None` without one.
fn completion_notes(content: &str) -> Option<&str> {
    let start = line_start(content, "## Completion notes")?;
    let body = &content[start..];
    let body = &body[body.find('\n').map_or(body.len(), |i| i + 1)..];
    let end = heading_at_line_start(body, "## ").unwrap_or(body.len());
    Some(&body[..end])
}

/// The byte offset of the first line of `text` that starts with `prefix`.
fn line_start(text: &str, prefix: &str) -> Option<usize> {
    if text.starts_with(prefix) {
        return Some(0);
    }
    text.match_indices('\n')
        .map(|(i, _)| i + 1)
        .find(|&i| text[i..].starts_with(prefix))
}

/// The byte offset of the first line that starts with `prefix` and is not
/// inside a code fence.
fn heading_at_line_start(text: &str, prefix: &str) -> Option<usize> {
    let mut fence = false;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            fence = !fence;
        } else if !fence && line.starts_with(prefix) {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

/// The text under [`HEADING`] in a ticket's completion notes, up to the next
/// `##` or `###` heading, trimmed; `None` when the notes have no such
/// heading.
pub fn section(content: &str) -> Option<&str> {
    let notes = completion_notes(content)?;
    let start = line_start(notes, HEADING)?;
    let body = &notes[start + HEADING.len()..];
    let body = &body[body.find('\n').map_or(body.len(), |i| i + 1)..];
    let end = [
        heading_at_line_start(body, "### "),
        heading_at_line_start(body, "## "),
    ]
    .into_iter()
    .flatten()
    .min()
    .unwrap_or(body.len());
    Some(body[..end].trim())
}

/// Whether the completion notes have a line `Nothing for the playtest: <why>`
/// (as a bullet or not), with a reason after the colon.
pub fn says_nothing(content: &str) -> bool {
    completion_notes(content).is_some_and(|notes| {
        notes.lines().any(|line| {
            let line = line.trim_start();
            let line = line.strip_prefix("- ").unwrap_or(line);
            line.strip_prefix(NOTHING)
                .is_some_and(|why| !why.trim().is_empty())
        })
    })
}

/// Whether a done ticket of type `kind` is missing both the section (with
/// something in it) and the `Nothing for the playtest:` line.
pub fn missing(kind: &str, content: &str) -> bool {
    GAMEPLAY_TYPES.contains(&kind)
        && section(content).is_none_or(str::is_empty)
        && !says_nothing(content)
}

/// One done ticket's notes for the playtest.
#[derive(Debug, PartialEq, Eq)]
pub struct Notes {
    pub id: String,
    pub title: String,
    pub section: String,
}

/// A Markdown file of a ticket folder other than its `README.md`.
fn is_ticket_file(name: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        && name != "README.md"
}

/// Every done ticket's section, by file name order (so by id).
pub fn collect(repo_root: &Path) -> std::io::Result<Vec<Notes>> {
    let dir = repo_root.join("tickets").join("done");
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let name = entry?.file_name();
        if let Some(name) = name.to_str().filter(|n| is_ticket_file(n)) {
            names.push(name.to_owned());
        }
    }
    names.sort();
    let mut out = Vec::new();
    for name in names {
        let content = fs::read_to_string(dir.join(&name))?;
        let Some(text) = section(&content).filter(|s| !s.is_empty()) else {
            continue;
        };
        out.push(Notes {
            id: frontmatter_value(&content, "id")
                .unwrap_or_else(|| name[..4.min(name.len())].to_owned()),
            title: frontmatter_value(&content, "title").unwrap_or_default(),
            section: text.to_owned(),
        });
    }
    Ok(out)
}

/// A plain `key: value` of the frontmatter, quotes taken off.
fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let frontmatter = content.strip_prefix("---")?.split("\n---").next()?;
    frontmatter.lines().find_map(|line| {
        let value = line.strip_prefix(key)?.strip_prefix(':')?.trim();
        Some(value.trim_matches('"').to_owned())
    })
}

/// The listing `playtest-notes` prints: each ticket's id and title as a
/// heading, then its section.
pub fn render(notes: &[Notes]) -> String {
    let mut out = String::new();
    for note in notes {
        let _ = write!(
            out,
            "## {} — {}\n\n{}\n\n",
            note.id, note.title, note.section
        );
    }
    let _ = writeln!(
        out,
        "{} done tickets leave notes for the playtest.",
        notes.len()
    );
    out
}

/// `cargo xtask playtest-notes`: prints every done ticket's section.
pub fn command(repo_root: &Path, args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask playtest-notes");
        return 2;
    }
    match collect(repo_root) {
        Ok(notes) => {
            print!("{}", render(&notes));
            0
        }
        Err(e) => {
            eprintln!("playtest-notes: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICKET: &str = "---\nid: \"0477\"\ntitle: The danger zone counts arts\ntype: bug\n---\n\n\
        # 0477\n\n## Completion notes\n\n- Did it.\n\n### For the Act 1 playtest (1064)\n\n\
        - Red tiles show weapon reach only.\n\n### Later\n\nnot this\n";

    #[test]
    fn section_reads_up_to_the_next_heading() {
        assert_eq!(section(TICKET), Some("- Red tiles show weapon reach only."));
    }

    #[test]
    fn section_ends_at_the_next_top_heading_too() {
        let ticket = "## Completion notes\n\n### For the Act 1 playtest (1064)\n\nA feel.\n\n## Other\n\nno\n";
        assert_eq!(section(ticket), Some("A feel."));
    }

    #[test]
    fn section_ignores_a_heading_inside_a_code_fence() {
        let ticket = "## Completion notes\n\n### For the Act 1 playtest (1064)\n\n```\n### not a heading\n```\nend\n";
        assert_eq!(section(ticket), Some("```\n### not a heading\n```\nend"));
    }

    #[test]
    fn section_outside_the_completion_notes_does_not_count() {
        let ticket =
            "## Scope\n\n### For the Act 1 playtest (1064)\n\nx\n\n## Completion notes\n\nDone.\n";
        assert_eq!(section(ticket), None);
        assert_eq!(section("no notes at all"), None);
    }

    #[test]
    fn nothing_line_needs_a_reason() {
        let with = "## Completion notes\n\n- Nothing for the playtest: dev tooling only.\n";
        let bare = "## Completion notes\n\nNothing for the playtest:\n";
        assert!(says_nothing(with));
        assert!(!says_nothing(bare));
        assert!(says_nothing(
            "## Completion notes\n\nNothing for the playtest: no play.\n"
        ));
    }

    #[test]
    fn a_gameplay_ticket_with_neither_line_is_missing() {
        let bare = "## Completion notes\n\n- Did it.\n";
        let empty = "## Completion notes\n\n### For the Act 1 playtest (1064)\n\n";
        for kind in GAMEPLAY_TYPES {
            assert!(missing(kind, bare), "{kind}");
            assert!(missing(kind, empty), "{kind}");
            assert!(!missing(kind, TICKET), "{kind}");
        }
        assert!(!missing("infra", bare));
        assert!(!missing("design-decision", bare));
    }

    #[test]
    fn collect_lists_done_tickets_with_a_section() {
        let root =
            std::env::temp_dir().join(format!("xtask-playtest-notes-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let done = root.join("tickets/done");
        fs::create_dir_all(&done).unwrap();
        fs::write(done.join("0477-danger.md"), TICKET).unwrap();
        fs::write(
            done.join("0478-none.md"),
            "---\nid: \"0478\"\n---\n## Completion notes\n\nx\n",
        )
        .unwrap();
        fs::write(done.join("README.md"), TICKET).unwrap();

        let notes = collect(&root).unwrap();

        assert_eq!(
            notes,
            vec![Notes {
                id: "0477".to_owned(),
                title: "The danger zone counts arts".to_owned(),
                section: "- Red tiles show weapon reach only.".to_owned(),
            }]
        );
        let text = render(&notes);
        assert!(
            text.starts_with("## 0477 — The danger zone counts arts\n\n- Red tiles"),
            "{text}"
        );
        assert!(
            text.ends_with("1 done tickets leave notes for the playtest.\n"),
            "{text}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_real_repo_lists_0761_and_0477() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("xtask is at <repo>/crates/xtask");
        let ids: Vec<String> = collect(repo_root)
            .unwrap()
            .into_iter()
            .map(|n| n.id)
            .collect();
        for id in ["0477", "0761"] {
            assert!(ids.iter().any(|i| i == id), "{id} missing from {ids:?}");
        }
    }

    #[test]
    fn command_refuses_arguments() {
        assert_eq!(command(Path::new("."), &["--by-area".to_owned()]), 2);
    }
}
