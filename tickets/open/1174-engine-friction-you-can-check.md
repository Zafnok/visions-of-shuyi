---
id: "1174"
title: "Engine friction you can check: an area column, a trigger check in the gates, and net lines for refactors"
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1174 — Engine friction you can check

## Context

ADR-0092 and the `engine-friction` skill turn engine friction into work
in two ways: a review on a ticket (at 3× or more, or on a signal) and a
**sweep** of the whole game (1153 was the first). A sweep starts when

1. a milestone in `docs/ROADMAP.md` closes (the last open ticket of its
   block moves to `tickets/done/`), or
2. `docs/engine/friction.md` has three rows on the same engine area,
   since the last sweep, with outcome "accept" or a ratio from 2× to 3×.

"The session that sees the trigger" writes the next sweep ticket. Nick
asked on 2026-10-20 whether there was a long-term plan for this. There
isn't one yet, and three gaps make the rule depend on luck:

- **The three-rows trigger can't be checked.** The log has no area
  column. The area is a judgment made when the row is written, and
  nothing counts rows per area since the last sweep. The rows since 1153
  (1159 screen stack 2.1× accept, 1160 list cursor 2.7× accept, 1161
  game-side 3.1×, 1162 menu 10.4×, 1163 question 5.1×, 1165 drawing
  3.9×, 1167 grid skin 5.2×) touch different areas, so no sweep
  triggered. If they had touched the same area, nobody would have
  noticed.
- **The milestone trigger relies on a session noticing.** The `11xx`
  block closes after the remaining carve tickets (1166, 1168–1173 and
  this one), and nothing schedules the second sweep for that moment.
- **Refactors log false overruns.** `cargo xtask code-lines` counts a
  changed line as one removed and one added, and the ratio uses the
  *added* column. 1160, 1161, 1162, 1163, 1165 and 1167 were refactors
  whose production lines **fell**, yet four of them measured 3.1× to
  10.4× and needed "accept" reviews. Their notes repeat the same lesson
  ("count the lines a move rewrites") without changing the rule. Once
  the area trigger is counted, this noise would start sweeps.

This ticket closes all three: an `Area` and a `Net` column in the log,
a `cargo xtask friction-check` in the gates that fails when a sweep is
due and none is filed, a net column in `code-lines` with a refactor
rule, and the second sweep written now as a ticket that waits for the
`11xx` block.

## Nick input

None.

## Scope

**In:**
- `docs/engine/friction.md`: an `Area` column and a `Net` column,
  a fixed list of areas, every existing row backfilled (step 1).
- `cargo xtask code-lines`: a `net` column (added − removed) per crate
  and in the totals (step 2).
- The refactor rule in the `engine-friction` skill (step 3).
- `cargo xtask friction-check` (new `crates/xtask/src/friction_check.rs`),
  run by the `run-gates` skill and CI beside ticket-lint (step 4).
- A short amendment to ADR-0092 saying why this gate is allowed when a
  gate on the ratio is not (step 5).
- The second sweep ticket, written now and blocked by the rest of the
  `11xx` block, plus its line in `docs/ROADMAP.md` (step 6).

**Out (do not do):**
- A gate on any ratio, estimate or line count (ADR-0092 rejects one;
  `friction-check` reads the log's outcomes and the ticket folders, not
  the diff).
- `code-lines --tree` (ticket 1172). Either order works: both edit
  `crates/xtask/src/code_lines.rs`, and whichever lands second rebases
  over the other. If 1172 is done first, its tree mode has no diff and
  gets no net column.
- Running a sweep. The sweep ticket written in step 6 runs it.
- Changing what counts as a production line.

## Implementation steps

Expected size: about 250 production lines (`friction_check.rs` about
200: a Markdown table reader, two checks and the messages; `code_lines.rs`
about 15; `main.rs` about 10), plus tests and docs. A refactor rule
applies to none of it: it is new code.

### 1. The `Area` and `Net` columns

In `docs/engine/friction.md`:

1. Add a short section **Areas** above the table with the fixed list.
   One row's area is the engine module its job leaned on, or `game` when
   the work was game-side only, or `tooling` for the repository's tools:

   | Area | What it covers |
   | ---- | -------------- |
   | `screen stack` | screens, push and pop, results, `covered`/`uncovered` |
   | `list cursor` | the list cursor and scroll windows |
   | `menu` | menus and menu items |
   | `question` | the question widget |
   | `text box` | the dialogue text box, paging, typed text |
   | `drawing` | panels, meters, centred and cut text, `px_rect` |
   | `grid skin` | the tactics kit's grid scene and skins |
   | `options` | the options screen and its rows |
   | `screens` | the engine's ready-made screens (options, key bindings, profiles, name entry) |
   | `bundle` | the asset bundle, loaders, formats, word wrap in assets |
   | `cinematic` | the cinematic timeline and shots |
   | `frame time` | frame time, clocks, skippable timers |
   | `platform` | the platform shell and backends (desktop, web, Playdate) |
   | `audio` | the audio player |
   | `game` | game-side work with no engine area |
   | `tooling` | xtask, `shuyi-cli`, the gates |

   A new area is added to this list in the PR that first needs it,
   never invented in a row.
2. Add `Area` after `The job` and `Net` after `Measured`. A row may name
   several areas separated by `;`, like the Outcome cell; each counts.
   `Net` is game production lines added − removed from
   `cargo xtask code-lines` (`+67`, `−151`), or `—` for a sweep row
   (a sweep measures the tree, not a diff).
3. Backfill every row. Use this table; check the net numbers against
   each row's own note (or by running `cargo xtask code-lines --base
   <the PR's merge base>` on the merged PR's commits) and fix any that
   disagree:

   | Row | Area | Net |
   | --- | ---- | --- |
   | 1151 | `tooling` | +312 (check: it may have removed a few) |
   | 1153 battle `mod.rs` | `screen stack` | — |
   | 1153 cinematics | `cinematic; bundle` | — |
   | 1153 `map_view*` | `grid skin` | — |
   | 1153 `mode.rs` | `menu; game` | — |
   | 1153 dialogue | `list cursor; text box` | — |
   | 1153 preparations | `list cursor; question` | — |
   | 1153 profile | `screens` | — |
   | 1153 class change | `list cursor; question` | — |
   | 1153 progress | `frame time` | — |
   | 1153 save | `list cursor; question; screen stack` | — |
   | 1153 battle glyph and view | `drawing` | — |
   | 1153 title | `screen stack; audio` | — |
   | 1153 `flow.rs` | `screen stack; game` | — |
   | 1153 info | `drawing` | — |
   | 1153 options | `options` | — |
   | 1153 stage | `drawing` | — |
   | 1153 Playdate | `platform; audio` | — |
   | 1160 | `list cursor` | −48 |
   | 1158 | `screen stack` | −96 |
   | 1159 | `screen stack` | +67 |
   | 1161 | `game` | −151 |
   | 1162 | `menu` | −68 |
   | 1163 | `question` | −24 |
   | 1165 | `drawing` | −6 |
   | 1167 | `grid skin` | −196 |

   Rows added by tickets merged after this one was written get filled the
   same way.
4. Change the opening paragraph: the three-rows trigger is checked by
   `cargo xtask friction-check`, which counts the rows after the last
   `(sweep)` row.

### 2. `code-lines` prints net

In `crates/xtask/src/code_lines.rs`, `table` gets a third column `net`
(`added as isize − removed as isize`, printed with its sign: `+67`,
`−151` written as `-151` in ASCII, `0`), on every crate row and both
totals. Update `the_table_has_a_row_per_crate_and_totals` and add a
case where removed is larger than added. The doc comment of `run` and
the skill's step 2 name the column.

### 3. The refactor rule

In `.claude/skills/engine-friction/SKILL.md`, step 2 *Measure*:

- The measure stays the **added** column of `total (game)`, and the
  ratio stays added ÷ estimate.
- **New rule: when `net` of `total (game)` is zero or negative, the
  ratio triggers nothing.** The change made the game smaller, so the
  extra added lines are rewritten lines, not glue. The PR line says
  "Engine friction: estimate N, measured M, net −K: a refactor, no
  ratio", and no log row is written unless a signal (step 3) fired;
  a signal still triggers the review at any net.
- Why this rule and not "estimate the changed lines": the 1161, 1162,
  1163, 1165 and 1167 notes each said to estimate the rewritten lines
  and the next estimate still missed. A rule the tool decides beats a
  habit. Applied to the log, it would have skipped 1160, 1161, 1162,
  1163, 1165 and 1167 and kept 1159 (+67), the one that grew.
- The ratio table gets a first line: "net ≤ 0 | nothing on the ratio;
  signals still apply".

Also in the skill:

- **Step 7** (the log row): the row has `Area` (from the list in
  `docs/engine/friction.md`, `;` for several) and `Net`; the example row
  shows both.
- **Step 3**, the fourth signal: "`cargo xtask friction-check` reports
  it" replaces counting by hand.
- **Sweep, When a sweep runs**: both triggers are checked by
  `cargo xtask friction-check` in the gates; it fails until the sweep
  ticket is filed. The sweep ticket's Context has one line
  `**Sweep trigger:** milestone \`11xx\`` or `**Sweep trigger:** area
  \`menu\`` (several triggers, several lines). It is how the check finds
  it.
- **Sweep, How a sweep runs, step 4** (Log): each `(sweep)` row has an
  Area and `—` for Net.
- **Worked examples 1 and 2**: their log rows gain an Area (`text box`;
  `scene`, which joins the list if the example keeps it, or use
  `cinematic`) and a Net.

### 4. `cargo xtask friction-check`

New `crates/xtask/src/friction_check.rs`, wired in `main.rs` like
`ticket-lint`. It reads `docs/engine/friction.md`, `docs/ROADMAP.md`
and the files in `tickets/open/` and `tickets/done/`. No git, no
network, no diff.

1. **Read the log.** Parse the first table whose header row starts
   `| Date | Ticket |`. Find columns by header name, not position. Fail
   with a clear message on a row whose `Area` names an area not in the
   *Areas* list, or with an empty Area or Net.
2. **Area trigger.** Take the rows after the last row whose Ticket cell
   contains `(sweep)`. A row **counts** when its Outcome cell starts with
   `accept` or `4` (any of its findings is outcome 4, `4:` after a `;`
   too) or its ratio is ≥ 2 and < 3, **and** its Net is `—` or above
   zero (the refactor rule). Count per area (`game` and `tooling` count
   like any other). At three or more on one area, the trigger fires
   unless **an open sweep ticket** exists: a file in `tickets/open/`
   with a `**Sweep trigger:**` line (any area: a sweep covers the whole
   game).
3. **Milestone trigger.** Read the block column of the *Milestones*
   table in `docs/ROADMAP.md` (`00xx`, `01xx`, … `11xx`). A block is
   **closed** when `tickets/open/` has no ticket of that block other
   than sweep tickets (the sweep ticket of `11xx` lives in `11xx`
   itself). For a closed block, the trigger fires unless a ticket in
   `tickets/open/` **or** `tickets/done/` has `**Sweep trigger:**
   milestone \`NNxx\`` for it. Today no block is closed, so nothing
   needs a baseline list; if a block closed before a sweep ticket
   existed, its sweep is simply due.
4. **Message** on a fire, one per trigger, then exit code 1:

   ```text
   friction-check: a sweep is due: three rows on "list cursor" since the last sweep (1160, 1175, 1180) count as accept or 2×–3×.
   Write the next sweep ticket now, in this PR, with the write-ticket skill: copy 1153 as the engine-friction skill's Sweep section says, and put "**Sweep trigger:** area `list cursor`" in its Context.
   ```

   and for a milestone: "…the `05xx` block has no open tickets left and
   no sweep ticket names it…". On success it prints one line:
   `friction-check: no sweep due (areas since the last sweep: screen stack 1, …)`.
5. **Fail, not warn.** A warning in a CI log is what the current rule
   already is: nobody reads it. Failing costs one ticket file, written in
   the PR that tripped it, which is exactly the skill's "the session that
   sees the trigger writes the sweep ticket": the PR that writes the
   third row, or that moves a block's last ticket to `done/`, is the one
   that turns red. It never asks for a ratio or a line count, so it can't
   be gamed by padding an estimate (the reason ADR-0092 rejects a ratio
   gate).
6. **Gates.** Add `cargo xtask friction-check` to the `run-gates`
   skill's list (after `engine-pin --check`, with a comment) and to
   `.github/workflows/ci.yml` as a step after "Run ticket lint" in the
   same job.
7. **Tests** (unit, in the module, over strings and a temporary folder):
   - three counting rows on one area with no open sweep ticket fire;
     the same with an open sweep ticket pass;
   - a row with a negative Net doesn't count; a 2.5× row and an
     `accept` row with Net `—` count; a 1.5× row with outcome `3:` doesn't;
   - rows before the last `(sweep)` row don't count;
   - a row with two areas counts for both;
   - an unknown area fails with its name in the message;
   - a block with only a sweep ticket open is closed; a closed block
     passes when a done ticket names it and fails when none does;
   - and one test that runs the check on the repository itself and
     passes (it is the acceptance criterion below).

### 5. ADR-0092

Add an *Amendment (1174)* section, following the `write-adr` skill's
rule for amending: the log has Area and Net; a negative net turns off
the ratio; `friction-check` is a gate on whether a due sweep is
**filed**, not on any number, so the "no CI gate on the ratio" decision
stands.

### 6. Schedule the second sweep

Write the second sweep ticket with the `write-ticket` skill, the next
free `11xx` number, titled "The second engine-friction sweep: measure
the game's biggest jobs against what they should take, and file the
fixes":

- copy 1153 as the skill's *Sweep* section says (its two changes: the
  trigger in its Context, and the last sweep's filed rows not yet done
  among its candidates);
- `**Sweep trigger:** milestone \`11xx\`` in its Context;
- `blocked_by`: every other open `11xx` ticket at the time (today 1166,
  1168, 1169, 1170, 1171, 1172, 1173 and 1174 itself), so it is picked
  up when the block closes; `1172` is needed first anyway (its
  `--tree`);
- `model: reasoning`, `effort: high`, like 1153.

Then in `docs/ROADMAP.md`, the line this ticket's PR added (*The
engine*, step 2) names the sweep ticket's number in place of "the
second sweep". A new `11xx` ticket written later adds itself to the
sweep's `blocked_by` (say so in the skill's *When a sweep runs*).

## Acceptance criteria

- [ ] `docs/engine/friction.md` has an *Areas* list, and every row of its
      table has an `Area` from that list and a `Net`.
- [ ] `cargo xtask code-lines` prints a `net` column on every row and on
      `total (game)`; the table test covers a negative net.
- [ ] The `engine-friction` skill's steps 2, 3 and 7, its *Sweep*
      section and both worked examples match the new rule and columns.
- [ ] `cargo xtask friction-check` passes on this branch, printing the
      per-area counts; each unit test in step 4.7 passes.
- [ ] Adding three counting rows on one area to a copy of the log (a unit
      test) makes it fail with the message in step 4.4.
- [ ] The `run-gates` skill and `.github/workflows/ci.yml` run
      `cargo xtask friction-check`.
- [ ] ADR-0092 has the amendment.
- [ ] The second sweep ticket exists in `tickets/open/` with its
      `**Sweep trigger:**` line and the `blocked_by` in step 6, and
      `docs/ROADMAP.md` names it.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `friction_check.rs` (step 4.7) and the `code_lines.rs` table
  test.
- Integration: the check run on the repository's own files passes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
