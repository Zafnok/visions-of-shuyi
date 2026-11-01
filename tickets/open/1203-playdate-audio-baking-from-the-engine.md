---
id: "1203"
title: Bake Playdate audio with the shipped engine converter
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["engine#3029"]
nick_input: none
completed:
---

# 1203 — Playdate audio baking from the engine

## Context

The 2026-11-01 PR audit found engine PR #201 (3029) has shipped and is
already pinned, while `crates/xtask/src/playdate_audio.rs` keeps the same
OGG/WAV decode, mono conversion and IMA ADPCM writer. Its 754-line source
was part of oversized 1170. Adopt the public engine
`shuyi_cli::export::playdate::audio` API now, independently of the runtime
port; 1170 needs this adapter first and 1206 removes the remaining build
wrapper afterward. ADR-0084 fixes the current audio format and selection.

## Nick input

None. Host tests of the produced files verify this mechanical reuse.
No device, new setup or gameplay check after merge.

## Scope

**In:**
- A small game adapter selecting all current sound variants and exactly
  `crates/content/playdate-music.txt`'s music, with the same output paths.
- Engine `GameAudio`/`audio::jobs`, `audio::bake` and cache helpers in place
  of the game's decoder, mono conversion, ADPCM encoder and cache rules.
- Keep current `cargo xtask playdate` working on those engine outputs.
  Retain only game-specific inventory/report formatting still used.

**Out (do not do):**
- Backend/start/SDK migration (1170), retired-source removal (1204/1205),
  old exporter removal (1206), sale packaging (1052) and voices (1048).
- New sounds, changed cues/volumes, a new format, bought art or new engine
  APIs. This adapter passes the existing game selection into shipped APIs.

## Implementation steps

1. Verify 3029 at the pin. Read its public `Job`, `jobs`, `decode`,
   `adpcm_wav`, `cache_dir` and `bake`; record the current source/destination
   inventory and small deterministic PCM/ADPCM fixtures before editing.
2. Build `GameAudio` from the game's manifest/files, filtered by the
   existing Playdate music selection. Reuse engine job building and baking
   with a versioned/checksummed cache; no parallel encoder remains here.
3. Delete the old conversion implementation and its algorithm tests.
   Keep adapter tests of selection, destinations, missing sources and
   generated output compatibility. Preserve the build command and count
   report until 1170/1206 replace them.
4. Remove `hound`/`lewton` dependencies only if no other game tool uses
   them; update existing audio/build docs and the friction ownership row.

Expected size: about 60–100 adapter lines and 754 old source/test lines
removed, with focused adapter checks; under 1,400 changed lines including
docs and this ticket's move.

## Acceptance criteria

- [ ] Existing sound/music selection and output paths match the baseline;
      every needed variant is present and unselected music is absent.
- [ ] Engine-baked fixture audio has the same decoded samples/ADPCM format
      as before; current device/Simulator build plans consume its paths.
- [ ] A missing selected source fails naming it. Reusing cached output
      after a source-byte change cannot silently return the old audio.
- [ ] No game-local Vorbis/WAV decoder, mono mixer or ADPCM encoder remains
      in the Playdate tool path, including moved/renamed copies.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Host integration: selected cue/file inventory, output compatibility,
  source refusal and cache invalidation through the actual engine bake API.
- Do not duplicate the engine codec unit tests in this adapter.

## Completion notes

*(Filled in by the session that completes the ticket: retained game policy,
removed implementation, comparison evidence and 1170/1206 handoff.)*
