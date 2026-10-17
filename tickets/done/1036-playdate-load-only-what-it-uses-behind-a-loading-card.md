---
id: "1036"
title: "Playdate: load only the data the Playdate build uses, behind a loading card, and measure it on the device"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1033"]
nick_input: none
completed: 2026-10-17
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

- [x] The Playdate build embeds only the listed files (the `.pdx`'s
      `pdex.bin` size before and after is in the PR).
- [x] The harness plays through Chapter 1 against the trimmed bundle.
- [x] The loading card shows within the first frame.
- [ ] Device numbers are in the PR; the follow-up ticket exists if over
      2 s.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Integration: Chapter 1 through the harness with the trimmed bundle.
- Manual: device run (numbers in the PR).

## Completion notes

**What was done** (ADR-0082):

- **The Playdate's bundle.** `crates/content/playdate-bundle.txt` lists
  what the Playdate leaves out of `assets/`: every `README.md`, the
  license texts, the PC's icons (`icon/`) and the sound files
  (`audio/sfx/`, until 1047). `build.rs` writes the Playdate's table and
  the paths it left out; the engine's `Layer::leaving_out` /
  `Bundle::left_out` let the audio check skip a left-out sound file
  instead of calling it missing. `pdex.bin`: **2,847,117 → 2,487,387
  bytes** (-359,730, -12.6%).
- **The test.** `crates/playdate/src/chapter_1.rs` starts the game with
  the front end's own code (`start.rs`, `card.rs`), on a thread reading
  the Playdate's bundle (the engine's `reading`), and plays Chapter 1 from
  New Game to "To be continued" (the battle with the PC test's recorded
  commands). It fails if the game reads a file the list left out, or if
  the bundle holds a file nothing read. Checked both ways by hand.
- **The loading card** (`src/card.rs`) replaces 1031's "Loading…": the
  first frame reads only the font (`fonts/atlas.ron`, `atlas_1bit.bin`)
  and draws the game's name with the one-bit renderer; the next frame
  loads the rest. The game then draws with that font. The SDK's text
  calls (`pd_clear`, `pd_draw_text`) are gone.
- **The measuring.** The front end watches every file read (the engine's
  `watch`) and times it to the next read; the start's steps that read no
  file are named (`(the player's settings and saves)`, `(the title)`).
  It logs when the card and the game's first frame were shown, the load
  time, the stack mark, and the ten slowest; `load-report.txt` holds every
  file. A Simulator run: card at 12 ms, data in 11 ms, the title at 49 ms
  (PC speed, not the device's).

**Deviations:**

- The list comes from the test, not from logging a Simulator play of
  Chapter 1 by hand: the test runs the same Rust code on every PR, so the
  list can't drift. The Simulator run checked the card and the report.
- The list is of what is *left out*, so a new data file reaches the
  Playdate by default (the test flags it if nothing reads it).
- Kept in the Playdate's bundle although the PC's: the full-colour
  pictures (portraits, battle art, title backdrops, which their loaders
  check against the data; 1040 replaces them with one-bit ones), the PC's
  colour themes (Options still offers them until 1045) and the PC font's
  PNG (the font check reads its size). All are read, so the test will say
  when they stop being.
- **No lazy loading.** Dialogue, the biggest file, is checked at load
  against the battles, chapters, supports, openings and cinematics, so
  loading it per scene is not a small change; it is noted for the binary
  format follow-up. What was left out isn't parsed, so the load time
  barely changes: the device numbers decide the follow-up.

**Claude's starting rule** (Nick can veto at 1050): the loading card is
the game's name, centred, black on the white ground, in the game's font.

**Notes for Nick:** the card shows for as long as the data loads (about
3.7 s on your Playdate in 1026), then the title.
