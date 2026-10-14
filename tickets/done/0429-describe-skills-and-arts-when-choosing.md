---
id: "0429"
title: Show what a skill or art does while choosing it in battle
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-14
---

# 0429 — Show what a skill or art does while choosing it

## Context

Nick: "we need this info readily available when selecting in battle. It's a
nuisance to have to know each skill by name."

Today a skill's effect is written out in one place only: the unit info
screen's *Skills* block (0412), one line per skill from
`effect_text(&SkillDef)` in `crates/ui/src/screens/battle/skills.rs`. The two
places where the player actually *chooses* one show only names:

- **Skill menu** (action menu → `Skill`, 0412): `skill_menu` in
  `skills.rs` draws `Brace     3/3` (its uses left this battle, ticket 0316). No effect.
- **Arts list** while targeting (0414): `art_menu` in
  `crates/ui/src/screens/battle/art_list.rs` draws
  `Guard Break     −4 dur  D`. No effect. The forecast shows the chosen
  line's non-number notes (`pierces`, `pins: Mov −3`, from `note_text`), but
  not its hit/crit/damage changes or what it's for in words.

The spell menu (0410, not built yet) has the same gap in its plan; this
ticket also adds the requirement there (see *Scope*).

Relevant: `docs/design/look-and-feel.md` (layout and colours are Nick's),
`docs/design/combat-arts.md`, `docs/design/progression.md` (*Skills*),
ADR-0004 (only `ui`/`app` draw; text comes from `core` data).

## Nick input

**Layout first (look & feel is Nick's call, never pick it yourself):** before
coding, use the `ascii-art` skill to render at least three real mockups of
the Skill menu and the arts list with a description shown for the
highlighted line, and ask Nick with the `ask-nick` skill. Example options
(drawn from real games; add others if they fit):

1. **Description line under the list** (Fire Emblem: Three Houses' arts
   list: a text strip for the focused art).
2. **Side box beside the list** (Fire Emblem Engage / Fates skill
   window: name, cost and effect in a small panel next to the list).
3. **Effect in the help bar** at the bottom of the screen (Advance Wars /
   Into the Breach-style one-liner for the hovered item).
4. Describe your own.

Record the answer in `look-and-feel.md` (a *Descriptions in lists* rule, worded
as a player situation with an example, per Nick's preference) so later menus
(0410 spells, 0408 preparations, 0603 promotion) follow it.

**Sign-off:** in Quick Battle, open the lord's and the knight's `Skill` menu
and the archer's and lord's arts list; move through the lines. Can you tell
what each one does without leaving the menu?

## Scope

**In:**
- A description for the **focused** line of the Skill menu (skill effect).
- A description for the **focused** line of the arts list: `Attack`
  (plain attack), each Combat Art, and each combat active.
- An art effect text function (arts have none yet), same style as
  `effect_text` for skills.
- The description updates as the focus moves; dimmed lines still show
  theirs (so the player learns what they can't afford).
- The `look-and-feel.md` rule from Nick's answer.
- One line added to 0410's *Scope → In* and step 1: the spell list shows the
  focused spell's description per the `look-and-feel.md` rule. **If 0410 is
  already in `tickets/done/`** (the two tickets don't block each other),
  write a follow-up ticket instead (`write-ticket`) that adds the
  description to the spell list.

**Out (do not do):**
- Rewording the info screen's Skills block (it already has effect lines),
  unless Nick's chosen layout asks for the same text there.
- Flavour/lore text or hand-written descriptions per skill in data: build
  the text from the effect data (so it can't drift from the rules). If Nick
  wants written descriptions, write a follow-up ticket.
- Item/consumable menus (0407 already shows `effect_text` for consumables).
- Spell menu code (0410).
- Any rule or number change.

## Implementation steps

1. Ask Nick for the layout (see *Nick input*); record it in
   `docs/design/look-and-feel.md`. Do not start step 3 before the answer.
2. **Art text:** in `art_list.rs`, add `pub fn art_effect_text(art: &ArtDef)
   -> String` built from `ArtEffect` (read `crates/core/src/art.rs` and the
   field list at the top of `assets/data/arts.ron`): number changes first
   (`+20 hit`, `Mt +3`, `eff. vs Armored ×3`, `min range 2`, sword follow-up,
   axe min damage), then the existing `note_text` notes, joined with `; `.
   Reuse `mods_text`/`timed_text`/`stats_text` from `skills.rs` where they
   fit. Add `Technique::effect_text(&self, state)` returning the art text, the
   skill's `effect_text`, or `plain attack` for `Attack` (check the exact
   variant names in the `Technique` enum at the top of `art_list.rs`).
3. **Draw it** where Nick chose, for the focused line of:
   - the arts list (`draw_list` in `art_list.rs`; the focus is the `Menu`'s
     focused index),
   - the Skill menu (the call site is in `mode.rs` near
     `skill_menu(state, sel.unit, &choices)`; `SkillChoice` holds the skill).
   Text colour: `UiColor::TextDim` like the info screen's effect line; cut
   long text with `pen.cut`/the existing truncation helper rather than
   overflowing the box. Must fit the right-handed and left-handed layouts
   (0208) and not cover the attacker or the target (see `list_origin`).
4. If the chosen layout puts text in the help bar, follow the
   `keyboard-input` skill (key names from the keymap, never literals).
5. Edit `tickets/open/0410-*.md` as described in *Scope*, or write the
   follow-up ticket if 0410 is already done.

## Acceptance criteria

- [ ] `look-and-feel.md` has the *Descriptions in lists* rule with Nick's
      choice and an example.
- [ ] Unit test `art_effect_text` for every art in `arts.ron` (each returns a
      non-empty string; spot-check at least three exact strings, e.g. an art
      with `no_counter`, one with `on_first_hit`, one with `effective`).
- [ ] Harness: in the arts list, moving Down changes the description to the
      next line's text; same for the Skill menu (Brace → `self: Def +5 Res
      +5`).
- [ ] Harness: a dimmed line still shows its description when focused
      (if the list lets focus reach it; otherwise document why not).
- [ ] Snapshots: arts list with description (archer and lord), Skill menu
      with description, both in the left-handed layout too.
- [ ] 0410 mentions the description in its scope and step 1 (or, if 0410
      was already done, a follow-up ticket for the spell list exists).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `art_effect_text` strings; `Technique` effect text for `Attack`,
  an art and an active.
- Snapshot / integration: harness focus changes and the snapshots above.

## Note: ticket 0442 is done (2026-10-12)

Ticket 0442 moved the Combat Arts out of the forecast into their own menu
(`docs/design/combat-arts.md` → *The arts menu and the forecast*). So in
this ticket:

- "The arts list" is the **arts menu**: `Arts` in the action menu
  (`Mode::ArtsMenu`, lines from `art_lines`, drawn by `arts_menu` in
  `art_list.rs` and by `draw_menu` in `mod.rs`). It has no `Attack` line,
  and there is no `draw_list` any more.
- Spell actives are **lines of the spell list** (`Fire + Overcast`,
  `SpellChoice::active` in `magic.rs`): describe the active on those lines.
- The sign-off's "archer's and lord's arts list" is their `Arts` menu.

## Nick again (playtest round 2, 2026-10-14)

> "I see still no helper text for i.e. Brace what the hell does it do I
> thought we already took care of this ticket but it still tells me nothing
> about what it does when I hover over it in skill menu about to select it"

He thought this was done; it isn't. **The playtest waits for it**
(0804's `blocked_by`). Brace is the example to check first in the sign-off.

## Completion notes

**Done (2026-10-14).** Nick saw four real renders (A strip in the menu
box, B window under the list, C help bar, D side panel) and chose "A but
with the more pronounced coloration of B": the effect is a row under a
rule inside the menu box, in white, the name not repeated. He also chose
that the cursor stops on greyed-out lines. Recorded in
`look-and-feel.md` → *Descriptions in lists*.

- `MenuItem::with_note` and `Menu::browsing_disabled` (`widgets/menu.rs`);
  the glyph skin paints the focused item's note under a rule, the box as
  wide as its widest note so it doesn't jump as the focus moves, and a
  focused disabled line as a muted bar (`panel_border`).
- `art_effect_text` and `Technique::effect_text` (`art_list.rs`); the arts
  menu and the `Skill` menu give every line its note and browse disabled
  lines (Confirm on one still does nothing).
- Tests: every art's text (non-empty, ≤ 40 characters, exact strings for
  all ten), every skill's text ≤ 40 characters, `Technique` texts, the
  menu and skin, harness tests that the note follows the focus in both
  menus and onto a dimmed line (`the_skill_menu_describes_the_focused_line_dimmed_or_not`:
  spent Brace reads `self: Def +5 Res +5`), snapshots of the lord's, the
  archer's and the left-handed arts menu and of the Skill menu.

**Deviations**
- Text colour is `text` (white), not `text_dim` as step 3 said: Nick's
  choice.
- Line Pierce's description reads `then strikes the unit behind` instead
  of the forecast's terse `pierces`, so the description says what it does.
- No `draw_list` / no `Attack` line exists any more (0442); the note is a
  menu item field painted by the menu skin, so `draw_menu` didn't change.
- 0410 was already done: follow-up **0453** (spell list descriptions)
  instead of editing it.

**Claude's starting rules** (veto freely)
- A greyed line under the cursor is a dimmer bar (the border colour)
  rather than the gold one.
- The `Arts` / `Skill` entries of the action menu are still greyed when
  nothing in them can be used, so you can't open the list just to read it.

**Follow-ups:** 0453 (spell list descriptions), 0454 (a menu too wide for
either side of its unit covers it: seen on the archer's Arts menu; it was
already so before this ticket).
