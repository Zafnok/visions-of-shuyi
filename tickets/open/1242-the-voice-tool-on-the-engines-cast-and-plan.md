---
id: "1242"
title: "The voice tool on the engine's cast and clip plan"
type: infra
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["engine#2250", "engine#2251"]
nick_input: none
completed:
---

# 1242 — The voice tool on the engine's cast and clip plan

## Context

`cargo xtask voice` (0721) keeps two generic pieces of its own: a cast
reader (`crates/xtask/src/voice/files.rs`: `Cast`, `Voice`, `Reader`,
`cast`, `Cast::entry`) and a plan of which clips to make, make again,
record again or remove (`crates/xtask/src/voice/plan.rs`), with the
recording script built from it (`script` in `crates/xtask/src/voice.rs`).
0760 stopped the game using the engine's old, thinner `voice::Cast`.
Rather than drop that from the engine, the engine grows a cast a voice
tool can use (engine#2250) and the plan (engine#2251), so other games
get them; this ticket moves the tool onto both (ADR-0092: engine feature).

What stays the game's: running the speech programs, levelling and checking
takes (`synth.rs`), `say.ron`, `sent.ron`, the reference clips'
fingerprints, and which variants the lead's lines need.

## Nick input

None.

## Scope

**In:**
- ~~Move the pin past engine 2250 and 2251 (`engine-change` skill, step 7;
  `engine-pin` skill's review for the engine PRs it passes).~~ (done by the pin move of 2026-11-10).
- The tool reads `cast.ron` with the engine's `cast_from_source`, with
  `Reader` as its settings. The lead's two voices become the engine's
  speaker-and-variant keys (`lead.m`, `lead.f` in place of `lead_m`,
  `lead_f`): in the private repository's `voice/en/cast.ron` (then
  `cargo shuyi private-assets --pin`), `crates/app/tests/voice/en/cast.ron`,
  the xtask tests' `TEST_CAST`, `script --character`'s help, and
  `docs/voice.md`.
- `plan.rs` becomes the hooks the engine's plan takes (variants of a
  line, the words a variant says, why a generated clip's voice changed:
  reference fingerprint, tool name, `say.ron` spelling) and the tool's
  `status`, `generate` and `script` read the engine's `VoicePlan`.
- Delete what moved.

**Out (do not do):**
- Any change to how clips are made, levelled or checked.
- Any change to the game's playback (`trpg_content::voice`).

## Implementation steps

1. ~~Move the pin (`cargo xtask engine-pin`), with the review and the
   ledger section.~~ (done by the pin move of 2026-11-10).
2. `files.rs`: drop `Cast`, `Voice`, `cast` and `Cast::entry`; read the
   cast with the engine's, `Reader` as `S`. Rename the lead's keys
   everywhere listed above.
3. `plan.rs`: keep only the hooks; call the engine's plan.
4. `voice.rs`: `status`, `generate` and `script` on the engine's plan and
   `recording_script`.
5. The xtask voice tests keep passing with only the renames and the new
   types; add none of the engine's own cases.

## Acceptance criteria

- [ ] `crates/xtask/src/voice/` has no cast reader and no clip planning of
      its own.
- [ ] `cargo xtask voice status` on the test scene prints what it printed
      before (the xtask tests' expected output, with the lead's keys
      renamed).
- [ ] Every `cast.ron` in the repository and the private repository uses
      the engine's form.
- [ ] Production lines fall (`cargo xtask code-lines`); the friction row
      in `docs/engine/friction.md` says by how much.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing xtask voice tests, updated for the renames; the
  test folder's cast parses with the engine's reader.
