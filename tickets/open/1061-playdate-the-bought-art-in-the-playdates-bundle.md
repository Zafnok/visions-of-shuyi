---
id: "1061"
title: "Playdate: the bought art in the Playdate's bundle, as one-bit pictures"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1040"]
nick_input: none
completed:
---

# 1061 — The bought art in the Playdate's bundle

## Context

1040 (ADR-0083) made the Playdate's pictures one-bit when its bundle is
built: `crates/content/build.rs` converts every picture of the kinds in
`crates/content/src/playdate_images.rs` (portraits, combat pictures,
battle and title backdrops) and puts the one-bit file at the PNG's path
in the Playdate's table. But that table is made from `assets/` only: the
Playdate's bundle has **no private layer** (ADR-0070 §3: "the test files
and private layer stay `std`-only"; ADR-0082). So a Playdate build only
ever shows the public placeholders, never the bought faces, fighters and
backdrops 0062 decided (`docs/design/playdate.md`, *One-bit pictures*).

`cargo xtask playdate-images` already converts the bought art into
`assets-private/generated/playdate-images/` to look at; nothing embeds it.

1041 (the combat picture with the bought backdrop) and 1042 (one-bit
portraits of the bought faces, with touch-ups) need the bought art on the
Playdate to be checked and signed off in the Simulator or on Nick's
Playdate.

## Nick input

None.

## Scope

**In:**
- **A private layer in the Playdate's bundle**, with the `private-assets`
  feature (ADR-0040): `build.rs` also writes a table of
  `assets-private/game/` (the same `playdate-bundle.txt` rules where they
  apply), its pictures converted by the same table with the private
  battler sidecars' scales, and `bundle::playdate_layers()` lays it over
  the public one as `bundle::layers()` does on the PC (`.private()`).
- **Touched-up pictures win** (1042): a `<stem>.1bit` file beside a PNG
  in `assets-private/game/` is embedded as that PNG's picture as it is,
  instead of converting the PNG. `playdate-images` copies it to its
  output instead of converting, so what it shows is what ships.
- **`cargo xtask playdate --private-assets`** builds with the bought art
  (a `private-assets` feature on `trpg-playdate` forwarding to
  `trpg-content`'s), as `cargo xtask web --private-assets` does. Without
  it, the placeholders, as today.

**Out (do not do):**
- Making or touching up pictures (1042), the screens that show them
  (1041, 1043, 1044).
- Any gate reading `assets-private/` (CLAUDE.md: gates never read it).
- Selling builds (1052 decides which build ships).

## Implementation steps

1. Read ADR-0083, ADR-0082, ADR-0040, `crates/content/build.rs`,
   `crates/content/src/bundle.rs` (`playdate_layer`, `layers`),
   `crates/xtask/src/web.rs` (its `--private-assets`).
2. `build.rs`: when `CARGO_FEATURE_PRIVATE_ASSETS` is set, walk
   `assets-private/game/`, convert its pictures (`playdate_images::jobs`
   with a reader of that folder), honour a `.1bit` beside a PNG, and
   write `playdate_private_files.rs`; otherwise write an empty table.
3. `bundle.rs`: `playdate_layers()` puts
   `Layer::table(PRIVATE_DISPLAY_ROOT, PLAYDATE_PRIVATE).private()` on
   top when the feature is on.
4. `trpg-playdate`'s `private-assets` feature; `xtask playdate
   --private-assets` (parse, plan, tests); `playdate-images` copies a
   touched-up `.1bit`.
5. Build with the bought art and look at a portrait and a fighter in the
   Simulator (a screenshot of the running game may be shown; no file
   made from the art is committed).

## Acceptance criteria

- [ ] `cargo xtask playdate --private-assets` builds a `.pdx` whose
      portraits are the bought faces in one-bit (seen in the Simulator).
- [ ] Without the feature the Playdate's bundle is as before (the
      Chapter 1 test passes unchanged).
- [ ] A `.1bit` file beside a PNG in the private layer is embedded as it
      is (a `build.rs`-level or xtask test on a scratch folder, not the
      bought art).
- [ ] Nothing made from the bought art is committed.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: `xtask playdate` parses `--private-assets` and passes the
  feature; `playdate-images` copies a touched-up file.
- Manual: the Simulator with the bought art.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
