---
id: "0149"
title: "Typos ignores commit hashes by pattern, not by allowing `ba` everywhere"
type: bug
milestone: M0 Foundation
model: haiku-4.5 / gpt-6-luna
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0149 — Typos ignores commit hashes by pattern, not by allowing `ba` everywhere

## Context

#574 (1240) met a `typos` failure on the engine pin's hash `c1ba9ec8` in
`docs/engine/pins.md`: typos split it as "c1", "ba" and "9ec8". It passed
by adding `ba = "ba"` to `_typos.toml`'s words, so a real `ba` typo now
passes anywhere in the repository. That weakens a gate, which
`AGENTS.md` forbids. #514 hit a hash too, and so did the retro's own
engine PR (Zafnok/shuyi-engine#314, on a pin hash written short). The
engine's `_typos.toml` ignores only 16-digit snapshot hashes
(`extend-ignore-re = ['\b[0-9a-f]{16}\b']`), so neither repository yet
handles a commit hash. Found by the retro of 2026-11-08.

## Nick input

None.

## Scope

**In:**
- `_typos.toml` gets an `extend-ignore-re` for hex hashes of 7 to 40
  characters that contain a digit (for example
  `\b(?=[0-9a-f]*[0-9])[0-9a-f]{7,40}\b`, so a hex-only word like
  `deadbeef` is still read). Tell the engine about the pattern in a
  `Game NNNN:` note so it can do the same.
- Remove `ba = "ba"` and any other word allowed only for a hash.
- A line in `run-gates`' typos section: never allow a word that is a
  fragment of a hash.

**Out (do not do):**
- Other typos rules.

## Implementation steps

1. Check that the typos version CI pins supports look-ahead in
   `extend-ignore-re`. If it doesn't, list the pattern without it.
2. Edit `_typos.toml` (lines around 43–44) and run `typos`.
3. Add the `run-gates` line, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] `_typos.toml` has no `ba` entry, and `typos` passes on the tree.
- [ ] A file with the word `ba` alone, not in a hash, fails `typos`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Manual: the two typos runs above, in the notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
