---
name: engine-friction
description: Judge whether game code is far bigger than the job, and if so whether the fix is an engine feature, engine ergonomics, a game-side structure, or nothing (ADR-0092). Use when a ticket's measured production lines reach 3× its estimate, when you write a workaround for an engine assumption, when the same boilerplate appears a third time, when a performance budget is missed because of the engine, or when the friction log shows three "accept" rows on one engine area.
---

# Engine friction review

The rule is ADR-0092; read it once. The engine and the game are both
ours: a missing engine piece is normal work, never a reason to wait.
The numbers only decide *when to look*. What to do is a judgment, and
this skill makes it the same way every time.

`work-ticket` already asks for steps 1 and 2 on every ticket. The rest
runs only when they trigger it.

## Steps

### 1. Write the estimate (before the first code)

One sentence and a number: how many production lines the change
*should* take with a good engine, and the reference you compared it
with.

- Start from the ticket: a ticket that adds a screen or a scene states
  its expected size in its Implementation steps.
- References: the same job in another engine or framework (Ren'Py's
  `say` for a dialogue line, an egui window, a Godot scene), or a piece
  of this repository that does something similar (the engine's
  `crates/ui/src/screens/options.rs` for a screen with a view and
  a skin, ADR-0054).
- Count production code only: no tests, snapshots, data files or docs.

Example: "a yes/no pop-up is one view and one skin, a fraction of the
options screen: 40 lines."

Write it in your working notes before coding; it goes in the PR body
later. Don't revise it after measuring: a wrong estimate is a finding
too (outcome 4).

### 2. Measure

```bash
cargo xtask code-lines
```

It compares the working tree with its merge base with `origin/main`
(`--base <ref>` for another). The measure is the **added** column of
`total (game)`: a changed line counts as one removed and one added. If
you changed the engine in the same PR (fix first, before 1131), its lines
are in `total (engine)` and are not part of the game's ratio.

Ratio = measured ÷ estimate:

| Ratio | What to do |
| ----- | ---------- |
| under 2× | nothing; the PR line says "under 2×" |
| 2× to 3× | one line in the Completion notes ("friction: 2.4×, accepted because …") and a row in the log (step 7) |
| 3× or more | this review, steps 3–7, before opening the PR |
| 10× or more | this review, and fix first unless the review says why not |

### 3. Check the signals

Any of these triggers the review at any ratio:

- **A workaround for an engine assumption**: a `cfg` to dodge engine
  behavior, engine code copied into the game, reaching into an engine
  type's internals, fighting a trait to make it fit.
- **The third copy of the same boilerplate** across screens. Search for
  it: `grep -rn "<a distinctive line you just wrote>" crates/`.
- **A performance budget missed** because of how the engine does
  something (the Playdate is about 700× slower than a PC; ADR-0070).
- **Three or more rows in `docs/engine/friction.md` on the same engine
  area with outcome "accept"**.

No trigger: stop here.

### 4. Find where the extra code went

1. Name the job in one sentence, as a player or a reader of the code
   would ("show one line of dialogue with the speaker's face").
2. Read the diff and sort its lines into two piles:
   - **the game's real logic**: rules, content, what this game says
     and decides;
   - **glue**: lines that exist only to get around the engine (setup
     the engine could default, state the engine should keep, copies,
     conversions, workarounds).
3. Each cluster of glue is one **finding**. Most reviews have one to
   three.

### 5. Pick one outcome per finding, then the leak test

1. **Engine feature**: a generic capability is missing. It passes the
   ADR-0063 §5 test: *would a platformer or a farming game want this
   unchanged?* → engine; *would another grid-tactics game?* → tactics
   kit.
2. **Engine ergonomics or optimization**: the capability exists but
   costs too much code or time to use (a missing builder, a missing
   default, a slow path).
3. **Game-side structure**: the game needs its own data structure,
   helper or reorganization. This is the answer whenever the fix would
   name a game concept.
4. **Accept**: the job really is that big. Say why. If the estimate
   was the problem, say what you'll estimate differently next time.

**Leak test** for outcomes 1 and 2: the engine change must not name a
class, unit, faction, Combat Art, chapter, character, story beat, or any
id from the game's data. If the only shape that helps is game-shaped, it
is outcome 3. If a generic shape sits under the game-shaped one, the
engine gets the generic shape and the game wraps it ("dialogue box" is a
game idea; "a text panel with a portrait slot and paged text" is
generic).

### 6. Fix first, or ship and follow up

Never wait. For each outcome 1–3 finding, exactly one of:

- **Fix first**: the fix is one PR's worth or less and cuts the game
  change sharply. Do it now, in this session, then finish the game
  ticket on it.
  - The engine has its own repository (since ticket 1131): the engine
    change is an engine PR first (made in the local checkout, `cargo
    xtask engine`), merged with a merge commit, then this ticket's PR
    moves the pin (`cargo xtask engine-pin`; `docs/engine/README.md`).
    1152 turns this into the `engine-change` skill.
  - A game-side structure (outcome 3) that is small goes in this PR.
- **Ship and follow up**: otherwise. The game ticket lands with the
  workaround, each workaround marked with a comment
  `// engine-friction: <ticket id>`, and you write two tickets
  (`write-ticket`):
  1. the engine or game-structure ticket (its generic shape, the leak
     test passed, the friction numbers in its Context);
  2. a game ticket that deletes the workaround, `blocked_by` the first.

At 10× or more, fix first is the default; the review must say why not
(for example: the fix is several PRs' worth).

### 7. Add a row to the log

Add one row to `docs/engine/friction.md` for every review and every
2×–3× note, newest last:

| Date | Ticket | The job | Estimate | Measured | Ratio | Outcome | Follow-up |
| ---- | ------ | ------- | -------- | -------- | ----- | ------- | --------- |
| 2026-10-18 | NNNN | one sentence | 40 | 130 | 3.3× | 3: game-side builder, fixed first | — |

Outcome is the number and a few words; several findings go in one cell,
separated by `;`. Follow-up names the tickets written in step 6, or `—`.

Then put the line in the PR body: "Engine friction: estimate N, measured
M (R×), <outcome>".

## Worked example 1: a dialogue box that needed hacks

*(Written from the ADR to show the steps; not from real code.)*

- **Estimate**: "show a line of dialogue with the speaker's face and
  page long text; Ren'Py does it with one `say` statement plus a screen
  style: 60 lines."
- **Measured**: 420 added in `crates/ui`. Ratio 7×: review. Signals
  too: the text-wrapping code was copied out of an engine widget, and
  the game reads the widget's private scroll offset through a helper
  that reaches into it.
- **Job**: "show one line of dialogue in a box with the speaker's
  portrait, a page at a time."
- **Where it went**: about 90 lines are real logic (which face, from the
  speaker and their mood; the name plate; when to wait for the player).
  About 330 are glue: wrapping and paging text inside a fixed panel,
  keeping a portrait slot beside it, and tracking the page.
- **Findings**:
  1. Paging text in a panel with a picture slot is missing from the
     engine. A farming game's shopkeeper talks the same way: **outcome 1,
     engine feature**. Leak test: the game-shaped idea is "dialogue box";
     the generic shape under it is "a text panel with a portrait slot and
     paged text". The engine gets that, named so, and knows nothing about
     speakers or moods.
  2. Picking the face from the speaker and mood, and the name plate,
     name game concepts: **outcome 3, game-side structure**, a small
     dialogue box builder in the game over the new panel.
- **Decision**: the panel is one PR's worth and cuts the game change to
  about 90 lines, so **fix first**: the panel goes into the engine's
  `crates/ui` in an engine PR, then this PR moves the pin, puts the
  builder into the game, and deletes the copied wrapping code.
- **Log row**: `| 2026-10-18 | NNNN | one line of dialogue in a box with a
  portrait, paged | 60 | 420 | 7× | 1: text panel with portrait slot and
  paging (engine); 3: game dialogue box builder; fixed first | — |`

## Worked example 2: a scene at 700 lines

*(Written from the ADR to show the steps; not from real code.)*

- **Estimate**: "a scene of a camp at night with a dozen props and two
  characters walking in; a Godot scene file plus a short script: 30
  lines."
- **Measured**: 700. Ratio about 23×: review, and fix first is the
  default.
- **Job**: "lay out a scene and walk two characters into it."
- **Where it went**: about 60 lines of real logic (which props, where
  the characters stop, the order of the walks). About 640 lines of glue:
  each prop built by hand with the same layer, position, anchor and tint
  setup; every walk written as frame-by-frame position updates because
  the scene has no "move this to there over time" call.
- **Findings**: the scene can do all of it, but each use costs ten
  lines of setup. **Outcome 2, engine ergonomics**: a scene builder
  (place a prop with defaults, chain overrides) and a tween helper for
  movement. Leak test: neither names a character or a chapter; a
  platformer's cutscene wants both unchanged.
- **Decision**: the builder is one PR; the tween helper changes the
  scene's update loop that a dozen screens use, so it is several PRs'
  worth. That is the "why not" for the tween at 23×. The builder is
  **fixed first** (in this PR, before 1131). The walks **ship and follow
  up**: each frame-by-frame walk marked `// engine-friction: MMMM`, an
  engine ticket MMMM for the tween helper, and a game ticket that
  rewrites the walks on it, blocked by MMMM.
- **Log row**: `| 2026-10-18 | NNNN | lay out a scene and walk two
  characters in | 30 | 700 | 23× | 2: scene builder, fixed first; 2:
  tween helper, follow-up | MMMM, PPPP |`

## Don'ts

- Don't let a finding block the game ticket. Fix first or ship and
  follow up; nothing else.
- Don't put a game concept in the engine to make a review come out
  "engine". Run the leak test.
- Don't refactor other code you find while reviewing. Write a ticket.
- Don't add a CI gate on the ratio, and don't adjust the estimate after
  measuring.
