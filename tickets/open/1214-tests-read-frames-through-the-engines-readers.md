---
id: "1214"
title: "Tests read frames through the engine's readers, and the Harness forwards its scripts with the engine's macro"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3204", "1237"]
nick_input: none
completed:
---

# 1214 — Tests read frames through the engine's readers

## Context

From the game/engine boundary audit of 2026-11-03. The engine had no
public way to read a frame's text, so the game's tests hold about 54
copies of a `row`/`shows`/`text(buf, x, y, w)` helper over
`GlyphBuffer::get` (`tests/it/battle.rs:148-160`,
`class_change.rs:23-35`, `battle/magic_tests.rs:94-116`,
`skill_tests.rs:43`, `item_tests.rs:76`, …) and 7 copies of the
`every_glyph_drawn_is_in_the_font` test (`tests/it/battle.rs:300`,
`credits.rs:180`, `key_bindings_screen.rs:353`, `layout_picker.rs:58`,
`options.rs:675`, `rebind_buttons.rs:348`, `title.rs:155`), plus
`assert_in_font` in `battle/turn_tests.rs:59`. And
`crates/ui/src/harness.rs:170-237` re-declares 10 engine scripts
(`music_load_delay`, `without_music`, `use_pad`, `keys`, `type_text`,
`hold`, `pad`, `hold_pad`, `wait`, `clear_audio`) as forwarders, so they
chain with the game's own lookups.

[engine#3204](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3204-tests-read-a-frames-text-and-a-games-harness-keeps-the-scripts.md)
adds `GlyphBuffer::row_text`, `text_at`, `rows`, `shows`,
`glyphs_missing`, the same on the engine's Harness, and
`shuyi_platform::forward_scripts!`. This ticket moves the game onto
them. It follows 1237, which gathers the shared test drivers and moves
some behavior asserts onto views, and leaves these helpers to this
ticket.

## Nick input

None. Tests only; nothing a player sees changes.

## Scope

**In:**
- Move the pin past engine#3204 (`engine-change` skill §7, the
  `engine-pin` skill), unless an open `engine-pin` PR has done so.
- `harness.rs`: the 10 forwarders become one `forward_scripts!` line.
  `system_menu`, the start-ups and the lookups stay.
- Every pure copy of a frame text helper goes; a test calls
  `buf.row_text(y)`, `buf.text_at(..)`, `buf.shows(..)`, `h.row(y)` or
  `h.shows(..)`. A helper that names a region (`help`: row 31 left of
  the debug hint; `panel`) stays as one line over `text_at`.
- The font tests keep one test per screen, each one assert on
  `glyphs_missing`.
- Expected size: about 40 production lines removed, 1 added; about
  350-420 test lines removed.

**Out (do not do):**
- Moving behavior asserts from cells to views (1237 does that).
- The `map_scene`/`map_scene_in`/`map_text` method and free-function
  pairs in `harness.rs` (about 9 lines; the MCP tools hold a `Core`).
- Helpers with another job (`title_cinematic.rs`'s `Shows`,
  `key_bindings_screen.rs`'s `shows(h, action, text)` over the view).

**Either order with 1232:** if 1232 is done, the `tests/it` files and
tests it deleted need nothing here; if not, convert them here, and 1232
deletes them as they are.

**Either order with 1211:** if 1211 is done, `tests/it/credits.rs` is
trimmed to the game's tests: convert its kept font check and snapshot
split where they now are; if not, convert them as listed, and 1211 trims
the file as this ticket left it.

## Implementation steps

1. **Pin** (as above).
2. **Harness.** In `crates/ui/src/harness.rs`, delete the 10
   forwarders and write
   `shuyi_platform::forward_scripts!(Harness, 0, Shuyi);`. Keep the
   module doc's sentence on chaining, naming the macro.
3. **Helpers.** `grep -rnE "fn (row|rows|shows|text|screen_text)\("
   crates/ui` lists them (about 54 in `src/screens/**/tests.rs`,
   `*_tests.rs`, inline `mod tests` and `tests/it/*.rs`). Delete each
   one whose body only reads cells and replace its calls (`.trim()`
   where the copy trimmed: the engine's readers don't). Inline cell
   loops in a test (`tips/glyph.rs`'s tests) go the same way.
4. **Font tests.** Each `every_glyph_drawn_is_in_the_font` becomes
   `assert_eq!(h.glyphs_missing(&font), [])` on its screen;
   `turn_tests.rs`'s `assert_in_font` becomes the same on its buffer,
   as do the inline checks in `tests/it/battle.rs:282` and
   `title.rs:261`. The other splits of a snapshot at `--- colours ---`
   (`credits.rs:20`'s `glyphs`, `options.rs:182`'s `glyphs_shown`)
   read `h.rows()` instead.
5. **Measure** with `cargo xtask code-lines` and record the row in
   `docs/engine/friction.md` (estimate: 40 production lines removed).

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#3204.
- [ ] `crates/ui/src/harness.rs` has no `pub fn keys` (or other of the
      10 forwarders) and calls `forward_scripts!`.
- [ ] `grep -rn "map_or(' ', |c| c.glyph)" crates/ui` finds nothing
      (inline reads, as in `tips/glyph.rs`'s tests, use the readers
      too).
- [ ] `grep -rn --include=*.rs -- "--- colours ---" crates/ui` finds
      nothing.
- [ ] The same tests pass, none removed (`cargo test --no-fail-fast`).
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- The existing unit and integration tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
