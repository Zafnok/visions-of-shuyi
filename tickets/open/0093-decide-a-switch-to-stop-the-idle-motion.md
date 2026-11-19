---
id: "0093"
title: "Decide: a switch to stop the looping motion (campfire, embers, unit steps, pulses)"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0093 — Decide: a switch to stop the looping motion (campfire, embers, unit steps, pulses)

## Context

The test audit before the writer trial (ticket 0766) found that the
design notes promise a setting that doesn't exist. `docs/design/look-and-feel.md`
(*Title screen*, the fire, around line 1609) says:

> With idle motion off (Options) it stands still on its first frame and
> no embers rise *(Claude's starting rule)*.

Ticket 0922's completion notes say the same thing. But the Options screen has no such row.
`crates/ui/src/screens/options.rs`, `Row::ALL` (27 rows: speeds, cursor,
map look and zoom, threat arrows, range style, intro, towns, theme,
volumes, voices, auto-advance, layout, key bindings, mode, tutorial, reset
tips, restore defaults), has no motion row, and `docs/design/options.md` has no
motion setting either. 0830's completion notes already said: "`options.md` has no
"Reduce motion" or animation-speed setting".

What exists is a technical switch. The engine's `Ctx::idle_motion`
(`IdleMotion::On`/`Off`) defaults to `Off`, and the engine's platform runner
always sets it to `On` (shuyi-engine `crates/platform/src/app.rs`). The
Playdate does too (`crates/playdate/src/start.rs`). `Off` is only used so
that test pictures hold still (`crates/ui/src/screen.rs`, `loop_clock_s`,
ticket 1290). With it off, these hold still today:

- the title's campfire (on its first frame) and its embers, and the intro
  cinematic's embers (`screens/title.rs`, `cinematic.rs`);
- the battle units' idle motion and the map scene's clock
  (`screens/battle/mod.rs`, `clock_ms`, `notes_t`);
- the battle cursor's pulse (held at full brightness);
- the forecast's blinking lost-HP bar (`battle/glyph/forecast.rs`,
  `lost_shaded`; decided 2026-11-16);
- the world map's motion after a move (`screens/world_map.rs`).

With it off, a selected unit's ranges also show at once instead of after
the delay 0923 added.

So in the shipped game and the writer demo, a player can't stop any of
this motion. The design note's starting rule assumed a player switch, and
whether one exists is a look and feel choice, so it is Nick's. Not a
blocker for the writer trial (0766): the writers shouldn't look for the
option.

## Nick input

**Decision** (`ask-nick`, asked before anything is built). The question:
*Should a player be able to stop the game's looping decoration (the
title's campfire and embers, units stepping in place on the map, the
cursor's pulse, the forecast's blinking HP bar)?*

- **A. No switch; it always moves.** This is how it works today. Like the
  Fire Emblem games on the Game Boy Advance, where map units always move
  in place and the Options have no way to stop it. The design note is
  corrected to say so.
- **B. An Options row, "Idle motion: On / Off"** (or another name Nick
  picks), starting On. Like the "Reduce Motion" setting on iPhone, iPad
  and Mac, or Celeste's "Screen Shake" On/Off. With it Off, everything in
  the list above holds still. Follow-up question: should the blinking HP
  bar and the cursor pulse stop too, or only the decoration (fire, embers,
  unit steps)? They carry information, so a still version may be needed.
- **C. Option B, plus starting Off when the computer or browser is set to
  reduce motion** (the web's "reduced motion" preference, Windows'
  "Animation effects" off). The agent checks whether each platform can
  tell the game this before offering C. If a platform can't, it says so.
- **D. Describe your own.**

Show it with `cargo shuyi frame-png` pictures of the title and a battle
frame at two moments with motion on, plus the still frame with it off.
Crop the battle frame to the panel and a few tiles (ADR-0118): stage it
in a throwaway `zz_` battle file, take `frame-png` at two clock times,
and revert it before the PR. Never a check after merge: whatever is built
from his answer is verified by tests and pictures.

## Scope

**In:**
- The question above with its pictures (throwaway, never committed).
- Nick's words and picks recorded in `docs/design/options.md` (a new
  section on motion) and in `docs/design/look-and-feel.md` (*Title screen*):
  the "(Options)" claim is replaced by what he decided.
- The `docs/design/README.md` rows for the Options screen and the title
  screen naming this ticket and its result.
- If he picks B, C or D with a player-facing switch: a build ticket in the
  `08xx` block (Options), `blocked_by` this ticket. It names the row, its
  text key, its default, which motions it stops, its place in
  `Row::ALL` and the Playdate's row list (`PC_ONLY`/`PLAYDATE_ONLY` in
  `options.rs`), and the setting's home (`ShuyiSettings`, saved with the
  player's profile, `options.md`). If Nick wants it before the Act 1
  playtest, add it to 1064's `blocked_by` and to *Act 1 critical path*
  in `docs/ROADMAP.md`.

**Out (do not do):**
- Building the row or changing any code.
- Changing how tests use `IdleMotion::Off` (1290).
- Editing done tickets (0922's notes stay as written).
- Any change to the engine. If the build ticket needs one (for example, the
  runner reading a setting instead of always setting On), the build
  ticket says so under `engine-change`, not this one.
- The writer packet (0766).

## Implementation steps

1. Read `docs/design/options.md`, `look-and-feel.md` (*Title screen*,
   the fire and the embers; the forecast's blinking bar), 0922's and
   0830's completion notes, `crates/ui/src/screen.rs` (`loop_clock_s`)
   and the uses of `ctx.idle_motion` (`git grep idle_motion crates`).
2. For C, check whether the web build and Windows can tell the game that
   reduced motion is wanted. Note the answer for the question; don't build
   anything.
3. Render the pictures (a throwaway `zz_` battle file and a `frame-png`
   at two clock times with idle motion On, one with it Off), then revert.
4. Ask Nick with `ask-nick` (options A–D above, and B's follow-up if he
   picks B or C). Expect a second round if he names a different set of
   motions.
5. Record his words and the rule in `options.md` and `look-and-feel.md`,
   update `docs/design/README.md`, and write the build ticket if one is
   needed.

## Acceptance criteria

- [ ] `docs/design/look-and-feel.md` no longer says "With idle motion off
      (Options)" unless Nick decided that such a row exists. It states
      what he decided, in his words or marked as Claude's starting rule.
- [ ] `docs/design/options.md` has a section on motion with Nick's answer
      quoted.
- [ ] The `docs/design/README.md` rows for the Options screen and the title
      screen name this ticket.
- [ ] If Nick picked a player switch, a build ticket exists in
      `tickets/open/` (`08xx`) with this ticket in its `blocked_by`.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`. The build ticket, if any,
names its own tests: a unit test that the setting maps to
`Ctx::idle_motion`, a test that the title's view has no embers and frame 0
with it Off, and snapshots of the Options list with the new row on PC and
Playdate.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
