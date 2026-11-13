---
id: "0497"
title: "Vix speaks the tips that show outside battle (Load Game, Profile)"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["0494"]
nick_input: none
completed: 2026-11-12
---

# 0497 — Vix speaks the tips that show outside battle (Load Game, Profile)

## Context

Two tips show over screens outside battle (0874): `export_save` (the Load
Game list) and `export_profile` (the Profile screen), through
`TipScreen` (`crates/ui/src/tips/screen.rs`). 0494 made their clips
(`voice/en/vix/`), but `TipScreen` never asks for a voice: only the
battle screen says the guide's lines (`GuideVoice`,
`crates/ui/src/screens/battle/guide_voice.rs`; `grep -rn play_voice
crates/ui/src` finds the battle's and the debug menu's only). So her box
there is silent in every build.

## Nick input

None. Verified by tests; how she sounds goes on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- `TipScreen` preloads its tip's line when made, says it as the box comes
  up (`ctx.play_voice(&trpg_content::voice::guide_line(&tip.text))`, only
  when `ctx.has_voice`), and stops it when the box closes.
- `docs/voice.md`, "How the game plays them": the tips outside battle.

**Out (do not do):**
- Rewording a line (AGENTS.md rule 3).
- Which keys close the box (0495).

## Implementation steps

1. In `TipScreen`, say the line on its first `update` (a `said: bool`),
   after `ctx.preload_voices` of it when the screen is made
   (`TipScreen::once` has the `Ctx`).
2. On `Transition::Pop`, `ctx.stop_voice()` when it was said.
3. Tests in `crates/ui/src/tips/screen.rs`: with a manifest holding the
   tip's clip, the first frame asks for it once; Confirm stops it; with
   voices off nothing is asked for (the battle's tests in
   `crates/ui/tests/it/lessons.rs` show how to build the manifest).

## Acceptance criteria

- [x] The tip box over the Load Game list asks for its clip once and
      stops it when closed (test).
- [x] Nothing is asked for with voices off (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `TipScreen`'s voice requests.

## Completion notes

- `TipScreen` keeps its tip's guide line: `once` preloads it, the first
  `update` says it (only when `ctx.has_voice`, which is false with voices
  off), and Confirm/Cancel stops it when it was said. A Confirm on the very
  first frame closes the box without saying it.
- Tests in `crates/ui/src/tips/screen.rs`: preload on open, one request on
  the first frame and none after, Stop on Confirm; nothing at all with
  voices off. They reuse the battle's manifest helper
  (`screens::battle::guide_voice::tests::voiced`, now `pub(crate)`) rather
  than a third copy of it.
- `docs/voice.md`: the tips outside battle are voiced.
- No gameplay rule decided. How she sounds over Load Game and Profile goes
  on the Act 1 playtest's checklist (1064).
