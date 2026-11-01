---
id: "0753"
title: "Wren's revelation keeps its source on Classic death paths"
type: bug
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: high
status: todo
blocked_by: ["0748", "0749", "0750"]
nick_input: none
completed:
---

# 0753 — Wren's revelation keeps its source on Classic death paths

## Context

The 2026-11-01 audit found a knowledge-continuity miss between
[PR 480](https://github.com/Zafnok/visions-of-shuyi/pull/480) (0748) and
[PR 489](https://github.com/Zafnok/visions-of-shuyi/pull/489) (0750).
Rue is `heretic`; Hollis is `retainer`.

In `assets/dialogue/ch06.dlg`, `ch06_after` tells the party that Wren
walked off the Ashfields with two keepers in gray only when Rue is
present. If Rue died earlier in Classic, the corresponding branches
omit the revelation. In `ch08_victory`, living Hollis nonetheless says
Rue saw this, and without Hollis the lead repeats the same walking and
gray-keepers facts. Chapter 7's "catalogued, not lost" does not establish
that eyewitness account. Checking who is alive now cannot establish
whether a fact was disclosed earlier.

0750's critique pass claimed no contradictions with the ledger. Both
`docs/story/chapters/ch08.md` and the ledger now repeat that unsupported
assumption. The finale's required reveal is still canon: preserve the
outline's two versions and Dace's turn, rather than remove the reveal or
make Rue survive. 0854 integrates these corrected scenes afterward.

## Nick input

None for a continuity repair within the established beats and cast.
Follow `story-writing` and its canon order. If a proposed remedy changes
a canon beat, creates new lore or needs a new named character, ask Nick
with `ask-nick` before writing that dependent change. Do not invent a
gameplay rule, force an optional companion to survive, or ask for a check
after merge. Verification is rendered-path tests and fresh-reader review;
the wording is read in the Act 1 playtest (1064).

## Scope

**In:**
- Repair the earliest necessary disclosure and later attributions so the
  information reaching Dace has an established source on every supported
  Classic path, including Rue dead before Chapter 6.
- Keep knowledge already learned when Rue later dies, and distinguish
  that case from her dying before she could tell it.
- Keep the chapter beat sheets and the ledger's knowledge/absence entries
  consistent with the actual rendered scenes.
- Targeted regression fixtures using real parsed scenes and presence
  histories, not merely a search for character-name strings.

**Out (do not do):**
- Rewriting the chapters wholesale; changing the finale, twists, cast,
  recruitment or death rules; new art, voices or music.
- Building Chapter 8's battle, triggers and places (0854, required after
  this ticket), or the act-ending flow (1065).
- A generic story-knowledge engine or a full-campaign automation system.

## Implementation steps

**Size:** about 60–150 changed script/beat-sheet/ledger lines and 150–250
test lines, at most 1,200 changed lines total including this ticket's
archive move. Compare the existing presence tests in
`crates/content/src/dialogue/tests/presence.rs` and 0750's absence branches.

1. Read `story-writing`, the outline's Chapter 6–8 and Classic branches,
   the three scripts and beat sheets, and the ledger's "Who knows what"
   and absence entries. Identify each fact used in the finale and its
   earliest supported disclosure on the path, keeping canon intact.
2. Add a failing rendered-path regression first for Rue dead before
   Chapter 6, with Hollis alive and with Hollis gone. Use the real
   `Scene::resolved`/`Present` path and player text; record the scenes
   that establish the facts before the gangway repeats them.
3. Repair only the necessary scene beats and attributions. Use existing
   facts/sources and authoring mechanisms; if persistent knowledge state
   is necessary, file a narrowly scoped implementation prerequisite
   before adding a new game mechanic here.
4. Also render Rue present for the disclosure but dead by Chapter 8,
   Rue alive throughout, and Hollis falling in Chapter 8. No later death
   may erase legitimately learned information or restore a dead speaker.
5. Fresh-reader continuity pass on each affected rendered path; update
   beat sheets and ledger, run `cargo xtask check-script` and content
   gates. Leave 0854 concrete scene/path fixtures to preserve when wired.

## Acceptance criteria

- [ ] Regressions expose the old unsupported attribution and pass after
      repair for Rue dead before Chapter 6, with Hollis alive or gone.
- [ ] Every eyewitness/detail used at the gangway has an earlier source
      on that rendered path; "catalogued" alone is not treated as proof
      of how Wren left the field or who accompanied her.
- [ ] Rue telling the party and dying later preserves learned knowledge;
      the living-Rue path, both Hollis versions and the required finale
      reveal still work without speaking for a dead companion.
- [ ] Beat sheets and ledger knowledge/absence entries match those paths;
      a fresh reader's continuity findings and resolutions are in the PR.
- [ ] `cargo xtask check-script` and the content gates pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content: real-scene presence/history regressions for the cases above.
- Integration: a targeted player-text rendering sequence proving where
  each used fact was learned; 0854 repeats the relevant paths when wired.
- No new whole-chapter snapshots or generic narrative inference checker.

## Completion notes

*(Filled in by the session that completes the ticket: repaired paths,
source-of-knowledge evidence, critique pass, follow-ups and notes for 1064.)*
