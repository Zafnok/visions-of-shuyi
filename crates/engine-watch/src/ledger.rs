//! The pin ledger, `docs/engine/pins.md` (ADR-0096 §6): one section per pin
//! move, newest last, headed `## YYYY-MM-DD: <from 8 hex> → <to 8 hex> (N
//! engine PRs)`, with a row and a decision for every *Breaking*, *Unblocks*
//! and *New for games* engine PR. `cargo xtask engine-pin --check` fails on
//! what [`problems`] and [`gains_undecided`] find.

use std::collections::HashSet;
use std::hash::BuildHasher;

/// The ledger's path from the repository's root.
pub const PATH: &str = "docs/engine/pins.md";

/// One row of a section's table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerRow {
    /// The first cell: the engine PR.
    pub pr: String,
    /// The last cell: what the game does about it.
    pub decision: String,
}

/// One pin move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// The heading's text, after `## `.
    pub heading: String,
    /// The day of the move, `YYYY-MM-DD`.
    pub date: String,
    /// The commit pinned, as the heading writes it (8 hex digits).
    pub to: String,
    /// Its table's rows.
    pub rows: Vec<LedgerRow>,
}

fn is_hex(text: &str) -> bool {
    (7..=40).contains(&text.len())
        && text
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// A heading's day and the commit it pins.
fn heading(text: &str) -> Option<(String, String)> {
    let (date, rest) = text.split_once(": ")?;
    let valid_date = date.len() == 10
        && date.chars().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        });
    let to = rest.split_once(" → ")?.1.split([' ', ',', '(']).next()?;
    (valid_date && is_hex(to)).then(|| (date.to_owned(), to.to_owned()))
}

/// The ledger's sections, or what is wrong with its form.
pub fn parse(text: &str) -> Result<Vec<Section>, String> {
    let mut sections: Vec<Section> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if let Some(title) = line.strip_prefix("## ") {
            let (date, to) = heading(title).ok_or_else(|| {
                format!(
                    "{PATH}:{}: expected `## YYYY-MM-DD: <from> → <to 8 hex> …`, found `{line}`",
                    i + 1
                )
            })?;
            sections.push(Section {
                heading: title.to_owned(),
                date,
                to,
                rows: Vec::new(),
            });
            continue;
        }
        let Some(inner) = line.trim().strip_prefix('|') else {
            continue;
        };
        let cells: Vec<&str> = inner
            .strip_suffix('|')
            .unwrap_or(inner)
            .split('|')
            .map(str::trim)
            .collect();
        let is_rule = cells
            .iter()
            .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'));
        if is_rule || cells.first() == Some(&"Engine PR") {
            continue;
        }
        let Some(section) = sections.last_mut() else {
            return Err(format!(
                "{PATH}:{}: a table row before the first section",
                i + 1
            ));
        };
        section.rows.push(LedgerRow {
            pr: cells.first().map_or_else(String::new, |c| (*c).to_owned()),
            decision: if cells.len() > 1 {
                cells.last().map_or_else(String::new, |c| (*c).to_owned())
            } else {
                String::new()
            },
        });
    }
    Ok(sections)
}

/// The tickets an `adopt → NNNN` decision names.
pub fn adopted(decision: &str) -> Vec<String> {
    decision
        .split("adopt → ")
        .skip(1)
        .filter_map(|rest| {
            rest.get(..4)
                .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
        })
        .map(str::to_owned)
        .collect()
}

/// The engine PR a row's first cell names: `#88` or `[#88](…)`.
fn pr_number(cell: &str) -> Option<u32> {
    let digits = cell.split_once('#')?.1;
    let end = digits
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(digits.len());
    digits[..end].parse().ok()
}

/// Whether a decision fixes a break and says nothing about adopting.
fn is_fixed_only(decision: &str) -> bool {
    decision.starts_with("fixed:")
        && !decision.contains("adopt → ")
        && !decision.contains("not for the game:")
}

/// The engine PRs of a section's rows with only a `fixed:`: the ones to ask
/// GitHub about.
pub fn fixed_only(section: &Section) -> Vec<u32> {
    section
        .rows
        .iter()
        .filter(|row| is_fixed_only(&row.decision))
        .filter_map(|row| pr_number(&row.pr))
        .collect()
}

/// What is wrong with a section's rows given `both`, the engine PRs the
/// report lists under *Breaking* and *New for games*: a row with only a
/// `fixed:`.
pub fn gains_undecided(section: &Section, both: &[u32]) -> Vec<String> {
    fixed_only(section)
        .into_iter()
        .filter(|number| both.contains(number))
        .map(|number| {
            format!(
                "{PATH}, `{}`: engine PR #{number} also adds public items, so its row needs `fixed: …; adopt → NNNN` or `fixed: …; not for the game: …` (the `engine-pin` skill)",
                section.heading
            )
        })
        .collect()
}

/// What is wrong with the ledger: its last section isn't `pin`, a row has
/// no decision, or an `adopt → NNNN` names none of `ticket_ids` (the
/// tickets in `tickets/open/` and `tickets/done/`).
pub fn problems<S: BuildHasher>(
    sections: &[Section],
    pin: &str,
    ticket_ids: &HashSet<String, S>,
) -> Vec<String> {
    let mut out = Vec::new();
    match sections.last() {
        None => out.push(format!("{PATH} has no section: it needs one per pin move (the `engine-pin` skill)")),
        Some(last) if !pin.trim().starts_with(&last.to) => out.push(format!(
            "{PATH}: the last section pins {}, but shuyi-engine.rev is {}: write this pin move's section (the `engine-pin` skill)",
            last.to,
            pin.trim()
        )),
        Some(_) => {}
    }
    for section in sections {
        for row in &section.rows {
            if row.decision.is_empty() {
                out.push(format!(
                    "{PATH}, `{}`: the row of {} has no decision",
                    section.heading, row.pr
                ));
            }
            for id in adopted(&row.decision) {
                if !ticket_ids.contains(&id) {
                    out.push(format!(
                        "{PATH}, `{}`: `adopt → {id}` names no ticket in tickets/open or tickets/done",
                        section.heading
                    ));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIN: &str = "6e7f71fd10e6ef49bc0c6f0f2250a9527c29a1e8";

    fn ledger(rows: &str) -> String {
        format!(
            "# Engine pins\n\n## 2026-10-25: baseline → 3f2256d4\n\nThe pins before.\n\n\
             ## 2026-10-26: 3f2256d4 → 6e7f71fd (26 engine PRs), #430\n\n\
             | Engine PR | Ticket | What | Decision |\n| --- | --- | --- | :-: |\n{rows}\n\
             Nothing for the game: 2 PRs (#75, #79)\n"
        )
    }

    fn ids(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn a_ledger_parses_into_its_sections_and_rows() {
        let text = ledger(
            "| #88 | 2013 | Constructors | fixed: built with `new` |\n| #78 | 2511 | A prelude | adopt → 1191 |",
        );
        let sections = parse(&text).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].to, "3f2256d4");
        assert_eq!(sections[0].date, "2026-10-25");
        assert_eq!(sections[0].rows, [] as [crate::ledger::LedgerRow; 0]);
        assert_eq!(sections[1].to, "6e7f71fd");
        assert_eq!(sections[1].rows.len(), 2);
        assert_eq!(sections[1].rows[0].pr, "#88");
        assert_eq!(sections[1].rows[1].decision, "adopt → 1191");
        assert_eq!(
            problems(&sections, PIN, &ids(&["1191"])),
            [] as [std::string::String; 0]
        );
    }

    #[test]
    fn a_row_before_any_section_or_a_bad_heading_is_an_error() {
        let err = parse("# Engine pins\n\n| not | a section |\n").unwrap_err();
        assert!(err.contains(":3: a table row before"), "{err}");
        let err = parse("## 2026-10-25 baseline\n").unwrap_err();
        assert!(err.contains(":1: expected `## YYYY-MM-DD"), "{err}");
        assert!(parse("## 2026-1-25: a → 3f2256d4\n").is_err());
        assert!(parse("## 2026-10-25: a → main\n").is_err());
        // The date is ten characters, the commit 7 to 40 hex digits.
        assert!(parse("## 2026-10-251: a → 3f2256d4\n").is_err());
        assert!(parse("## 2026-10-25: a → zzzzzzzz\n").is_err());
        assert!(parse("## 2026-10-25: a → 3f2\n").is_err());
        assert!(parse("## 2026-10-25: a → 3f2256d\n").is_ok());
    }

    #[test]
    fn the_last_section_must_be_the_pin() {
        let sections = parse("## 2026-10-25: baseline → 3f2256d4\n").unwrap();
        let found = problems(&sections, PIN, &ids(&[]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("last section pins 3f2256d4"), "{found:?}");
        assert!(found[0].contains(PIN), "{found:?}");
        assert_eq!(
            problems(&sections, "3f2256d46ac066ee\n", &ids(&[])),
            [] as [std::string::String; 0]
        );
        let none = problems(&[], PIN, &ids(&[]));
        assert!(none[0].contains("has no section"), "{none:?}");
    }

    #[test]
    fn a_row_needs_a_decision() {
        let text = String::from(
            "## 2026-10-26: 3f2256d4 → 6e7f71fd\n| Engine PR | Ticket | What | Decision |\n| --- | --- | --- | --- |\n| #88 | 2013 | Constructors |  |\n| #90 |\n",
        );
        let found = problems(&parse(&text).unwrap(), PIN, &ids(&[]));
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found[0].contains("the row of #88 has no decision"),
            "{found:?}"
        );
        assert!(
            found[1].contains("the row of #90 has no decision"),
            "{found:?}"
        );
    }

    #[test]
    fn a_fixed_row_of_a_pr_that_also_adds_needs_an_adoption_decision() {
        let section = |decision: &str| {
            let text = format!(
                "## 2026-11-02: 1eaa1fa0 → 6e7f71fd\n| #88 | 2013 | Constructors | {decision} |\n| [#90](https://x/90) | 2024 | More | fixed: `new` |\n"
            );
            parse(&text).unwrap().remove(0)
        };
        let only = section("fixed: built with `new`");
        assert_eq!(fixed_only(&only), [88, 90]);
        let found = gains_undecided(&only, &[88]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("engine PR #88 also adds"), "{found:?}");
        // Only breaks: nothing else to decide.
        assert_eq!(gains_undecided(&only, &[]), [] as [String; 0]);
        for decided in [
            "fixed: built with `new`; adopt → 1191",
            "fixed: built with `new`; not for the game: no modules",
        ] {
            assert_eq!(fixed_only(&section(decided)), [90]);
            assert_eq!(gains_undecided(&section(decided), &[88]), [] as [String; 0]);
        }
        // Other decisions aren't asked about.
        assert_eq!(fixed_only(&section("unblocks 1180")), [90]);
    }

    #[test]
    fn a_row_names_its_pr_as_text_or_as_a_link() {
        assert_eq!(pr_number("#88"), Some(88));
        assert_eq!(pr_number("[#102](https://x/102)"), Some(102));
        assert_eq!(pr_number("#"), None);
        assert_eq!(pr_number("88"), None);
    }

    #[test]
    fn an_adoption_must_name_a_ticket() {
        let text = "## 2026-10-26: 3f2256d4 → 6e7f71fd\n| #78 | 2511 | A prelude | adopt → 1191 (after engine#2600), adopt → 1192 |\n";
        let sections = parse(text).unwrap();
        assert_eq!(adopted(&sections[0].rows[0].decision), ["1191", "1192"]);
        let found = problems(&sections, PIN, &ids(&["1191"]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].contains("`adopt → 1192` names no ticket"),
            "{found:?}"
        );
        assert_eq!(
            adopted("not for the game: templates"),
            [] as [std::string::String; 0]
        );
        assert_eq!(adopted("adopt → 12"), [] as [std::string::String; 0]);
    }
}
