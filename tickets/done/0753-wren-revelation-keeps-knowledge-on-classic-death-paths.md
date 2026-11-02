---
id: "0753"
title: "Wren's revelation keeps its source on Classic death paths"
type: bug
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: high
status: done
blocked_by: ["0748", "0749", "0750"]
nick_input: none
completed: 2026-11-01
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

- [x] Regressions expose the old unsupported attribution and pass after
      repair for Rue dead before Chapter 6, with Hollis alive or gone.
- [x] Every eyewitness/detail used at the gangway has an earlier source
      on that rendered path; "catalogued" alone is not treated as proof
      of how Wren left the field or who accompanied her.
- [x] Rue telling the party and dying later preserves learned knowledge;
      the living-Rue path, both Hollis versions and the required finale
      reveal still work without speaking for a dead companion.
- [x] Beat sheets and ledger knowledge/absence entries match those paths;
      a fresh reader's continuity findings and resolutions are in the PR.
- [x] `cargo xtask check-script` and the content gates pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Content: real-scene presence/history regressions for the cases above.
- Integration: a targeted player-text rendering sequence proving where
  each used fact was learned; 0854 repeats the relevant paths when wired.
- No new whole-chapter snapshots or generic narrative inference checker.

## Completion notes

**The bug.** `ch08_victory` had Hollis say "Rue saw her. Walking off the
field, after, with two keepers in gray" whenever he was alive, and the
lead say "She walked out of the fire. Two keepers in gray took her"
whenever he wasn't, but Rue alone had seen it, and on a Classic path
where she fell before `ch06_after` nobody had told anyone. An `@if`
asks who is in the army now; it can't tell "Rue told them and fell
since" from "Rue fell before she could", and both paths render the same
gangway.

**The repair (scripts only; no new mechanic, no knowledge state).**
The rule it rests on, now in `assets/dialogue/README.md` (*Rules for
`@if` blocks*): a line in a dead companion's `@else` must be true
whether they fell before or after they could have spoken.

- `ch07_crane`: when Rue is not in the room, Crane adds how Wren left
  the field ("Marr's girl walked off the field on her own feet, between
  two keepers in gray, and was entered in my book that evening. You've
  mislaid the one with the burned hands. She saw it."). Rue alive there
  means she told them on the dead ground, so he says no more; Rue gone
  means her first telling on one path and a confirmation on the other,
  true on both. Without Hollis, the lead asks ("Wren Marr. Where is
  she.") and Crane answers the same way, so the facts reach the lead on
  every path.
- `ch08_victory`, Hollis's version: Rue's name only while she is there;
  gone, "She walked off that field. After. Two keepers in gray had her."
  The lead's version was already sourced once Crane speaks in Chapter 7.
- `ch08_death_retainer`: "what the girl saw on the plain" only with Rue
  on the map; else "you tell him she walked off that plain".
- `ch06_after` is unchanged: without Rue, Hollis is honestly told
  nothing that night.

**Source-of-knowledge evidence.**
`crates/content/src/dialogue/tests/knowledge.rs` plays the real scenes
(`Scene::resolved` for who is in the army, names substituted) along ten
Classic paths and reads the player text: a relay of "walked off" or
"keepers in gray" by anyone but Rue or Crane needs an earlier source on
that path; "Rue saw her" needs Rue to have said it herself; the gangway
and the dock carry both facts on every path; and where each fact was
first said is pinned per path (the dead ground when Rue was there, the
barracks when she wasn't). Six of the ten failed on the old scripts
(the three "Rue fell before Chapter 6" paths, the Hollis death-quote
path, and the two "told, then fell" paths whose gangway named her).

| Path | First source | Gangway |
| ---- | ------------ | ------- |
| Rue fell before Ch6; Hollis dies on the quay | Crane, `ch07_crane` | Hollis, no name |
| Rue and Hollis both fell before Ch6 | Crane, after the lead asks | the lead |
| Rue fell before Ch6; Hollis fell in the Undercroft or the docks battle | Crane | the lead |
| Rue told Hollis, fell before Ch8 | Rue, `ch06_after`; Crane confirms if she is gone by Ch7 | Hollis, no name |
| Rue told Hollis; both gone | Rue, `ch06_after` | the lead |
| Rue alive | Rue, `ch06_after` | "Rue saw her", or Rue herself |

**Critique pass** (a fresh Opus cold read of both repaired paths, the
scenes as the player sees them, with what the party knew walking in).
Continuity held on both paths once Crane speaks. Hits on the new lines,
all fixed: "she was there for it" read as the writer's note and as
slang, and "she... The girl" set a pronoun trap (now "Marr's girl
walked off... You've mislaid the one with the burned hands. She saw
it."); "entered that evening" could mean "went in" (now "entered in my
book"); Crane's reply to the lead repeated "heir of House Veyne" from
his next box (dropped); Hollis's dying line carried one fact too many
("on her own feet" cut). The reader found the lead's "Wren Marr. Where
is she." earned (Hollis's question in Hollis's clipped style) and
Hollis's new line the same man as "Up. All of you, up."

**Deviations.** None from the steps. Not changed, because they are
0749's and 0750's wording and not continuity: the cold read's notes
below.

**No starting rules:** nothing gameplay-affecting was decided.

**Follow-ups:** none filed. **Notes for 1064** (wording the Act 1
playtest reads, from the cold read): the lead's four one-line boxes on
the Hollis-less gangway read as a recital (the 40-character limit stops
a merge); "Boy! ...Veyne." switches addressee mid-box in Hollis's death
quote; "There are no orders" and "He looks at the lead's shoulder. It's
up." were unclear to a cold reader; the warden's side wasn't clear at
"Nobody in here, sir"; "walking about", "out of interest" and
"catalogued" read as British (the scripts and docs use "catalogued"
throughout, so it was left consistent); the reader would skip the
Othe-to-"A door opens" lore run. The lead's "walked out of the fire" on
a path where the source said "off the field" is the outline's own
wording and was kept.
