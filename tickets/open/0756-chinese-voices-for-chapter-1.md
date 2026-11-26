---
id: "0756"
title: Chinese voices for Chapter 1
type: content
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0045", "0726", "0722", "0755", "0180"]
nick_input: decision
completed:
---

# 0756 — Chinese voices for Chapter 1

## Context

Nick chose "Chinese voices too" in 0043
(`docs/design/voices-languages-and-script.md`, *Voices*). The same tools speak
Mandarin. Whether anyone checks the Chinese, and who, is 0045's question; never
assume Nick's wife reviews it. 0726 makes Chapter 1's Chinese text. 0755 does
the same job for Japanese and makes the manifest follow the language.

**After the writer's rewrite** (Nick, 2026-11-08; Lane J of *Act 1
critical path*): a human writer is being hired (0766) and may reword
every line. A line's id is a hash of its words, so a clip or a
translation made before the rewrite is redone after it. Don't start
this ticket while 0766 is open without asking Nick.

2026-11-17 friction sweep
([`docs/research/friction-2026-11-17.md`](../../docs/research/friction-2026-11-17.md),
bucket B1): 0180 is in `blocked_by`, since each clip under `voice/` is
a hand-written line in `shuyi.ron`'s disclosure lists (3 per clip
today) until 0180 writes them.

**From 0725 (2026-11-25).** The pack's code is `zhhans`. A Chinese line
that says `{lead}` needs the words the voice says for the name, in
Chinese (`{lead|流亡者}` as the starting rule, as English has
`{lead|the exile}`, 0754). A pack's token check doesn't allow a stand-in
yet (`assets-private/story/zh/README.md`, *Tokens*): whichever of 0755
and this ticket comes first makes the pack accept it and the voice tool
read it, before any clip of such a line is made.

## Nick input

**Decision:** one listening round before generating, as in 0755, of each Chapter
1 speaker's voice reading three Mandarin lines on both readers. Those who sound
wrong get newly designed voices (no pitch shifting).

## Scope

**In:** the listening round; `voice/zh/` clips, manifest and cast in the private
repository; a pin. The language folder's name follows what 0725 uses.

**Out (do not do):** Traditional Chinese; later chapters; a voice language
setting.

## Implementation steps

1. Make the round's page. Keep its source in `assets-src/audio/`.
2. Record any changed voices in `assets-src/audio/voice-refs/` as `<id>.zh.flac`,
   and add them to the design doc's cast table.
3. `cargo xtask voice generate --lang zh`. Transcribe each clip with a Mandarin
   speech-to-text model and compare with `spoken`; list mismatches. Regenerate
   bad ones; a line that fails three times stays silent and is listed in the
   notes. Apply 0045's answer on checking.
4. Commit to the private repository and pin.

## Acceptance criteria

- [ ] Nick's answers are in the design doc, verbatim.
- [ ] `cargo xtask voice status --lang zh` reports nothing missing for `ch01`, besides lines listed as left silent.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new; 0755's test covers the language.

## Completion notes

