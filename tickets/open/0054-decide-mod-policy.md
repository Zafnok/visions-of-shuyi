---
id: "0054"
title: "Decide: the mod policy — what mods may change, modded saves, Workshop, the licence"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0804"]
nick_input: decision
completed:
---

# 0054 — Decide: the mod policy

## Context

Nick, 2026-10-04 and 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Modding and dev tools*): yes to modding and to "a bunch of dev tools",
"a scripting DSL or something for easy modding support". How it's built is
technical (tickets 1011–1024). What players and modders are **allowed** to
do is Nick's call, because part of it is the licence, and `LICENSE` is
his alone (ADR-0013).

What already exists: `LICENSE` section 4 lets anyone make and share mods
for free if the mod needs a legitimate copy of the game, doesn't include
the game's executables or "substantial portions" of it, isn't sold, is
labelled unofficial, and is taken down on request. The bought art (Tiny
Tales and other packs, ADR-0032, ADR-0040) has its own licences; a mod that
**uses** the game's art by name inside the game doesn't redistribute it,
but a tool that **exported** it would.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own". Say plainly where an
answer touches the licence, and that a lawyer's look at `LICENSE` before
the Steam launch is still recommended (ADR-0013).

## Scope

**In:** the questions below; the answers in `replayability.md` (*Modding
and dev tools*); the `docs/design/README.md` row; any change to the
technical tickets 1011–1024 that the answers call for.

**Out (do not do):** editing `LICENSE` (if Nick wants a change, he makes
it or asks for it in his own words, and that is its own PR); any code.

## Implementation steps

1. Read `LICENSE` (section 4), ADR-0013, ADR-0032, ADR-0040, ADR-0051 and
   the licences of the bought packs in the private repository's credits
   (`assets/data/credits.ron` names them). Note for each pack whether it
   allows the game's own art to be **used** by mods inside the game (it
   isn't copied anywhere) and whether it forbids anything a mod tool might
   do (extracting, re-saving, editing a sprite).
2. **Ask Nick**, in this order:
   1. **What mods may change.** Options: anything, including the story and
      whole new campaigns ("total conversions", as Fire Emblem's ROM-hack
      scene and Battle for Wesnoth's add-ons do); new battles, units and
      items only, the story untouched; balance and data only.
   2. **Mods and the game's own art.** Options: mods may use any of the
      game's pictures and sounds by name, but never ship copies (the tools
      refuse to export bought art); mods bring all their own art. Report
      what step 1 found in the packs' licences.
   3. **Modded saves.** Options: modded saves are marked and kept apart
      (Stardew Valley with SMAPI, XCOM 2); marked only; no difference. And
      whether stars, the veteran's ghost and later leaderboards count with
      mods on (Steam games usually turn achievements off with mods).
   4. **Steam Workshop.** Wanted at launch, later, or never. Workshop's own
      terms say what Valve may do with uploads; they don't conflict with
      `LICENSE` section 4 as far as Claude can tell, but a lawyer should
      confirm.
   5. **Mods on the web build.** Options: native builds only (Windows,
      later Linux and macOS); the web build too, with a mod loaded from a
      zip file the player picks.
   6. **Credit and featuring.** Whether the game ever lists or recommends
      mods (an official "featured mods" list, a credits line), and whether
      Nick wants the right to include a mod's ideas (already in `LICENSE`).
   7. **The licence itself.** Show section 4 as it stands. Does Nick want
      anything changed (for example, allowing paid mods, or the
      "substantial portions" wording now that the data files are the main
      modding surface)? If yes: note his words; he changes `LICENSE`
      himself or asks for a PR.
3. Record each answer verbatim and as a rule.
4. Update the technical tickets where an answer changes them: 1011's ADR
   (what mods may override), 1015 (marking modded saves), 1022 (mod
   campaigns: drop it if story changes aren't allowed), 1024 (Workshop:
   drop it or keep it), and the web-build line in each.

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The bought packs' licences were read and what they allow for mods is
      written down with a source for each.
- [ ] 1011–1024 match the answers; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
