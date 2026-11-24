---
id: "0490"
title: "Vix's Arts lesson shows that an art is an attack"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-17
---

# 0490 — Vix's Arts lesson shows that an art is an attack

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "We should make it clear Arts are special attacks NOT just buffs that
> don't end your turn. My friend thought Arts are buffs that do nothing bc
> they end your turn. But they are buffed special attacks (attack tied to
> the art). Make it more clear probably Vix forced tutorial or smth"

What an art is (`docs/design/combat-arts.md` ~:120–210 and *The arts menu
and the forecast* ~:756–830, Nick's): a weapon technique picked from the
`Arts` entry of the action menu **before the target**; it boosts every
strike of that one combat and costs the weapon's durability. In code an
art is an attack: `Technique::Art` → `UnitAction::Attack { art, .. }`
(`crates/ui/src/screens/battle/art_list.rs` ~:42–60). Only Overwatch (bow,
rank B, not in Act 1) is an action of its own (`UnitAction::UseArt`,
"ends action").

What the player sees:

- The Arts menu (`art_list::arts_menu` ~:470): a line per art, `Flowing
  Cut  −2 dur  E`, the menu's top border naming the weapon (`Iron Sword
  20/20`), and the focused line's effect under the list (0429), e.g.
  `crit +20` or `no counter`. Nothing says the line is an attack: its
  note reads like a buff.
- The lesson `combat_arts` (`assets/data/lessons.ron` ~:159–168, level
  Short, trigger `ArtsAvailable`): "Arts are heavy hitters with their own
  menu. / They cost weapon wear, or uses per battle." Its only item is
  "Open Arts in the unit's menu" (`ArtsOpened`): the lesson closes as soon
  as the menu opens, before the player has seen an art hit.

Nick asks for a forced explanation, which in this game is a lesson (a
checklist the turn waits for; `docs/design/tutorial.md` *The lessons*,
*Different here*: "Combat Arts: their own menu, and that some cost uses
per battle").

**Text.** Nick paused AI story writing on 2026-11-08. Lesson text is
instructional: write it in plain teaching words, true to the rules above.
The hired writer (0766) may revise Vix's wording later.

## Nick input

None: he asked for it. The new item and words are *the agent's starting
rules*, written in `tutorial.md`, and go on the Act 1 playtest's checklist
(1064). Verified by the Harness tests below.

## Scope

**In:**
1. **A new item kind** `ArtAttacked` (`crates/content/src/lesson.rs`,
   `DoneWhen`): a unit of the player's attacked with an art or a weapon
   active chosen from the Arts menu (`Command::Act` with
   `UnitAction::Attack { art: Some(_), .. }` or `active: Some(_)`, and the
   combat resolved). Noticed in `notice_events`
   (`crates/ui/src/screens/battle/mod.rs` ~:1207), beside `Attacked`.
   Doable under the same rule as `ArtsOpened` (`doable.rs` ~:37: some unit
   left has an art it could use from a tile it can reach); `needs_unit`.
   Documented in the `lessons.ron` header's `done_when` list.
2. **The lesson `combat_arts`**: items "Open Arts in the unit's menu"
   (`ArtsOpened`) and "Attack with an art" (`ArtAttacked`), and new text
   saying an art is the attack itself, boosted, paid in weapon wear. A
   draft that fits the lesson limits (2 lines of at most 45 characters):

   ```
   An art is an attack, not a buff: one fight,
   every strike boosted. It costs weapon wear.
   ```

   ("or uses per battle" goes: arts and the weapon actives in the Arts
   menu cost durability; uses per battle are spells' and non-attack
   actives', `combat-arts.md` ~:125.)
3. **The Arts menu says it too**: the note under the list starts with
   what the art is, e.g. `Attack: crit +20` (a new key, e.g.
   `battle.art.note.attack` "Attack: {effect}"), for every line that
   attacks (not for an art used as an action, which keeps "ends action").
   *The agent's starting rule*; same in the Playdate's menu if it paints
   the note.
4. `tutorial.md` (*Different here*, Combat Arts): Nick's words
   (2026-11-08) and the two items; `combat-arts.md` (*The arts menu and
   the forecast*): one sentence for the note.

**Out (do not do):**
- Any rule of arts, or the `Arts` entry's name (Nick's).
- Arts that let the unit act again (0328). Either order: its arts are
  attacks too, so the item ticks for them.
- When the box at the end of the lessons comes (0485). Either order.
- Vix's voice clip: if 0476 (Vix speaks her lessons and tips) is done,
  the changed text needs a new clip (`docs/voice.md`, `cargo xtask
  voice`); if not, 0476 voices the lesson as it is then.

## Implementation steps

Expected size: about 60–100 production lines (a `DoneWhen`, its notice
and doable rule, data, a note key), plus tests.

1. Write the failing test first: on Chapter 1 at Short, opening Arts no
   longer closes the lesson; attacking with an art does.
2. `ArtAttacked`: enum, notice, doable, content validation.
3. The lesson's data and text; the Arts menu note.
4. Update `finish_short` in `crates/ui/tests/it/ch01_lessons.rs` (it now
   attacks with the Sergeant's art) and the lesson tests
   (`the_combat_arts_lesson_opens_with_usable_arts_and_closes_when_they_open`,
   `the_arts_lesson_doesnt_open_after_the_only_unit_with_arts_attacks`),
   the snapshots and the docs.

## Acceptance criteria

- [x] Opening Arts ticks the first item; the lesson stays open (Harness
      test, `ch01_lessons.rs`).
- [x] Attacking with an art ticks the second and closes the lesson; a
      plain Attack doesn't (tests).
- [x] The turn can't end while the lesson is open and a unit could still
      use an art; it is put away when none can (tests, as 0464's).
- [x] The Arts menu's note reads `Attack: …` on an art line (view test in
      `screens/battle/art_tests.rs`); an action art keeps "ends action".
- [x] The lesson's text passes the lesson checks and names no key.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `DoneWhen::ArtAttacked` noticed for an art and an active, not for
  a plain attack; its doable rule.
- Integration: the Chapter 1 lesson tests above.
- Snapshot: the lesson box and the Arts menu.

## Completion notes

- `DoneWhen::ArtAttacked` (`trpg-content`): ticked when a command
  `Act` with `UnitAction::Attack` carrying an art or an active is fought
  (`art_list::attacks_with_art`, noticed beside `Attacked`). Doable when
  some unit left has an art line it could attack with from a tile it can
  reach: the `ArtsOpened` rule minus an art used as an action of its own
  (Overwatch), which is no attack. `needs_unit`.
- The lesson `combat_arts` (public stand-in and the private
  `content/data/lessons.ron`, private PR
  `Zafnok/visions-of-shuyi-assets` branch `t0490-arts-lesson-attack`):
  items "Open Arts in the unit's menu" and "Attack with an art", text
  "An art is an attack, not a buff: one fight, / every strike boosted. It
  costs weapon wear." (the ticket's draft).
- The Arts menu's note under the list reads `Attack: <effect>` (key
  `battle.art.note.attack`); Overwatch keeps its effect alone. The menu
  grows by the note's width where needed (one column for the Quick
  Battle lord's).
- Tests: the two Chapter 1 tests (`ch01_lessons.rs`: opening Arts ticks
  one of two and keeps the lesson; an art attack closes it; a plain
  attack doesn't and the lesson is put away; the turn can't end while it
  is open), `finish_short` attacks with the Sergeant's art, a private
  snapshot of the lesson box over the Arts menu, the doable and
  notice unit tests, the menu-note test.
- Voice: 0476 is done, so the new text needed a clip or
  `every_lesson_and_tip_the_guide_says_has_a_clip` fails: made with
  `cargo xtask voice generate --scene vix` on the local voice lab
  (`vix_fb8784ee`, audit clean: 7.1 s, no long pause), the old clip
  removed, both in the private PR.

*The agent's starting rules* (for the Act 1 playtest, 1064):
- The lesson asks for two things: open Arts, then attack with an art (a
  weapon active from the Arts menu counts too). A plain attack ticks
  nothing.
- Vix's text: "An art is an attack, not a buff: one fight, every strike
  boosted. It costs weapon wear." It no longer says "or uses per
  battle".
- The note under the Arts menu starts with `Attack:` for every line that
  attacks, e.g. `Attack: +10 hit; no counter`.

- Engine friction: estimate 70 production lines (the ticket's 60-100,
  judged by `ArtsOpened`'s own notice and doable rule), measured 43: under 2x.
