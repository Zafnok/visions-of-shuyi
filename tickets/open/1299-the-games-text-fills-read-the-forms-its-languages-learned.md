---
id: "1299"
title: "The game's text fills read the forms its languages learned at load, and parse nothing on a frame"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1285"]
nick_input: none
completed:
---

# 1299 — The game's text fills read the kept forms

## Context

The engine pin move of 2026-11-18 passed engine PRs #499 (engine ticket
2266, the engine's ADR-0131) and #503 (2268, ADR-0132). A `Lang` now
learns each text's form when it loads (plain, a message with its parse
kept, or not a message), `Lang::form_in` and `Lang::data_form_in` hand it
out (`shuyi_assets::message::Form`), and the engine's `Ctx` key fills
(`text_with`, `text_styled`, `label`, …) read it and never parse. A
message's walk now borrows its values, so a select and a `{name}` copy
nothing into a `String`.

The game gets #503 at the pin, but not #499: its own fills take text it
already holds, so each call goes through `Form::of`, which parses a
message (a plural, a select) every time it is filled:

- `crates/ui/src/words.rs`'s `Words::text_with` (`Lang::text_in`, then
  `tips::fill_values`);
- `crates/ui/src/tips.rs`'s `fill_text`, `fill_styled` and
  `fill_values` (`shuyi_ui::text::fill`, `fill_styled_with`,
  `fill_values`);
- `crates/ui/src/profiles.rs`'s numbered name (`fill_values`).

The engine keeps a fill from a `Form` crate-private (`text::fill_form`),
so a game can't fill a form it got from `form_in` without the engine's
`Ctx`.

1285 measures the Playdate bench after engine 3219 (plain text fills
without a parse); this ticket follows it with its numbers.

## Nick input

None.

## Scope

**In:**
- Measure first: parses per frame in the bench scenarios (`cargo shuyi
  bench`; the engine's `parse-count` feature is tests-only, so count in a
  test through the Harness, or read Callgrind's `Message::parse` share on
  the Playdate's `battle-cursor-sweep`).
- Where a `Ctx` is at hand, fill by key through the engine's `Ctx` key
  fills, not `Words::text_with` plus `fill_values`.
- Where it isn't (views built from `Words`), fill from `form_in` /
  `data_form_in`. This needs a public fill from a `Form` in `shuyi-ui`:
  **ask Nick before filing that engine ticket** (`engine-change`, "ticket
  and follow-up"), then block this ticket's rest on it.
- If the measure shows the parses cost under about 1% of a Playdate
  frame, say so with the numbers, and do only the `Ctx` part.

**Out (do not do):**
- Changing any text or what it fills to.
- The game's dialogue scripts (`.dlg`), which aren't screen text.

Estimate: about 60 production lines changed at the call sites, no new
modules.

## Implementation steps

1. Count the parses per frame of the bench scenarios as they are (a
   test through the Harness that counts with the engine's `parse-count`
   feature on, or Callgrind).
2. Move each fill that has a `Ctx` onto its key fills.
3. For the rest, ask Nick about the engine ticket; once it is merged and
   pinned, fill `Words::text_with` and the tips from `form_in`.
4. Measure again; update `docs/targets.md`'s table if the bench moved.

## Acceptance criteria

- [ ] Every text fills byte for byte as before (the existing text, tip
      and words tests unchanged).
- [ ] The count of parses per frame before and after, in the completion
      notes; zero after the load in the battle scenarios, or the reason
      a fill still parses.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: a fill through the kept form equals the fill of the held text.
- Integration: a Harness frame of a battle parses nothing after the load.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
