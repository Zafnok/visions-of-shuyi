---
id: "0760"
title: "One form for `cast.ron`: drop the game's reader of the older one"
type: infra
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0721"]
nick_input: none
completed:
---

# 0760 — One form for `cast.ron`: drop the game's reader of the older one

## Context

0238 gave `cast.ron` the engine's form, speaker to voice id
(`shuyi_assets::voice::Cast`), read by `trpg_content::voice::cast_from_source`.
ADR-0107 rule 2 then said each entry names its reference clip and reader, and
0721 built that form for `cargo xtask voice`, the only thing that reads the
file (`docs/voice.md`, *`cast.ron`*). The game never reads it.

So two forms live side by side: the real cast in the private repository is
the tool's, while `crates/app/tests/voice/en/cast.ron` and the test that
reads it (`crates/app/src/tests.rs`,
`the_test_voice_folder_gives_the_game_its_three_clips`) keep the older one,
and `trpg_content::voice` still re-exports the engine's `Cast` and
`cast_from_source`.

## Nick input

None.

## Scope

**In:** the test folder's `cast.ron` in the tool's form; the game's reader
of the older form removed (or, if something still needs it, kept and named
as the older form); the docs' note on it.

**Out (do not do):** changing the engine's `voice` module. If it should
drop or change its `Cast` too, that is an engine ticket (`engine-change`
skill).

## Implementation steps

1. Rewrite `crates/app/tests/voice/en/cast.ron` in the tool's form (clip and
   reader per speaker; the test speakers may share a reference clip).
2. Remove `trpg_content::voice::{Cast, cast_from_source}` and
   `CAST_FILE`'s users that read the older form; have the app test check the
   cast with `cargo xtask voice`'s reader instead, or drop that half of it.
3. `docs/voice.md`: remove the note on the older form.
4. Tell the engine if its `Cast` has no user left (an engine ticket, not
   here).

## Acceptance criteria

- [ ] Every `cast.ron` in the repository has the tool's form.
- [ ] Nothing in the game reads the older form.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the test folder's cast parses with the tool's reader.
