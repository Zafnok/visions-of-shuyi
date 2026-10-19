---
id: "1157"
title: "Move the engine parts of open game tickets to the engine repository, and point their dependencies there"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: xhigh
status: todo
blocked_by: ["1131", "1152"]
nick_input: none
completed:
---

# 1157 — Move the engine parts of open game tickets to the engine repository

## Context

1131 moves everything under `engine/` to
[`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine) with
`git filter-repo`, deletes `engine/` here, and makes the game depend on
the engine by git URL and a pinned revision (`shuyi-engine.rev`,
`cargo xtask engine-pin`). From then on an engine change is an engine
PR first, then a game PR that moves the pin. 1152 adds the
`engine-change` skill and teaches `cargo xtask ticket-lint` to accept
`engine#NNNN` ids in `blocked_by`.

Open game tickets written before 1131 still name files under `engine/`,
game paths that have since moved into `engine/`, or plans that put code
in the engine crates (`shuyi-assets`, `shuyi-core`, `shuyi-ui`,
`shuyi-app`, `shuyi-cli`). Neither 1131 nor 1152 rewrites them. After
1131, a session picking one up looks for files that no longer exist in
this repository and would edit a copy of the engine that is not there.
This ticket splits each one into an engine ticket (in the engine
repository) and a game ticket (here) that waits for it.

**Why it waits for 1152 as well as 1131:** the game tickets this ticket
rewrites get `engine#NNNN` in their `blocked_by`. Until 1152 lands,
`ticket-lint` (`check_blocked_by_exists` in `crates/xtask/src/tickets.rs`)
rejects any id it does not find in this repository, so the gates fail.
1152's `engine-change` skill is also the procedure this ticket follows
for each split.

**1153 waits for this ticket.** The first engine-friction sweep files
engine tickets and game follow-ups. If it runs while open game tickets
still hold unfiled engine work, it either files duplicates or measures
game code against plans that are about to move. 1153's `blocked_by`
names 1157.

The engine repository's ticket rules are in its
[`tickets/README.md`](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/README.md):
new engine tickets are numbered by area (`20xx` repository, `21xx` core
and simulation, `22xx` assets and formats, `23xx` frame, screens and
skins, `24xx` input, `25xx` tools, …), and a game ticket is named there
as `game#NNNN`. The game tickets moved on 2026-10-14 (1119–1129,
1132–1135, 1140–1150) kept their numbers; tickets moved whole by this
ticket get **new** engine numbers in the right block: that README reads
any other four-digit id as a game ticket, so keeping a game number
there would be misread. Re-check the README when you start; follow it if it says
otherwise.

ADR-0092 (`docs/adr/`) and 1151's `engine-friction` skill: **the engine
never names a game concept.** Every engine ticket written here describes
a general engine feature that a non-tactics game would also want.

## Nick input

None.

## Scope

**In:**
- **Find every open game ticket with engine work.** Search
  `tickets/open/` for:
  - paths under `engine/`;
  - `crates/...` paths that no longer exist here but whose file exists
    in the engine repository at the pinned revision (in 1131's moved
    layout: `crates/ui/...` → the engine's `crates/ui/...`, and so on);
  - plans that put code in a `shuyi-*` crate or name `shuyi_assets`,
    `shuyi_core`, `shuyi_ui`, `shuyi_app` or `shuyi-cli`.

  Starting list, found on 2026-10-18 (re-check; there may be more):

  | Game ticket | Engine part |
  | ----------- | ----------- |
  | **0840** Options: last row sits on the box border | The engine's options skin, `engine/crates/ui/src/screens/options/glyph.rs`. If 0840 is already done, skip it |
  | **1044** Playdate title screen and intro cinematic | `engine/crates/ui/src/frame/cinema.rs` |
  | **0236** Wide glyphs (Japanese font) | `engine/crates/ui/src/frame/glyph_buffer.rs`, `engine/crates/app/src/render.rs`, `engine/crates/cli/src/font_atlas.rs` |
  | **0239** Chinese glyphs: a wide font per language | `engine/crates/app/src/render.rs`, `engine/crates/cli/src/font_atlas.rs` |
  | **0461** Road runs up to a bridge | `engine/crates/ui/src/grid/corners.rs` (the ticket still says `crates/ui/src/map_view/corners.rs`) |
  | **1014** Mods folder, manifest and load order | `engine/crates/app/src/storage/native.rs` |
  | **1011** ADR on how mods work | Plans runtime layers and the bundle in `shuyi_assets`, and a modder tool on `shuyi-cli` |
  | **1017** Scripting runtime and scripted skill effects | Plans the script runtime in `shuyi-core` |

  Not engine work, leave alone: mentions of `docs/engine/` (the game's
  own engine docs, e.g. in 0059 and 0060), and 1131, 1152 and 1153
  themselves.
- **Split each one** (the `engine-change` skill's "ticket and follow
  up" path):
  - The engine part becomes a new engine ticket, written in a checkout
    of the engine repository with **its** `write-ticket` skill and
    template. It names the game ticket as `game#NNNN` and describes the
    general feature (for 0236: "wide glyphs that take two cells in the
    glyph buffer, and a font atlas with them", not "Japanese for the
    tactics game"). It passes 1151's leak test: no game ids other than
    the `game#` link, no game concepts.
  - The game ticket keeps only the game part. It gets `engine#NNNN` in
    `blocked_by`, loses every stale engine path, and says in Context
    which engine ticket does the rest. Where the game part needs the
    pin moved, its steps say `cargo xtask engine-pin`.
  - A game ticket that is entirely engine work moves whole: write the
    engine ticket with the full content, then delete the game file, as
    was done for the tickets moved on 2026-10-14. Add a "Moved to the
    engine" line under *Numbering* in `tickets/README.md` with the old
    game id and the new engine id. Retarget anything that named it.
- **Redirect dependencies.**
  - Every game ticket whose `blocked_by` names a ticket that moved
    whole now names its `engine#NNNN`.
  - Every engine ticket that needs a game ticket first names it as
    `game#NNNN`.
  - `docs/ROADMAP.md`: where it lists a split or moved ticket (the
    lines for 0236, 0239, 0461, 1011, 1014, 1017, 1044), name the
    engine ticket beside it (`engine#NNNN`) or replace the moved id.
- **Check the tool commands.** About twenty open tickets say `cargo
  xtask frame-png | private-assets | lang-status | check-keys |
  check-text | font-atlas | web`. Those forward to `shuyi-cli` as a
  library dependency of `crates/xtask` (`crates/xtask/src/main.rs`,
  `fn shuyi`), so they should keep working with a git dependency.
  - Run each of them once against the pinned engine; note the result
    in the Completion notes.
  - The `cargo shuyi` alias in `.cargo/config.toml` is
    `run -q -p shuyi-cli --`, which needs `shuyi-cli` in this
    workspace. 1131's scope does not mention it. If 1131 left it
    broken, fix it here: point the alias at the forwarding in xtask
    (for example `run -q -p xtask -- shuyi --`, or whatever `fn shuyi`
    accepts), or, if that needs more than a one-line change and a
    `CLAUDE.md` edit, write a ticket for it instead. Either way update
    `CLAUDE.md` (the `shuyi.ron` row and the Environment notes) so the
    command it documents works.

**Out (do not do):**
- doing any of the engine or game work the split tickets describe;
- moving the code, the pin, `ticket-lint`'s `engine#` support or the
  `engine-change` skill (1131, 1152);
- the friction sweep (1153);
- rewriting done tickets in `tickets/done/` (history stays as it was);
- tickets in the engine repository that are not linked to a game
  ticket.

## Implementation steps

1. Check out the engine repository at the pinned revision (`cargo xtask
   engine` if 1131 added it, otherwise `git clone` into the scratchpad
   and `git checkout $(cat shuyi-engine.rev)`).
2. Search `tickets/open/` as described in Scope and write the full list
   (ticket, stale paths, engine crate, split or move whole) in a scratch
   table. Compare it with the starting list.
3. In the engine checkout, on a branch, write one engine ticket per
   engine part with the engine's `write-ticket` skill. Group parts that
   are one engine feature (0236 and 0239 share the wide-glyph and
   font-atlas work: one engine ticket for wide glyphs, and a second only
   if the per-language font is a separate feature). Run the engine's
   `ticket-lint` and open the engine PR; link it from this PR's body.
4. Here, rewrite each game ticket (or delete it if moved whole), add
   the `engine#NNNN` ids, and fix every `blocked_by` that pointed at a
   moved ticket. Search both ways: `grep -rn "<old id>" tickets/open
   docs/ROADMAP.md`.
5. Update `docs/ROADMAP.md` and `tickets/README.md`.
6. Run the tool commands; fix or ticket the `cargo shuyi` alias; update
   `CLAUDE.md`.
7. Run `cargo xtask ticket-lint` and `cargo xtask ticket-lint
   --check-engine` (the second checks the new `engine#` ids exist).

## Acceptance criteria

- [ ] `cargo xtask ticket-lint` passes, and `cargo xtask ticket-lint
      --check-engine` finds every `engine#NNNN` id.
- [ ] `grep -rnE "\bengine/crates|shuyi[-_](assets|core|ui|app|cli)" tickets/open`
      finds nothing except in 1131, 1152 and 1153, and no open game
      ticket names a `crates/...` path that does not exist in this
      repository.
- [ ] Every split pair links both ways: the game ticket has
      `engine#NNNN` in `blocked_by`, and the engine ticket names
      `game#NNNN`.
- [ ] Every engine ticket written here names no game concept (1151's
      leak test).
- [ ] `cargo shuyi frame-png` (or the command `CLAUDE.md` now documents)
      runs, and each `cargo xtask` tool command listed in Scope runs.
- [ ] The engine repository's PR with the new tickets is open or
      merged, and linked from this PR.
- [ ] The Completion notes list each game ticket moved or split, with
      its engine id.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (tickets and docs only), beyond `ticket-lint` in both
  repositories.

## Completion notes

*(Filled in by the session that completes the ticket.)*
