---
id: "0919"
title: The game's copyright notice names Ibasho Corp.
type: infra
milestone: M8 Release
model: haiku-4.5 / gpt-6-luna
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-14
---

# 0919 — The game's copyright notice names Ibasho Corp.

## Context

Ibasho Corp. now owns Visions of Shuyi: Nick assigned the game to it in a
signed Confirmatory IP Assignment dated 2026-11-14. The repository's
copyright notices still name Nick personally as the owner. Nick approved the
`LICENSE` change himself (AGENTS.md hard rule 6: `LICENSE` is his call),
2026-11-14.

## Nick input

None. Nick asked for the change and approved the `LICENSE` edit; he
reviews the PR.

## Scope

**In:**
- `LICENSE`: the copyright line, which named Nick personally, becomes `Copyright (c) 2026 Ibasho Corp. All rights reserved.`
- Every other copyright or ownership line in the repository that names
  Nick personally as the owner (README, Cargo metadata, the about and credits
  screens, store or reveal kit text, file headers) names Ibasho Corp.
  instead.
- Added in review (Nick, 2026-11-14: "keep it all consistent. Ibasho Corp.
  everywhere"): `LICENSE`'s licensor definition and attribution text, and
  the Playdate bundle's `author` field, name Ibasho Corp. too.

**Out (do not do):**
- Personal credit lines (a "Directed by" line naming Nick): they stay.
- `THIRD_PARTY_ASSETS.md`: unchanged.
- Shuyi Engine, its dependency and its notices: Nick owns the engine
  personally under MIT OR Apache-2.0.
- Any other wording in `LICENSE`: only Nick decides that.

## Implementation steps

1. Search the repository, without `target/`, for Nick's full name (both spellings),
   `Copyright` and `(c) 20`.
2. Change the `LICENSE` line above.
3. Change each other line found that states ownership or names the
   game's author; leave personal credit lines.
4. List in the completion notes what was left and why.

## Acceptance criteria

- [x] `LICENSE` line 3 reads `Copyright (c) 2026 Ibasho Corp. All rights reserved.`;
      its licensor (line 13) and attribution (line 51) name Ibasho Corp.
- [x] No ownership, copyright or author line outside `LICENSE` names Nick
      personally; `playdate::tests::pdxinfo_names_the_game_and_its_version` passes.
- [x] `THIRD_PARTY_ASSETS.md` and the engine pin are unchanged.
- [x] `cargo xtask ticket-lint` passes; CI runs the other gates on the PR.

## Tests required

- Unit: the existing `pdxinfo` test, updated to the new author.

## Completion notes

Changed:

- `LICENSE` line 3, the copyright line.
- `LICENSE` line 13, `"Licensor" means Ibasho Corp.`, and line 51, the
  attribution text `"tactical-rpg by Ibasho Corp."` (Nick, in review: Ibasho
  Corp. everywhere). The rest of `LICENSE` is unchanged.
- `README.md`'s *License* section, the copyright sentence.
- `crates/xtask/src/playdate.rs`: `AUTHOR`, the Playdate bundle's `author`
  field, and the test of the `pdxinfo` it writes.

Nothing else states ownership: no Cargo manifest has `authors` or a
copyright field, the public credits, store and reveal kit text name no
owner, no source file has a copyright header, and the private repository's
`content/` and `story/` have no copyright line. Done tickets that quote the
old name (0905, 1031) were updated too (below).

Nick, in review: no file names him in full, except where it is about
Shuyi Engine, which he owns personally. Two done tickets that quoted the
old attribution and Playdate author (0905, 1031) now name Ibasho Corp., and
this ticket no longer quotes his full name. No file of the repository or of
the private repository's `content/` and `story/` names him in full now.
The git history (old commits' text and author) still does: rewriting a
public repository's history is out of this ticket.

Unchanged: `THIRD_PARTY_ASSETS.md`, the Shuyi Engine pin and its notices.

Checks: `cargo xtask ticket-lint` and the `pdxinfo` test pass locally; CI
runs the rest.
