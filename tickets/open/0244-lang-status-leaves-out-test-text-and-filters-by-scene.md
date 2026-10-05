---
id: "0244"
title: "lang-status leaves out test text and can be narrowed to a file"
type: feature
milestone: M1 Engine
model: sonnet-5
effort: low
status: todo
blocked_by: ["0235"]
nick_input: none
completed:
---

# 0244 — lang-status leaves out test text and can be narrowed to a file

## Context

Since 0235, `cargo xtask lang-status <code>` lists every data text and
dialogue line a pack lacks
([ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)).
That includes text no player sees: the test scenes (`assets/dialogue/test*.dlg`),
the test chapter and battles, the `test_*` names and characters. A real
pack never translates those (0719 says so), so its report would list
them as missing for ever, and "nothing missing" could never be read off
it. 0719's acceptance asks exactly that for `ui.ron`, `names.*`, the data
Chapter 1 shows and `ch01`.

Found while working 0235.

## Nick input

None.

## Scope

**In:** what `lang-status` counts and prints.

**Out (do not do):** what the game loads or shows; the validation of
packs; any translation.

## Implementation steps

1. Decide what is test-only text from the data, not a list in code: lines
   of scenes in `test*.dlg` files, and data keys whose id starts with
   `test` (`names.test_knight`, `chapters.test.title`, `battles.test.*`).
   The report leaves them out and says how many it left out. The test
   pack (`assets/lang/test/`) is the exception: its report keeps them.
2. Options to narrow the report: `--only ui|data|dialogue` and
   `--file <dlg stem>` (the lines and captions of one `.dlg` file; this
   needs the file each scene came from, which `DataText` doesn't keep
   today).
3. Print the report in three parts (screen text, data, dialogue), each
   with its counts.

## Acceptance criteria

- [ ] A pack that translates everything but the test text reports nothing missing (test).
- [ ] `--file ch01` lists only `ch01.dlg`'s lines and captions (test).
- [ ] The test pack's report still lists the test scene's untranslated lines (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the filter, each option, the counts.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
