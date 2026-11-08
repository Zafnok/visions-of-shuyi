---
id: "0481"
title: "Decide and build: the objective shown when a battle starts, with how its parts combine"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0481 — The objective shown when a battle starts

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "The battles should explain the objective without needing to go to the
> objective in the options menu (i.e. a brief popup banner at the start
> of battle so ch2 isn't a surprise)"

> "The end condition for ch2 doesn't make much sense, it seems you need
> to both clear houses as well as kill the boss, it seemed from our
> workshopping it would be one or the other but it seems to be both.
> Make it more clear once the banner displays (AND or OR condition
> properly highlighted)"

Today:

- The objective is read only from the map menu → Objective
  (`MapEntry::Objective`, `objective_text()`,
  `crates/ui/src/screens/battle/map_menu.rs` ~:31, ~:225; `Mode::Objective`
  builds `DialogView::Objective(ObjectiveView { objective, village, turn,
  notes })`, `battle/mod.rs` ~:2539; painted at `glyph.rs` ~:369).
- The only banners are the phase banner (`PLAYER PHASE`, `Turn 2/8`, 1 s,
  `PHASE_BANNER_S`) and the outcome banners (`battle/banner.rs`,
  `BannerView`, `view.rs` ~:238).
- `Objective` (`crates/core/src/battle.rs` ~:550) is one of Rout,
  DefeatUnit, Seize, Survive, Escape, with an optional `turn_limit`; each
  battle sets it in `assets/battles/*.ron`.

**Chapter 2 is AND, and that is the design.** `assets/battles/ch02.ron`
~:67: `objective: DefeatUnit(unit: "hound_lieutenant", leftovers: Flee)`,
~:70 `village: (minimum: 3, …)`. The village is a *loss* condition:
`village_failed()` (`battle/village.rs` ~:202) loses the battle the moment
too few houses can still stand. Killing Hendry wins; houses still standing
then count as saved. Evacuating is never required. This is what Nick
decided in 0069 (`docs/design/turn-structure.md`, *Save the village*:
"it should be a real race not an optional one... save the village with a
minimum number houses allowed to pass") and what the beat sheet says
(`docs/story/chapters/ch02.md` ~:11: "defeat the lieutenant and save at
least a minimum of the houses"). The "one or the other" from the
workshop is recorded nowhere.

What misled the player: the Objective page shows two lines,
`Defeat {name}` and `Save at least {minimum} of {total} houses ({saved}
saved, {lost} lost)` (`assets/lang/en/ui.ron` ~:569, ~:575; built in
`battle/loot.rs` ~:37), with nothing joining them, and "save" reads like a
task to finish ("clear the houses") rather than "don't let too many
burn".

## Nick input

**Decision** (`ask-nick`, before building), on `frame-png` pictures of
Chapter 1's and Chapter 2's start, both looks:

1. **Chapter 2's rule:** keep it as decided in 0069 (win by defeating
   Hendry; lose if too many houses burn), or change it to either one
   wins? Quote his 0069 answer. If he changes it, that is a change to
   `turn-structure.md` and `ch02.ron` and the core's win check: do it in
   this ticket only if it stays small (an `any of` objective in core);
   otherwise record it and write a ticket.
2. **The banner's look**, options:
   - A. The phase banner's look, one line per part, joined by a
     highlighted **AND** / **OR**, up for a few seconds, then the first
     `PLAYER PHASE`.
   - B. Two labelled lines, **Win:** and **Lose:** (`Win: Defeat Hendry` /
     `Lose: 3 houses burn, or the lead falls`), as many tactics games
     list victory and defeat conditions at a battle's start (check which
     ones before naming them).
   - C. A panel that stays until a press (Confirm), not timed.
   - D. Combinations, or describe your own.

   And the same wording on the map menu's Objective page (yes / no).

## Scope

**In:**
- The question and the answers, recorded in `turn-structure.md`
  (objectives) and `look-and-feel.md` (the banner).
- An objective banner (a new `BannerView` kind, or a dialog, as picked)
  shown once when a battle starts, after the pre-battle talk and before
  the first phase banner; its text built from the battle's objective,
  village and turn limit by the same code as the Objective page (one
  function, so they never disagree).
- The join word or the Win/Lose labels, highlighted in a colour from the
  theme; the village line worded as a condition the player can fail.
- The Objective page uses the same wording if Nick says yes.
- Every skin of the banner: glyph, picture look, Playdate (it has phase
  banners; keep its fit rule).
- Rewind and Retry: the banner shows on a fresh start and a Retry, not
  after a rewind (*the agent's starting rule*, on 1064's checklist).

**Out (do not do):**
- A chapter title card (0812). Either order: if 0812 is done, the
  objective banner comes after its card; if not, 0812 places its card
  before this banner.
- The Seize line naming its place (0331) and the escape line naming its
  exit (0324): this banner uses their text as it is.
- Vix's lessons (0475, 0476).

## Implementation steps

1. Read `banner.rs`, `map_menu.rs` (`objective_text`), `battle/loot.rs`,
   `turn-structure.md` (*Objectives*, *Save the village*), 0069's done
   ticket.
2. Render the options with `cargo shuyi frame-png` for ch01 and ch02;
   ask Nick both questions; record the answers.
3. One function builds the objective's lines (with the join or labels);
   the banner and the Objective page call it.
4. The banner: a view kind, its timing (or its dismiss) in the battle's
   logic, painted by each skin.
5. If Nick changed Chapter 2's rule and it is small: the core change,
   `ch02.ron`, and the docs; otherwise a ticket.
6. Tests and snapshots.

## Acceptance criteria

- [ ] Nick's answers are recorded in `turn-structure.md` and
      `look-and-feel.md`.
- [ ] Starting Chapter 1 shows the objective banner once before the
      first phase banner (Harness test reading the view).
- [ ] Chapter 2's banner shows both parts with the join he picked
      highlighted, and the village part reads as a condition to fail
      (test on the built lines; `frame-png` picture in the PR).
- [ ] A rewind doesn't show it again; a Retry does (test).
- [ ] The Objective page and the banner build their lines with the same
      function (one test on both).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the objective lines for each objective kind, with and without a
  village and a turn limit.
- Integration: the Harness test at battle start.
- Snapshot: the banner for ch01 and ch02, glyph and Playdate.

## Completion notes

*(Filled in by the session that completes the ticket.)*
