---
id: "0253"
title: "The language files' key placeholders are checked against the game's actions"
type: feature
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0253 — The language files' key placeholders are checked against the game's actions

## Context

Filed by the engine pin move of 2026-11-10 (`docs/engine/pins.md`,
engine #304, engine ticket 2576). The engine now has
`shuyi_ui::text::check_key_placeholders::<A: ActionSet>(bundle, path,
text)`: in a language file's screen text, a capitalized `{Name}` names
a key, so it must be `{Cursor}` or an action of `A`; a lowercase
`{name}` is a value a screen passes and always passes. Each error names
the file, the key's line, `SH2008` and the known actions. The engine's
`BasicData::load` calls it, so `cargo shuyi check` fails on a
misspelled action there; the game isn't on `BasicData`, so it gets
nothing yet.

What the game checks today:
- tips: `shuyi_assets::tip`'s rules with the game's placeholder test
  (`crates/content/src/tip.rs` ~:146);
- lessons: its own loop (`crates/content/src/lesson.rs` ~:394, "unknown
  placeholder");
- **the UI text: nothing.** `assets/lang/en/ui.ron` holds 126 lines
  with a capitalized placeholder (`{Confirm}`, `{Cancel}`, `{Cursor}`,
  `{CursorUp}`, `{EndTurn}`, `{DangerZone}`, `{DialogueLog}` …), and the
  test pack `assets-test/lang/test/ui.ron` more. A misspelled one
  (`{Confirn}`) reaches the player as braces. The engine found exactly
  that shipped in its own examples.

The game's actions are `trpg_content::keymap::Action`, which implements
`ActionSet` (`crates/content/src/keymap.rs` ~:260).

## Nick input

None. Verified by tests.

## Scope

**In:**
- Every language pack's UI text (each pack's `ui.ron`, the source
  `en/ui.ron` and the `test` pack) checked with
  `check_key_placeholders::<Action>` where the game's text is loaded, so
  `cargo test` (and the game's own `cargo shuyi check` if it loads the
  text) fails on a placeholder that names no action, with the engine's
  message.
- If the lessons' own loop (`lesson.rs`) can call the same check without
  a new dependency, replace it and delete the loop; if not, leave it and
  say why in the completion notes.
- Fix any placeholder the check finds (a key name only; never reword a
  line: the dialogue-writing pause, `AGENTS.md` rule 3, covers the words,
  not a misspelled key name).

**Out (do not do):**
- Tips (already checked through the engine's tip rules).
- `.dlg` scripts (their own checks, `cargo xtask check-script`).
- New actions or keys (`keyboard-input` skill).

## Implementation steps

1. Read engine #304's diff (`crates/ui/src/text.rs`,
   `check_key_placeholders`) and where `BasicData::load` calls it.
2. Find where the game reads each pack's `ui.ron` into its text map
   (`crates/content/src/lang/data.rs`, and `Ctx::embedded` in
   `crates/ui/src/screen.rs`). `trpg_content` may not link `shuyi_ui`:
   if so, run the check in `crates/ui` (the load, or a test over every
   embedded pack, as the other content checks do).
3. Add the check; add a test with a misspelled placeholder in a small
   bundle that fails with `SH2008` and the file and line.
4. Run it over the real packs and fix what it finds.
5. Expected size: about 40 production lines and 40 of tests.

## Acceptance criteria

- [ ] A UI text with `{Confirn}` fails the game's load or test with the
      engine's `SH2008` message naming the file and line.
- [ ] `{Cursor}`, every `Action` name and a lowercase `{name}` pass.
- [ ] Every shipped and test pack passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the misspelled placeholder, the passing ones.
- Integration: every embedded pack's UI text checked.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
