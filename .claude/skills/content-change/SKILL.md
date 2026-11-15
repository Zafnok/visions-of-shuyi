---
name: content-change
description: This game's rules on top of the engine pack's shuyi-private-files for its content and story documents in the private assets repository — its names (`assets-private/content/`, `story/`, the pin `assets-private.rev`), the ticket in each name, the pin on the private branch while both pull requests are open, what the pin carries, `check-leaks` and the dialogue pause. Read shuyi-private-files first for the steps (checkout, `--no-track` branch, the two pull requests, `--pin`, two pull requests moving it, never in public), then this for the game's names in place of the template's `private/` and `private.rev`. Use when a ticket edits, adds, renames or removes a file in `assets-private/content/` or `assets-private/story/`, when two pull requests both move that pin, or when a game pull request's CI needs content that isn't on the private repository's `main` yet. Not for the bought art in `game/` and `test/` (ADR-0040 and ADR-0051, the work-ticket skill's credits) or for the engine (engine-change).
metadata:
  version: "2.0"
---

# Change the private content

**Read [`shuyi-private-files`](../shuyi-private-files/SKILL.md) first.** It
holds every step a Shuyi game with private files follows: the checkout,
a branch there made with `--no-track`, never pushing the private `main`,
pushing by the branch's full name, the private pull request first and the
game's that moves the pin with `cargo shuyi private-assets --pin`, listing
what the pin carries, two pull requests that both move it, and what never
goes in public. This skill holds only what is this game's (ADR-0117).

## This game's names

`cargo shuyi private-assets` reads them from `shuyi.ron`'s
`private_assets` (`checkout`, `pin_file`, `folders`), so the commands work
as written; the skill's text uses the template's names. Read them as:

| `shuyi-private-files` says | This game has |
| -------------------------- | ------------- |
| `private/` | `assets-private/` |
| `private.rev` | `assets-private.rev` |
| `private/files/`, laid over `crates/game/files/` | `assets-private/content/`, laid over `assets/` (but `content/tests/`, never embedded: ADR-0118 §3) |
| `private/docs/` | `assets-private/story/`: beats, bible, characters, outline, ledger, and `story/design/` |
| the private repository | [`Zafnok/visions-of-shuyi-assets`](https://github.com/Zafnok/visions-of-shuyi-assets) |
| `the_private_files_are_the_checkouts`, `cargo clean -p <crate>` | `bundle.rs`'s test that every layer is its folder, `cargo clean -p trpg-content` |
| the game's gates (`shuyi-gates`) | the `run-gates` skill |

The checkout also holds `game/` and `test/`, the **bought art**: not this
skill. It needs a credit (ADR-0051), the gates never read it (ADR-0040
§5), and `--pin` refuses a bought file without one. Never put a content
file in `game/`, or bought art in `content/`. The tools that read
`assets/` from disk read the content over it too (`cargo xtask
check-script`, `voice`, `cargo shuyi check-keys`, `lint`).

## The ticket in every name

- The private branch: `t<NNNN>-<slug>`, after the game ticket.
- Its commits: `[NNNN] <what changed>` (`git -C assets-private add
  content story`).
- The private pull request's title starts with `[NNNN]`: it is the commit
  the squash merge makes on `main`, which "What the pin carries" below
  reads.
- Its body names the game ticket, says it merges first, and links the
  game's pull request by full URL
  (`https://github.com/Zafnok/visions-of-shuyi/pull/<N>`). The game's
  names `Zafnok/visions-of-shuyi-assets#<N>` and says the pin moves to
  its merge.

## The game's CI reads the new content before either merges

Unlike the template's default, the game's pull request does not wait on
the old pin. While the private one is open, point the pin at its branch's
commit by hand, commit it with the game's change and push:

```bash
git -C assets-private rev-parse HEAD > assets-private.rev
```

CI checks out exactly that commit. **A game pull request never merges
with the pin on a branch commit:** after the squash merge that commit is
on no branch, and the next fetch of it may fail. Once the private pull
request merges, move the pin to `main` with `--pin` as
`shuyi-private-files` says.

A change to `story/` alone moves the pin too, so every worktree that
checks out the pin has the documents.

## What the pin carries

List only the content and story commits; art commits (`game/`, `test/`,
`voice/`) don't break a build:

```bash
git -C assets-private log --oneline "$(cat assets-private.rev)..origin/main" -- content story
```

For each that isn't this ticket's, take its `[NNNN]` and check its game
pull request merged: the ticket is in `tickets/done/` on the game's
`main`, or `gh pr list --repo Zafnok/visions-of-shuyi --search "[NNNN]"
--state merged`. If not, wait, or ask its session (`SendMessage` if it is
linked here); never pin past it, and never pin a commit before it that
leaves this ticket's out. Run it again whenever `main` moves.

## What this game adds to "never in public"

- `cargo xtask check-leaks` (ADR-0118 §4, in `run-gates`) finds a tracked
  file that quotes a script line or a map row. A tracked file may name a
  content id, never quote a line, draw a real map or tell the plot
  (ADR-0118 §2). A test that needs plot facts keeps them in
  `content/tests/`.
- No build that embeds the content is uploaded as a workflow artifact
  (ADR-0117 §4).
- **No bare `@word`** in either repository's titles, bodies, commits or
  comments: GitHub reads it as a mention. A dialogue directive goes in
  backticks (`` `@music` ``) or loses its `@`.

## Dialogue is paused

No agent writes or rewords a script line (Nick, 2026-11-08, until the
writer of ticket 0766): a `content/dialogue/` change follows the
`story-writing` skill for what is still allowed, and a writer's change
comes in through the `story-import` skill.
