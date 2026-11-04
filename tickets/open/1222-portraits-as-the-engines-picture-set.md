---
id: "1222"
title: "Portraits as the engine's picture set, with one list of the five expressions and one expression check"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2246"]
nick_input: none
completed:
---

# 1222 — Portraits as the engine's picture set

## Context

The game/engine boundary audit of 2026-11-03 found
`crates/content/src/portrait.rs` (112 production lines) generic and
marked E in `docs/engine/boundary.md` (line 170) but never moved, and
its rule written three times:

- `REQUIRED_EXPRESSIONS` (`portrait.rs:27`) and `STANDARD_EXPRESSIONS`
  (`dialogue.rs:61`) are the same five words, kept equal only by a test
  (`dialogue/tests.rs:494`);
- "an expression is one of the portrait's, or a standard one without a
  portrait" is in `dialogue/rules.rs` (`expression_problem`, lines
  114–136) and `cinematic.rs` (`characters_problems`, lines 556–574).

[engine#2246](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2246-a-picture-set-named-pictures-per-id.md)
adds `shuyi_assets::picture_set` (`PictureSetDef`, `PictureSetRon`,
`PictureRules`, `PictureSet`, `NamedPicture`, `name_problem`). This
ticket puts the portraits on it (ADR-0043 stands: one sidecar per
character, its images by name).

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- The pin past engine#2246.
- `portrait.rs` down to `PORTRAITS_DIR`, `FRAME_PX`, one `EXPRESSIONS`
  list, `RULES`, `pub type Portrait = PictureSet; pub type Expression =
  NamedPicture;`, `load_all` and one `expression_problem` worded for
  portraits; `STANDARD_EXPRESSIONS` goes (callers use `EXPRESSIONS`).
- `dialogue/rules.rs` and `cinematic.rs` call that one function; the
  cinematic's message takes the dialogue's wording, one per character.
- The sidecars' fields renamed to the engine's (`character` → `id`,
  `expressions` → `pictures`): `assets-test/portraits/*.ron`,
  `assets/portraits/*.ron` and its `README.md`, xtask's `sidecar_stub`
  (`crates/xtask/src/portrait_import.rs`) and its tests, and the private
  repository's `game/portraits/*.ron` (11 files) with
  `cargo shuyi private-assets --pin` in this PR (`AGENTS.md`, *Bought
  art*).
- Field uses renamed (`.character` → `.id`, `.expressions` →
  `.pictures`; about 45 lines in `content`, `ui` and `xtask`).
- `docs/engine/boundary.md` line 170.
- **Either order with 1219:** if 1219 is done, drop `portrait.rs` from
  the line-170 row it left; if not, remove `portrait.rs` from that row
  here (1219 then removes `tileset/look.rs`).
- **Either order with 1233:** if 1233 is done, edit the portrait row as
  it left it; if not, edit line 170 as it stands (1233 keeps this
  ticket's wording).

**Out (do not do):**
- How a portrait is drawn (`crates/ui/src/portrait.rs`), the lead's two
  portraits (`PORTRAIT_MALE`, `PORTRAIT_FEMALE`), which expressions exist.

## Implementation steps

1. **Pin** past engine#2246 (`engine-change` §7, `engine-pin` skill),
   unless an open `engine-pin` PR has; read its API.
2. `portrait.rs`: `RULES = PictureRules::new(&EXPRESSIONS, FRAME_PX)`;
   `load_all` is `bundle::load_each(PORTRAITS_DIR, "ron", |f|
   PictureSetDef::from_source(f.path, f.stem, f.text, &RULES)?.resolve(
   f.path, PORTRAITS_DIR, images, Some(bundle::install()), &RULES))`;
   `expression_problem(portraits: &[&Portrait], expression) ->
   Option<String>` words `name_problem(…, &EXPRESSIONS)` as
   `dialogue/rules.rs` does now. Delete `Sidecar`, `parse_portrait`,
   `check_image`; `tip.rs`'s test calls `from_source`/`resolve`.
3. `dialogue/rules.rs` and `cinematic.rs` on it; delete
   `STANDARD_EXPRESSIONS` and the equality test.
4. Register `PictureSetRon::new(RULES)` in `crates/content/src/formats.rs`.
5. Renames (above), private repository first.
6. `portrait/tests.rs`: delete what `engine#2246` tests (id, missing
   names, images, sizes, order); keep the embedded portraits loading,
   the five names, the 256 px frame, and one error coming back with the
   sidecar's file.
7. **Measure** with `cargo xtask code-lines`: estimate about 90
   production lines removed and 30 added; about 220 test lines removed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed engine#2246.
- [ ] `grep -rn "STANDARD_EXPRESSIONS\|REQUIRED_EXPRESSIONS\|fn parse_portrait\|fn check_image" crates`
      finds nothing.
- [ ] `grep -rn "character:\|expressions:" assets/portraits assets-test/portraits`
      finds nothing, and `assets-private.rev` is at the private repository's
      rename.
- [ ] The dialogue tests of expressions pass with their messages
      unchanged; every portrait loads (the all-assets and private-assets
      tests).
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the kept portrait tests; the dialogue and cinematic expression
  tests.
- Integration: the all-assets test and the private-assets tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
