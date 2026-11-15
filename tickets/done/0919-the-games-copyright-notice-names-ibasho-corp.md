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
copyright notices still name Nick Wentz as the owner. Nick approved the
`LICENSE` change himself (AGENTS.md hard rule 6: `LICENSE` is his call),
2026-11-14.

## Nick input

None. Nick asked for the change and approved the `LICENSE` edit; he
reviews the PR.

## Scope

**In:**
- `LICENSE`: the line `Copyright (c) 2026 Nick Wentz. All rights reserved.`
  becomes `Copyright (c) 2026 Ibasho Corp. All rights reserved.` Nothing
  else in `LICENSE` changes.
- Every other copyright or ownership line in the repository that names
  Nick Wentz as the owner (README, Cargo metadata, the about and credits
  screens, store or reveal kit text, file headers) names Ibasho Corp.
  instead.

**Out (do not do):**
- Author and credit lines ("Directed by Nick Wentz", an `author` field):
  they stay.
- `THIRD_PARTY_ASSETS.md`: unchanged.
- Shuyi Engine, its dependency and its notices: Nick owns the engine
  personally under MIT OR Apache-2.0.
- Any other wording in `LICENSE` (its definitions, its attribution text):
  only Nick decides those.

## Implementation steps

1. Search the repository, without `target/`, for `Nick Wentz`,
   `Copyright` and `(c) 20`.
2. Change the `LICENSE` line above.
3. Change each other line found that states ownership; leave author and
   credit lines.
4. List in the completion notes what was left and why.

## Acceptance criteria

- [x] `LICENSE` line 3 reads `Copyright (c) 2026 Ibasho Corp. All rights reserved.`
      and `git diff` shows no other change to `LICENSE`.
- [x] No ownership or copyright line outside `LICENSE` names Nick Wentz.
- [x] `THIRD_PARTY_ASSETS.md` and the engine pin are unchanged.
- [x] `cargo xtask ticket-lint` passes; the change is text no code reads, so
      the other gates run in CI on the PR.

## Tests required

- None: text only. `cargo xtask ticket-lint` and CI's checks.

## Completion notes

Changed two lines: `LICENSE` line 3 and the copyright sentence of
`README.md`'s *License* section. Nothing else in the repository states
ownership: no Cargo manifest has `authors` or a copyright field, the public
credits, store and reveal kit text name no owner, no source file has a
copyright header, and the private repository's `content/` and `story/`
have no copyright line either.

Left as they are, on purpose:

- `LICENSE` line 13, `"Licensor" means Nick Wentz.`, and line 51, the
  attribution text `"tactical-rpg by Nick Wentz"`: the ticket keeps the rest
  of `LICENSE` unchanged. The licensor is normally the owner, so line 13
  now disagrees with line 3; whether it becomes Ibasho Corp. is Nick's
  call, in a PR of its own.
- `crates/xtask/src/playdate.rs`, `AUTHOR = "Nick Wentz"`: the Playdate
  bundle's `author` field, an author line, not ownership.
- `THIRD_PARTY_ASSETS.md` and the Shuyi Engine pin and notices.

Gates: text only, so no build or test reads it (nothing in `crates/`
reads the notice); `cargo xtask ticket-lint` passes.
