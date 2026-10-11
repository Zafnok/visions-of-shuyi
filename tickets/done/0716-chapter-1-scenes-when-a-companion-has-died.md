---
id: "0716"
title: Chapter 1 scenes when a companion has died
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: done
blocked_by: ["0707", "0715"]
nick_input: sign-off
completed: 2026-10-04
---

# 0716 — Chapter 1 scenes when a companion has died

## Context

The Chapter 1 script (0707, `assets/dialogue/ch01.dlg`) is written for a
battle everyone lived through. In Classic a companion who falls is dead
(`docs/design/death-and-difficulty.md`), and today they still talk in the
victory scene: Tamsin closes Harl's eyes, Rue recognises the seal, Maud
says they can't stay, Aske asks for her arrows, Hollis looks at the wax.
0715 gives scripts a way to mark lines by who is still in the army. This
ticket uses it in Chapter 1, following the `story-writing` skill.

The story's rule is already set (0701, approved at the gates): a dead
companion is gone, and scenes are written so "their absence is felt"
(`docs/story/characters/retainer.md`, "Classic permadeath"). Nothing here
is a new design decision.

## Nick input

**Sign-off:** the PR lists, for each companion, what the victory scene does
without them, as text. Nick comments if a beat reads wrong.

## Scope

**In:**
- `ch01_victory`: every line a companion speaks, and every line about what
  a companion is doing, works when that companion is dead. The chapter's
  facts must still reach the player on every path: the order bears the
  Veyne seal; only Dace can use it; they leave for Veyne Hall. Rue's
  suspicion ("I've seen that seal before") is hers alone: if she is dead
  it is not said, and the ledger records that it wasn't.
- `ch01_victory`'s reply choice: a reply answered by a companion needs an
  answer when they are dead (Hollis, Tamsin and Aske answer today).
- In-battle scenes that speak to or about another companion who may have
  fallen first: `ch01_first_turn` (none can have), `ch01_death_keeper` and
  `ch01_retreat_keeper` (Maud speaks to Rue), `ch01_death_heretic` (Rue
  names "the Keeper"), `ch01_boss_engage_sergeant` (only Tamsin; fine).
  Check each and fix the ones that read wrong.
- `ch01_prebattle` and `ch01_intro` need nothing: nobody can be dead yet.
  Say so in the Completion notes rather than leaving it unsaid.
- `docs/story/ledger.md`, "After Chapter 1": for each companion, what is
  different if they died in Chapter 1 (what they never said, who knows
  what instead). Later chapters' scripts read this.
- The critique pass, including one read-through per dead companion and one
  with only the lead and Hollis alive.

**Out (do not do):**
- The engine (0715).
- Chapters 2 and later. Their script tickets handle their own scenes; this
  ticket only records in the ledger what a Chapter 1 death changes.
- A grief scene longer than the budget allows: `ch01_victory` stays at
  about 20 text boxes on every path (`docs/story/chapters/ch01.md`,
  "Budget").
- New reply choices.

## What 0715 built (read first)

The block is `@if <character>` … `@else` … `@endif`
(`assets/dialogue/README.md`, "Who is still there"; ADR-0055). What
matters for this ticket:

- Blocks nest and may stand in a reply's reaction; a `@choice` may stand
  in a block.
- Both parts of a block must leave the same characters on screen and the
  same caption: a companion who enters in a block leaves in it.
- The scenes of a trigger can count on the characters the trigger names
  (a death quote on the one dying, the boss scenes on the boss). The
  first chapter's intro scenes, and its battle's turn-1 scene
  (`ch01_first_turn`), can count on the whole starting party: no blocks
  there.
- A scene with nothing left to say for those there isn't played.
- **The check only bites for characters in the New Game roster**, and the
  companions join it in 0803 (now blocked by this ticket). Until then
  `ch01.dlg` passes whatever it does. So this ticket adds a test
  (`crates/content/src/dialogue/tests/presence.rs`) that runs
  `check_presence` on the scenes of `ch01.dlg` with a `Cast` written by
  hand from the script's header: the five companions in `may_be_absent`;
  `certain` for each trigger scene as its trigger names (and all five for
  `ch01_intro`, `ch01_prebattle` and `ch01_first_turn`). It expects no
  errors. 0803 deletes it once the battle and chapter files say the same.

## Implementation steps

1. Reread the canon (the skill's canon order), `ch01.dlg` and the sheets of
   all six.
2. For each of the five companions, write down what `ch01_victory` loses
   without them and who or what carries it instead. Starting points:
   - **Tamsin dead:** nobody jokes over Harl. Hollis finds the orders.
     Her "as far as there's pay" is gone; someone notices her horse, or her
     ledger. The ledger went to the lead in her death quote.
   - **Rue dead:** no seal line. Maud's "others will come" loses its
     reason (they came for her too); he can still say the order named
     three and two are standing.
   - **Maud dead:** who explains that the seal goes with the title?
     Hollis won't. Tamsin can ("I thought they took it off you").
   - **Aske dead:** her arrows line goes; the blunt reply needs another
     answer.
   - **Hollis dead:** the hardest one. "Everyone looks at Hollis" becomes
     nobody being left to look at. The spade line goes; someone else
     buries Harl, or nobody does. The line before the reply choice is his.
3. Mark the lines with 0715's block. Keep the all-alive path word for word
   as 0707 wrote it unless the critique finds a reason to change it.
4. Count text boxes on the longest and shortest paths; both within budget.
5. Validate (`cargo test -p trpg-content`), then the critique pass.
6. Update the ledger.

## Acceptance criteria

- [x] No scene in `ch01.dlg` puts a dead companion on screen or has one
      speak (0715's check passes with its rule on for this file).
- [x] With any one companion dead, and with all five dead, `ch01_victory`
      still tells the player the three facts in Scope.
- [x] The all-alive path is unchanged, or each change is listed in the
      Completion notes with its reason.
- [x] Critique checklist completed and pasted in the Completion notes.
- [x] Ledger updated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the all-assets test (the script validates).
- The one test described under "What 0715 built": `ch01.dlg` passes the
  presence check with its five companions able to be gone. No other new
  code tests: 0715 tests the mechanism.

## Completion notes

Written on Opus 5.5, not fable-5.1: Nick's choice for this session.

### What was done

- `ch01_victory` is rewritten with 0715's `@if` blocks. It plays for any
  of the 32 combinations of the five companions. Longest path 20 text
  boxes (everyone alive, as before), shortest 14 (only the lead left).
- The three facts reach the player on every path: the order's seal is in
  narration nobody can be missing from; who may use it is said by Maud,
  or Tamsin, or narration; Veyne Hall is said by Hollis or narration,
  and by the blunt reply.
- New test `chapter_1_never_shows_a_companion_who_may_have_fallen`
  (`crates/content/src/dialogue/tests/presence.rs`), as the ticket
  describes. A step in 0803 now says to delete it.
- Ledger: "If a companion died in Chapter 1 (Classic)". Beat sheet: one
  bullet under scene 4.

### What the victory scene does without each companion

**Tamsin dead.** Nobody jokes over Harl or closes his eyes. Narration:
"Harl Coster lies where he fell. Tamsin's ledger is in [the lead]'s coat
now, and nobody is left to mark him paid." Hollis gets down "one knee at
a time" and finds the order: "The sergeant would have had a word for
him. I haven't." (If Hollis is dead too, the lead kneels and takes it.)
Nobody says Dace's name aloud: narration says it, "Nobody says the name",
and Hollis looks at the wax. Rue's line loses "I'll say something." If
the player picks the wry reply, Hollis: "That one was for the sergeant.
She'd have had an answer." Before the spade, Hollis: "The sergeant goes
in the chapel yard, in the red jacket. Somebody see to her horse. It's
still waiting on her."

**Maud dead.** Tamsin works the seal out herself: "They did. A seal goes
where the title goes. One man in the kingdom may press that into wax
now, and it isn't you." (With Tamsin dead too, narration says it.) Nobody
says the Vigil will send others; Aske says the plain version: "Someone
paid for these men. He will pay for more. I come with you. You owe me
eleven arrows, noble." Hollis: "The keeper goes by her own chapel wall.
There's nobody left who has the words, so I'll dig it deep. It's what
I've got."

**Rue dead.** "I've seen that seal before" is not said, and nothing
replaces it: the lead's "Where?" and her "You tell me" go with it. Maud
can no longer warn Rue, and nearly gives herself away: "Rue slept by my
stove all autumn, and somebody was told so. I... somebody was told. They
will hear that it is not finished here, and they will send others."
Hollis: "The girl goes in the yard with her hands wrapped. She was on
that paper next to me, and I never once asked her why."

**Aske dead.** Her arrows line goes. The blunt reply is answered by
Hollis: "Aye. That's the word." Hollis: "Aske faces north. She'd want
that. Her silver's yours to send, kid. Every coin of it."

**Hollis dead.** Maud can't finish in front of him, so she ends "...Oh,
Ama keep the Captain." Nobody is left to look at: Tamsin says "Dace
Marr, hero of the war. The Captain's boy. And the Captain went down this
morning without having to see it. Somebody say something. I'm no good in
a quiet." Nobody asks the lead to go; narration, before the reply
choice: "Dace is at Veyne Hall. One step off the march to ask him why,
and [the lead] is an outlaw for good. Hollis would have said so. Then he
would have said: go." Replies: earnest, "Nobody answers. Hollis would
have."; wry, Tamsin: "It's soldiering with worse food. The company was
better yesterday."; blunt, Aske: "Yes. Now. This fort smells of them."
Then: "[The lead] digs Hollis's grave [himself/herself], and takes the
letter from inside his coat. [He/She] does not burn it either. Nobody
digs a hole for Harl Coster." With Hollis dead, the other dead get a
line of narration each instead of his (Tamsin's horse at the gate; nobody
to say the words over Maud; Rue buried "under the only name anyone here
knew her by. Whatever she knew about any of this, she kept."; Aske's
grave facing north, her silver in the lead's pack).

**Only the lead and Hollis alive** (15 boxes): Hollis finds the order;
narration gives the seal's law and Dace's name; Hollis asks; after the
reply he names all four dead, one line each, then "Go and pack. I'll
find a spade."

**Only the lead alive** (14 boxes): all narration. The lead takes the
order, the narration gives the seal, the name and the road, the lead
answers nobody, buries five, and is on the road by dusk.

### Changes to the path where everyone lives

Every spoken line is word for word 0707's. Two narration lines changed,
both because they said "they" of a party that may be the lead alone:

1. First line: "Behind them, one barn smokes..." is now "Across the
   river, one barn smokes...".
2. Last line: "By dusk they are on the road out of Harrowby. Past the
   boundary stone, every step [the lead] takes..." is now "By dusk [the
   lead] is on the road out of Harrowby. Past the boundary stone, every
   step [he/she] takes...".

And the portraits, not the words: a block must leave the same screen
whether or not its character is there, so a companion who enters in a
block leaves in it. With everyone alive the right side is now empty
during "His orders..." (was Tamsin), the lead is on the left and the
right is empty during "Everyone looks at Hollis" (were Maud and Tamsin),
and Aske, Rue and Tamsin each say their coming-along line next to the
lead rather than next to each other.

### Deviations

- **The reply choice is written twice**, once in Hollis's block and once
  in its `@else`. Hollis's line stays in the box while the choice is
  open, and a block that ended before the choice would have taken his
  portrait and name plate off it. So the three replies have a second
  line id each (`..._2`), played only when Hollis is dead.
- **The dead are named after the choice, not where they would have
  spoken.** One line each, said by Hollis before his spade line (or
  narrated if he is dead). Spread through the scene they pushed several
  paths past 20 boxes, and left "I've buried better men than Harl
  Coster" as the only word on burying anyone.
- **In-battle scenes.** The ticket says Maud speaks to Rue in
  `ch01_death_keeper` and `ch01_retreat_keeper`; in 0707's final text she
  doesn't ("to the one it hurt"), and the line holds whether Rue is alive
  or not. `ch01_death_heretic` speaks of the Keeper's stew: "I'd have
  told him it was" still said **him** from when the keeper was Piers
  (0040 made her Maud), so it is now "her"; it reads right whether Maud
  is alive or dead. `ch01_first_turn`, the boss scenes and the other
  death and retreat lines needed nothing.
- `ch01_intro` and `ch01_prebattle` need nothing: nobody can be dead yet.
  The new test treats all five as certain there and in `ch01_first_turn`.

### Critique checklist

Read as an editor on all-alive, each one dead, lead + Hollis, lead only,
and three mixed paths (Tamsin only, Aske + Rue only, Maud + Aske dead).

- [x] Voice: none of the new lines could be swapped between speakers.
  Hollis is short and flat ("It's what I've got."); Tamsin talks in
  prices without a punchline ("The company was better yesterday."); Maud
  fails to hide the report out loud; Aske counts cause and effect ("He
  will pay for more.").
- [x] Exposition: the seal's law is one box on every path.
- [x] Contradictions. Fixed three found on the read-throughs: Maud's
  line for a dead Rue began "She" with nobody named; Hollis buried Rue
  "too" when she was the only one dead; with Maud dead Aske said "I come
  with you" before anyone had said they must leave. Checked against the
  death quotes: the ledger and the silver are the lead's because Tamsin
  and Aske gave them.
- [x] Scenes where nothing changes: every path still ends with the lead
  an outlaw on the road to Veyne Hall.
- [x] Lengths and expressions: the validator passes; only the five
  basic expressions are used in the new lines.
- [x] Main plot and a personal arc: the seal and the road (plot); each
  death closes that companion's Chapter 1 arc in one line.
- Left as it is, and why: with three or more dead some joins are abrupt
  (Tamsin alone says "Somebody say something" and then "And I'll ride
  along" to nobody). Tone follows *Where the jokes go*
  (`setting-and-tone.md`): no new line is a joke. 0707's own jokes in
  this scene are 0724's to rewrite, with these lines.

### Story calls made here (Nick may veto)

No gameplay rules. Four small story facts, all in the ledger:

- If Hollis dies, the lead takes Dace's letter from his coat and keeps
  it.
- If Hollis lives he buries the dead and then Harl; if he dies the lead
  buries the dead and nobody buries Harl.
- The graves are at Harrowby's chapel; Aske's faces north.
- Tamsin's horse is left waiting; who takes it isn't said.

### For later chapters (in the ledger, no tickets created)

A Chapter 1 death removes things later scripts count on: Rue is the only
one who knows Wren is alive and saw Dace at the cart; Maud's confession
needs Maud; Hollis's Chapter 4 to 8 beats need Hollis; Aske sees through
the false flag in Chapter 2. Each chapter's script ticket reads the
ledger section. Whether Dace learns his order killed his father is left
open there.
