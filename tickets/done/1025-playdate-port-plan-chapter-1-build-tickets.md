---
id: "1025"
title: "Playdate port: plan the build tickets for Chapter 1"
type: research
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1026", "0061"]
nick_input: none
completed: 2026-10-15
---

# 1025 — Playdate port: plan the build tickets for Chapter 1

## Context

Nick, 2026-10-07: "could we make a Playdate console port? […] as far as
controls limitations we have solutions like chording A+B if we need to or
otherwise have A/B off of a unit act differently like it already does and
the rewind mechanic can use the winding mechanism on the side. this is for
hella after launch but if we just reuse the retro glyph stuff it could
totally scale down to Playdate I think".

Nick, 2026-10-15, unparking it: "I wanna rush a playdate port!!! how can
we get ch1 on playdate asap?!" He chose to run the technical test first
and not wait for the engine carve (1103–1131). The work is split:

- **1026**: the technical test. Do `core`, `content` and `ui` build for
  the device without `std`? How long does an enemy turn take? Which
  bindings and toolchain? It writes the Playdate front-end ADR.
- **0061**: Nick's decisions (screen size and layout, telling things
  apart without colour, buttons and crank, what is left out, where it is
  sold), recorded in `docs/design/playdate.md`.
- **This ticket**: once both are done, write the build tickets that put
  Chapter 1 on the Playdate.

The engine carve goes on as planned. Carve tickets 1108 (console size)
and 1110 (platform shell) must keep a Playdate front end building if one
exists by then (noted in those tickets).

What the first look found (2026-10-07; check again when revived, the
numbers come from memory of the Playdate documentation, not a fresh read):

- **Screen.** The game draws a 100 × 32 grid of 8 × 16 px cells
  (`crates/ui/src/console.rs`): 800 × 512 px. The Playdate screen is
  400 × 240 px, black and white only (no greys, no colour). With today's
  font that is 50 × 15 cells, under a quarter of the room. So it is not a
  scale-down: every screen needs its own smaller layout, and the battle
  screen's 70-cell map plus 30-cell side panel (ADR-0018) cannot both stay
  on screen. A smaller font (6 × 8 px gives 66 × 30 cells) buys room at the
  cost of reading comfort on a 2.7-inch screen.
- **No colour.** Sides, ready/acted units, danger zones, terrain and
  weapon types are told apart by colour today (`docs/design/look-and-feel.md`).
  On one-bit they need another sign: inverted cells, dither patterns,
  letter case, blinking. Portraits and the bought map art are colour pixel
  art; the glyph map skin (ADR-0049) is the natural Playdate look.
- **Buttons.** D-pad, A, B, the crank, and the system Menu button (the
  system menu can carry up to three of the game's own entries). The game
  has 14 actions (`docs/design/controls.md`), but only cursor, Confirm,
  Cancel and End turn are required, and the map menu already opens from
  Cancel with nothing to cancel or Confirm on an empty tile.
- **Crank.** Rewind goes back to any earlier action in the battle
  (`docs/design/death-and-difficulty.md`), which suits winding the crank
  backwards through the history.
- **What carries over.** `core`, `content` and `ui` do not use macroquad
  (ADR-0004), screens build views that a skin paints (ADR-0054), and
  graphics are a skin (ADR-0038). What does not: `app` (macroquad, gilrs,
  quad-snd) is replaced by a Playdate front end; Playdate Rust has no
  standard library, so `core`, `content` and `ui` would have to build
  without it; the CPU is a 168 MHz ARM chip with 16 MB of memory, so the
  enemy AI's thinking time and the embedded data's size need measuring;
  music and voice need converting to formats the device plays.
- **Selling it.** A Playdate build can be sold on itch.io beside the
  Windows one (players sideload it); Panic's own Catalog store is curated
  and needs an application.

**What 1026 found (2026-10-15, measured on Nick's Playdate, Rev B).**
Full write-up: [ADR-0070](../../docs/adr/0070-playdate-front-end.md);
the code is on the never-merged branch
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate).

- **It runs.** `core`, `content` and `ui` build for the device with
  `no_std` + `alloc` behind a default `std` feature (+140, +318 and +401
  changed lines; their tests pass unchanged with `std`). The device loaded
  every data file and ran Chapter 1's enemy phases.
- **Tools:** stable Rust (no nightly) plus the SDK's own C build support,
  linked with MSYS2's `arm-none-eabi-gcc` (device) and MinGW `gcc`
  (Simulator `pdex.dll`), packed by `pdc`. No binding crate: a thin C
  shim of our own (boozook's `cargo-playdate` cannot read edition-2024
  manifests). SDK 3.1.2.
- **Speed:** the device is ~700× slower than the PC. Loading every data
  file takes **3.7 s**; a 14-enemy phase thinks for **1.2 s** in all (80 ms
  per enemy on average, **0.23 s** at worst). The Simulator runs at PC
  speed, so only device runs count.
- **Stack:** the game needs ~15 KB of stack; the device gives ~10 KB and
  the first build crashed. The front end runs the game on a stack it
  allocates (ADR-0070 §4).

What this adds to the build tickets below: a ticket for the `xtask`
Playdate build (from the spike's `build.sh`) and its CI job, which also
builds the three crates with `--no-default-features` for the device; the
own-stack entry in the front end's first ticket; loading only what 0061
keeps, behind a loading card, then measuring again (a binary data format
only if still too slow); and an optional AI speed-up ticket (plan once
per phase instead of per command, same choices) if the enemy pauses feel
long when Nick plays. The Playdate skin tickets wait for 1108's
`ConsoleSpec` (ADR-0070 §6). Each ticket with device-visible work names
how it is measured on Nick's Playdate.

## Nick input

`None.` His answers come from 0061.

## Scope

**In:** the build tickets (`write-ticket` skill) that take Chapter 1, as
0061 decided it, to a Playdate build that can be sold where 0061 says.
Each ticket is one PR's worth of work. Expect roughly:
- making `core`, `content` and `ui` build without `std` behind a default
  `std` feature, one ticket per crate, using 1026's findings (ADR-0070
  §3 lists every change; `spike/playdate` has a working version);
- the Playdate build: an `xtask` command and a CI job (stable Rust, the
  SDK's C build support, `pdc`; ADR-0070 §1);
- the Playdate front-end crate `crates/playdate` (beside `app`,
  ADR-0070 §4): the C shim, run loop, the game on its own stack, buttons
  and crank to `Action`s through a keymap (`keyboard-input` skill),
  saves;
- loading time: only the data the Playdate version uses, a loading card,
  measured on the device (ADR-0070 §5);
- the 1-bit font and the Playdate skins: the map scene and each Chapter 1
  screen's view painted at 400 × 240 (ADR-0038, ADR-0054), screen by
  screen, limited to what Chapter 1 uses;
- sound and music in formats the device plays (0061 keeps them), and the
  voices;
- one-bit portraits made from the bought art (check ADR-0040's license
  notes first) and a simple one-bit combat picture (0061 keeps both);
- the title cinematic on the Playdate (0061 keeps it);
- one build per language (English, Japanese, Chinese), not all languages
  in one build (0061);
- packaging and the store page: itch.io first, one purchase giving every
  language's build, then the Catalog application (0061; next to 0901).

**Out (do not do):** the build work itself; any change to the desktop or
web game's look or controls for the Playdate's sake; the engine carve.

## Implementation steps

1. Read 1026's ADR, `docs/design/playdate.md` (0061) and this ticket's
   *Context*.
2. Write the build tickets. Check each one's dependency chain both ways
   (`write-ticket`). For each ticket, say what happens if carve tickets
   1108/1110 land first, and add a matching line to 1108/1110 if they are
   still open.
3. Add the tickets to the Playdate bullet in `docs/ROADMAP.md`.
4. Run `cargo xtask ticket-lint`. Archive this ticket.

## Acceptance criteria

- [x] Build tickets exist for every part of 0061's answers and 1026's
      ADR, each with `blocked_by` filled in.
- [x] `docs/ROADMAP.md` lists them in order.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- None (planning).

## Completion notes

**Done.** 28 tickets (27 build tickets and one decision for Nick, 0062) take Chapter 1 to a Playdate build that is sold on
itch.io and then sent to Catalog. `docs/ROADMAP.md`'s Playdate bullet lists
them in order:

| Ticket | What |
| ------ | ---- |
| 1027, 1028, 1029 | `core`, `content`, `ui` build without `std` (one per crate), plus the `no-std` CI job |
| 1030 | The console size is passed in (`ConsoleSpec`): the part of carve ticket 1108 the skins need, done first |
| 1031 | `crates/playdate`: the C shim, allocator, the game on its own 64 KB stack, `cargo xtask playdate`, the CI device build, `THIRD_PARTY_ASSETS.md` rows |
| 1032 | The one-bit frame: 1-bit font, Playdate palette, renderer, `frame-png --playdate`, an ADR |
| 1033 | The front end runs the game: the loop, D-pad/A/B as a controller, saves on the SDK's files |
| 1034, 1035 | Button chords (A+B, hold B + left/right); the crank rewinds, the system menu, map menu entries |
| 1036 | Load only what the Playdate uses, a loading card, measured again. It writes the binary-format ticket only if loading still takes over 2 s |
| 1037 | The one-bit map skin, and choosing the Playdate's skins (`SkinSet`) |
| 1038, 1039 | The battle screen (full-screen map, corner pop-ups, menus); forecast, unit info, level-up, rewind |
| 0062 | Nick decides how the one-bit pictures look (portraits, combat picture, launcher card and icon) |
| 1040, 1041, 1042 | One-bit images at build time; the combat picture; one-bit portraits (private repository) |
| 1043, 1044 | The dialogue screen; the title screen and cinematic |
| 1045, 1046 | New Game and Options screens; preparations, results, saves, game over, credits |
| 1047, 1048 | Sound and music in device formats (an ADR); voices |
| 1049 | One build per language |
| 1050 | Nick plays Chapter 1 on his Playdate; his feedback becomes tickets |
| 1051 | Optional: the AI plans once per phase (only if the enemy pauses feel long) |
| 1052, 1053 | Package and sell on itch.io; prepare the Catalog application |

Every ticket with device work names what is measured on Nick's Playdate.

**Dependency chain, both ways:**
- The skins wait for the screens' views: 1038 and 1039 for 0242, 1043 for
  0242's split-off (c), 1045 for 0247. 0242 now says to add its two
  split-offs' ids to 1039's and 1043's `blocked_by` when it writes them.
- 1048 waits for 0722 (the voices). 1049 waits for 0719 and 0726 (the
  translations; the wide fonts come through them). 1052 waits for 0901
  (itch.io publishing).
- 1108 is now blocked by 1030, and skips its step 1 when it runs. 1108
  and 1110 say what to do with the Playdate code. 1103–1107, 1109,
  1111–1117 and 1131 got a **Playdate** line (ADR-0070 §6: carved crates
  keep the `std` feature and the `no-std` build). Each Playdate ticket
  says what happens if the carve ticket it touches lands first.
- 0812, 0813 and 0837 say to add a Playdate paint if 1044 or 1045 is
  already done. 1049 and 1052 do the same for the language channels.

**Deviations.**
- **A new decision ticket, 0062.** 0061 kept "one-bit portraits" and "a
  simple one-bit combat picture", but Nick only saw mockups of the map.
  How those pictures look, and the launcher card and icon every Playdate
  game needs, are look-and-feel questions for him, so they get a
  decision ticket rather than a guess.
- **1030 does part of 1108 early.** ADR-0070 §6 allowed it ("or do that
  part of 1108 first"). The carve chain to 1108 is long, and Nick asked
  to rush the port.
- **The binary data format** has no ticket yet. 1036 writes one only if
  its device measurement says it is needed. The AI speed-up (1051) does
  exist, but 1050 closes it as not needed if Nick finds the pauses fine.

**the agent's starting rules in the new tickets** (Nick can veto; the
build tickets mark them):
- The loading card shows the game's name, centred (1036).
- The Playdate's battle map has no help bar. The map menu and the
  buttons printed on the Playdate stand in for it (1038).
- The corner pop-ups move to the bottom when the cursor is in the top 5
  rows (1038; *tunable*, from 0061's "top corners, move down when the
  cursor comes near").
- The unit's own things go on Unit info. The chapter, turn and rewinds
  go in the map menu's header (1039, splitting 0061's "on the Unit info
  screen and in the map menu").
- The Playdate's title menu has no Quit entry; the system menu does that
  (1044).
- The dialogue face sits beside the text box as on the PC, or above it
  if it doesn't fit (1043).
- Technical, tunable: the A+B chord window starts at 80 ms (1034), and a
  crank notch is 30° (1035).

**For Nick:** the only new question for you is 0062 (how the one-bit
faces, combat picture and launcher card look). Everything else follows
your 0061 answers. The fastest path to playing Chapter 1 on your
Playdate is rows 1–8 of the ROADMAP list, then 1050.
