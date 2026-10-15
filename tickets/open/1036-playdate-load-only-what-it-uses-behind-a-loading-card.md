---
id: "1036"
title: "Playdate: load only the data the Playdate build uses, behind a loading card, and measure it on the device"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1033"]
nick_input: none
completed:
---

# 1036 — Playdate loading time

## Context

1026 measured **3.7 s** on Nick's Playdate (Rev B) to parse every data
file at start-up with the game's own RON loader (ADR-0070 §5; the PC takes
5 ms). ADR-0070 §5 plans: load only what the Playdate version uses, behind
a loading card, measure again, and convert the data to a binary format at
build time only if it is still too slow.

What the Playdate build doesn't need (`docs/design/playdate.md`): other
languages' packs (one build per language, 0061 Q5), the test files
(`assets-test/`, never in a shipped build anyway, ADR-0057), the PC-only
data (keyboard layouts and the layout picker's keys, colour palettes
and themes, the PC map skins' sprite data and tilesets if the Playdate
map skin doesn't draw them, full-colour images: 1040 makes one-bit ones),
and any chapter after Chapter 1 once there are some.

## Nick input

None to decide. Nick sideloads one build to measure.

## Scope

**In:**
- **A Playdate bundle:** the embedded file table (1028) for the Playdate
  build holds only what it uses. Decide the list from what the Playdate
  front end actually loads (log every bundle lookup in a Simulator run of
  Chapter 1 and compare) and keep it in one place (e.g. a
  `playdate-bundle.ron` include/exclude list read by 1028's `build.rs`
  under a cargo feature, or a filter in `xtask playdate`). A test checks
  that every file the Playdate's screens look up is in it, by running
  the harness through Chapter 1 against the trimmed bundle.
- **Loading the rest lazily** where the loader allows it, e.g. a
  scene's dialogue when the scene starts, if that is a small change;
  otherwise note it for the binary-format follow-up.
- **A loading card:** the first frame after start shows the game's
  one-bit look (the title logo or the game's name in the game's font,
  via the one-bit renderer, 1032) before the data is parsed, so the
  screen is never blank or the SDK's default. The font must be
  available before the content loads (the 1-bit atlas is embedded on its
  own). Its look is a placeholder in the game's style; Nick sees it at the
  1050 play-through. *Claude's starting rule (Nick can veto): the card
  shows the game's name, centred.* Replace 1031's temporary
  "Loading…" text.
- **Measure on the device:** load time before and after (1026: 3,708 ms),
  and which files take longest (log per file in a debug build).
- **If the load is still over 2 s on the device** (technical target, not
  a design rule: Panic's guidance is that games should start promptly),
  write the follow-up ticket "Playdate: convert the data to a binary
  format at build time" (`write-ticket` skill) with the measured numbers
  and the slowest files, blocked by this one, and add it to 1050's
  `blocked_by` and the ROADMAP's Playdate bullet.

**Out (do not do):**
- A binary data format (the follow-up above, only if needed).
- Changing what the PC or web builds embed or load.
- Other languages (1049).

**Carve (1103):** either order works. If the engine's bundle (1103, the
bundle as layers) landed first, the Playdate bundle is a layer list
instead of a filtered table.

## Implementation steps

1. Read ADR-0070 §5, `crates/content/src/bundle.rs` and 1028's
   `build.rs`.
2. Log lookups in the Simulator through Chapter 1; write the list.
3. The trimmed bundle and the harness test.
4. The loading card.
5. Build; ask Nick to sideload; record the numbers.

## Measured on Nick's Playdate

Nick sideloads and starts the game twice; he pastes the Simulator console.
The PR lists the start-up time to the loading card and to the title, the
slowest files, and the stack high-water mark.

## Acceptance criteria

- [ ] The Playdate build embeds only the listed files (the `.pdx`'s
      `pdex.bin` size before and after is in the PR).
- [ ] The harness plays through Chapter 1 against the trimmed bundle.
- [ ] The loading card shows within the first frame.
- [ ] Device numbers are in the PR; the follow-up ticket exists if over
      2 s.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Integration: Chapter 1 through the harness with the trimmed bundle.
- Manual: device run (numbers in the PR).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
