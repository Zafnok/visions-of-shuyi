---
id: "1184"
title: "The engine's one-bit look replaces the game's one-bit skins of the engine's screens and widgets"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: blocked
blocked_by: ["engine#2371", "1077"]
nick_input: none
completed:
---

# 1184 — The engine's one-bit look replaces the game's copies

## Context

From the retro of the last 50 PRs (2026-10-22). ADR-0091 chose "the
engine's screens painted by the game's" for the Playdate: the engine's
Options, Key bindings and Profiles screens, its name entry, question
and menu widgets, each painted in one bit by a skin the game wrote
(1045, #344; 1164 moved Profiles and name entry into the engine but kept
their one-bit skins here). That is about **617 production lines that
name no game concept**: `screens/playdate.rs` (the shared window, menu
and question painters, 251), `key_bindings/playdate.rs` (155),
`profile/playdate.rs` (80), `options/playdate.rs` (74),
`widgets/name_entry/playdate.rs` (57). The battle's one-bit menu painter
(`paint_menu` in `screens/battle/playdate.rs`, 1038) copies the engine's
menu skin too, because that skin hard-codes its border style and colour
roles.

The first friction sweep's Playdate row said "revisit with engine 3001
and 3010"; 3001 closed on 2026-10-20 and nothing followed. engine#2371
gives the engine's widget and screen skins a look and ships a one-bit
look for them. This ticket deletes the game's copies.

## Nick input

None. The Playdate's screens must look as they do now (the look is
Nick's, from 1045's mockups, and the one-row scroll rule from 1077);
verified by the one-bit snapshots, which must not change. If engine#2371's
look can't match a detail, keep that skin and say which in the
Completion notes.

## Scope

**In:**
- Move the pin past engine#2371 (`engine-change` skill, §7).
- The Playdate build picks the engine's one-bit look for the engine's
  screens (Options, Key bindings, Profiles, the layout picker if the
  Playdate shows it) and widgets (menu, question, name entry), instead
  of the game's skins.
- The battle's one-bit menus painted by the engine's menu skin in the
  one-bit look, if it covers the battle menu's note line and scroll
  marks; else keep `paint_menu` and write why.
- The game's one-bit files above, emptied of what moved; whatever
  names a game concept (the lead select's and mode select's one-bit
  skins, the map skin) stays.

**Out (do not do):**
- The PC look.
- The battle's own one-bit screens (map, forecast, unit info: game).

## Implementation steps

1. **Read** engine#2371's API and docs, ADR-0091, the files above, the
   `Skins` choice by target (`crates/ui/src/screen.rs` or where the game
   picks a skin per target), and the one-bit snapshots
   (`crates/ui/tests/it/playdate.rs` and the `snapshots/` beside the
   one-bit skins).
2. **Pin** past engine#2371.
3. One screen at a time, snapshots unchanged after each: Options, Key
   bindings, Profiles and name entry, then the shared window, menu and
   question painters, then the battle's menus.
4. **Measure** with `cargo xtask code-lines`: expect `trpg-ui` down by
   about 500 production lines; add the friction row if the added lines
   pass 2× the estimate (about 60).

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2371.
- [ ] Every one-bit snapshot is unchanged (or each change is listed in
      the PR and is a detail engine#2371 recorded as decided).
- [ ] `cargo xtask code-lines` shows `trpg-ui` production lines down by
      at least 400.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine` and the CI `playdate` job.

## Tests required

- Snapshot: the one-bit snapshots, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
