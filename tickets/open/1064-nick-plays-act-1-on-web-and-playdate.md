---
id: "1064"
title: "Nick plays Act 1 on web and Playdate; feedback becomes tickets"
type: playtest
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: [] # and every ticket whose frontmatter says blocks: ["1064"]
nick_input: sign-off
completed:
---

# 1064 — Nick plays Act 1 on web and Playdate

## Context

The one playtest left before release work: all of Act 1, Chapters 1–8
(`docs/story/outline.md`: Chapters 1–3 linear, 4–8 on the Act 1 world
map), on the web build and on Nick's Playdate. It replaces the Chapter 1
playtest's round 4 (0804) and the Chapter 1 play-through on the Playdate
(1050), both closed into this ticket on 2026-10-21 (ticket 0129).

Nick, 2026-10-21: "any tickets that say 'check after merge' aren't gonna
happen -- I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself. I don't wanna spot
verify any gameplay tickets anymore." So between now and this ticket,
gameplay tickets are verified by tests, `cargo shuyi frame-png` pictures
and the playtest bots (0506, 0509; the whole act by 0512), and whatever
was left to judge by playing collects here.

Its blockers (each says `blocks: ["1064"]` in its own frontmatter since
1352; `tickets/README.md`) are: the bots' run of all of Act 1 (0512), which itself
waits for the content and the Playdate work; the round 3 tickets 0804
was waiting for (its open blockers, carried over); the Playdate tickets
1050 was waiting for; and the Act 1 critical path in `docs/ROADMAP.md`
(*Act 1 critical path*, Phases 2 and 3).

The 2026-11-01 PR audit also gates this playtest on the same-combat
first-hit correction (0330), AI art reach and Cascade scoring (0521),
the death-path knowledge correction before the finale (0753), and the
specified camp/world-map music cycle (1080, the fade-ins and quiets;
1087, the 3 s fade-out, after engine 2398). Those
tickets prove their behavior automatically before this playtest; Nick
does not become their individual acceptance check. Chapter 2's carried
army proof (0520) and chosen art (0858) remain existing blockers.

Nick's own play of the Act 1 build (2026-11-08) added the tickets of
Lane I in *Act 1 critical path* (0332–0334, 0453, 0485–0491, 0528,
0527, 0610, 0867–0870, 1088, 1089; the decisions 0079–0084 come in
through them). Lane J, the human writer (0152, 0155–0160, 0764–0766, 0914, 0915), is
not a blocker: the playtest runs on the scripts that exist.

2026-11-17 friction sweep
([`docs/research/friction-2026-11-17.md`](../../docs/research/friction-2026-11-17.md),
bucket B1): the Chapter 3-5 art tickets 0861, 0862 and 0866 each said
"1064 covers the look" but were not blockers here; they are now, with
0877 (the per-chapter coverage report) and 0878 (class art). The sweep
measured these gaps in the build this playtest would show. Each one
**waits on Nick's answer in 0184** (wanted for this playtest, or
deliberately left out and said so on the checklist page); none is
decided here:

- **Faces:** 12 portrait sets exist; 20 speakers placed on screen have
  none. 0858, 0881, 0861, 0862 and 0866 cover the named cast; the minor
  speakers have no ticket.
- **Voices after Chapter 1:** 123 clips, all Chapter 1 or Vix; Chapters
  2-8 are silent. 0757 waits on the writer (0766) and is not a blocker
  here.
- **Class art:** Swordsman, Brawler and Shadowblade draw the stand-in
  figure (about 22% of placements); 0878 imports what is decided.
- **Indoor backdrop:** the 4 indoor maps fight in front of the fields
  backdrop; no interior picture is chosen.
- **Promotion unreachable:** the promote and reclass screen is built,
  but no shop, battle or chapter gives a seal (0606).
- **Supports and camp empty:** `supports.ron` has no pairs and
  `camp.ron` no events, so neither shows; their content is 1073, which
  waits on 0766.

The writer trial's test audit of the Chapter 1 demo (2026-11-19) added
three blockers: 1303 (unnamed enemies show the soldier face in unit
info), 0091 (telling the two sides' look-alike units apart on the map)
and 0879 (the number the volumes start at). 0499's starting rule "Red
for every rider, ours and theirs" is now 0091's decision, not a
checklist item; and 0879 makes #624's volume 80 → 97 Nick's decision.

## Nick input

**Playtest:** Nick plays Act 1 from New Game to "Act 1 ends" on the
full web build on the restricted itch.io playtest page (0158,
`docs/releasing.md`, "The playtest page": Pages carries no private
content), and Chapter 1 at least (all of Act 1 if he wants) on his
Playdate (sideload: Simulator, *Device → Upload Game to Device*). He
answers the checklist and reports anything else in plain words: bugs,
feel, difficulty, story, looks, controls, sounds, pauses that feel long.

## Scope

**In:**
- Cut a release the way 0804 did (bump the version in its own PR from a
  branch whose name doesn't start with `t1064`, tag, wait for the release
  and the playtest page's upload, `playtest-itch.yml`) and build the English `.pdx` (`cargo xtask playdate
  --release`).
- **The checklist**, published as a private claude.ai page that saves
  his answers (as 0804's round 3 did), with the links, the key
  cheat-sheet from `keymap.ron`, the Playdate's buttons and crank, a
  short "known to be unfinished" list (the default output of `cargo
  xtask act1-coverage`, 0877, with each gap Nick chose to leave out in
  0184), and questions in play order. Collect them from:
  - the new Chapter 1 script (0737): ask about it first, as round 4
    would have;
  - every ticket in `tickets/done/` whose *Nick input* quotes a waived
    sign-off (`grep -l "What the old sign-off asked" tickets/done`);
  - the agent's starting rules from `cargo xtask playtest-notes
    --by-area` (0183): `Seen in play:` items as questions grouped by
    area, about 30 in all; `Not seen in play:` items under one heading,
    kept unless Nick picks one to discuss;
  - 1050's questions: whether the enemy's pauses on the Playdate feel
    long, whether start-up feels slow (numbers from 0512's device run);
  - the new chapters' starting rules (0848–0854, 1070–1073) and the
    Act 1 battle rules (0069, 0319);
  - the volumes as straight percentages (0882): whether the places'
    sound beds sit well under the music at their starting numbers
    (ambience 90 against music 100, now about 1 dB apart where it was
    6 dB), and whether music, beds and voices sit well together.
- Turn every item of his feedback into a ticket (`write-ticket` skill,
  *Turning Nick's playtest feedback into tickets*): bugs with repro
  steps, feel items as tuning with proposed values, design changes as a
  `00xx` question first. Ask at most one clarifying question per unclear
  item.
- Record his answers that settle a starting rule in the design doc that
  holds it.

**Out (do not do):**
- Fixing anything in this ticket.
- Playtesting Act 2 or anything past "Act 1 ends".

## Implementation steps

1. Check no ticket in `tickets/open/` says `blocks: ["1064"]`, `main` is green and
   the playtest page has `main`'s build (`playtest-itch.yml`'s last run).
2. Version bump PR, tag, release; build the `.pdx`.
3. Build the checklist page from the sources above (`artifact-design`
   and `artifact-capabilities` skills: the page keeps his answers).
4. Send Nick the links and the page.
5. Read his answers back; write the tickets; record settled rules.
6. Put a summary in the Completion notes: his words verbatim and the
   ticket each became (or "won't do" with the reason he agreed to), as
   0804 did.
7. Set every `status: parked` ticket whose blockers are now all done
   back to `todo` (`ticket-lint` warns "unpark it", 0181; a no-op
   before 0181).

## Acceptance criteria

- [ ] Nick played Act 1 on the web build and on his Playdate.
- [ ] Every feedback item maps to a ticket (or an explicit "won't do"
      Nick agreed to); the table is in the Completion notes.
- [ ] The release exists and the `.pdx` was handed over.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (playtest).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
