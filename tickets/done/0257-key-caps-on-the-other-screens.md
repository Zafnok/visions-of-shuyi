---
id: "0257"
title: "Key caps on the other screens: shops, Preparations, saves, town, world map, class change and the rest"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0254"]
nick_input: none
completed: 2026-11-18
---

# 0257 — Key caps on the other screens

## Context

0254 turned on the key-cap style Nick decided in 0250
(`docs/design/look-and-feel.md`, *Key names in hints*) and switched the
battle's help bars and toggles (with its Guides, rewind and shop
screens, which the town shares), the text box's corner, the tip, lesson
and end-of-lessons boxes to styled text. The engine's own screens
(Options, Key bindings, its questions) already draw caps. 0254 split
here, as its *Split* said, because the game names keys in about 270
places across 55 files, far over its estimate of 15.

The tools are in place: `widgets::help::{action_key, key_pair,
help_line_styled, join_hints, text_with_keys, frame_widths,
wrap_styled}`, `tips::fill_styled`, `ctx.text_styled`, and the
`keyboard-input` skill's *Key caps*.

2026-11-17 friction sweep ([report](../../docs/research/friction-2026-11-17.md),
*Help bars are whole sentences per screen state*): the help bars are
whole sentences per screen state. `assets/lang/en/ui.ron` has about 114
help keys, `{Cancel} back` is written out 66 times, and some keys are
combinations of the same hints (`save.help.load`, `load_import`,
`load_folder`, `load_export`, `load_export_folder`, ~:177–181;
`profile.help.list`, `list_export`, `list_export_folder`, `add_import`,
~:57–60). 16 views keep `help: String`. Switching each sentence to
styled text walks every screen once, and 0255, 0256 and 0445 would
walk them again. So this ticket's approach changes: views carry a
**hint list** into `widgets/help.rs`, and the combination keys go.

## Nick input

None: decided in 0250.

## Scope

**In:**
- **Hint lists.** A help bar in a view is an ordered list of hints,
  each `(action or actions, verb text key, shown: bool)` (e.g. a
  `HelpBar` type in `crates/ui/src/widgets/help.rs`), built from
  verb-only text keys (`hint.back`, `hint.import`, …). One function in
  `widgets/help.rs` turns it into the styled line (key caps, the
  separator, fitting the row), on top of `help_line_styled`. A hint the
  state doesn't allow is `shown: false`, not a second text key.
- The combination keys go: one hint list per screen state replaces
  `save.help.load*`, `profile.help.list*` and `profile.help.add*`, and
  every other help key that only differs by which hints are shown. The
  test pack loses them too (or 0258's derived pack does).
- Every other view whose text names a key for the player switched
  to `StyledText` and painted with `print_styled`:
  - Preparations and its shop (`screens/preparations*`), the world map
    (`screens/world_map*`), camp, the save and load screens and the save
    import (`screens/save*`, `save_import.rs`), class change
    (`screens/class_change*`, with `progress::help`), results, game over,
    title, lead select, mode select, familiarity, credits, profiles, the
    voices question, the dialogue log;
  - in the battle: the forecast's `Left/Right weapon` hint
    (`ForecastView::swap`), the art, skill, magic, item and map menus'
    hints, the objective page, the info screen and the EXP pages
    (`progress.rs`), the battle notes;
- each view's test lists its key spans (as `tests/it/key_caps.rs`
  does), and a snapshot of a few with `Harness::with_key_caps`.

**Out:** the battle's own help bars, already styled by 0254: move them
onto hint lists only where a screen state's bar is a combination key;
otherwise either order with a later sweep. The save screens' own
actions (0259): either order, whichever lands second names the other's
actions. The debug tools (plain, as the engine leaves them); a
controller's buttons (0255) and the Playdate (0256); the engine's
styled word wrap (`widgets::help::wrap_styled` is the game's workaround
until the engine has one).

## Implementation steps

1. `grep -rn "key_name\|all_key_names\|cursor_keys_name\|help_line\|text_with" crates/ui/src`
   (not tests or `debug/`), less what 0254 switched.
2. Add the hint list and its painter to `widgets/help.rs`, with tests,
   then the verb-only keys in both language packs.
3. Switch each view to a hint list (help bars) or styled text (other
   text naming a key), its skin and its tests (`plain()` where a test
   compares words). Delete each combination key it replaces.
4. Snapshots, `cargo shuyi frame-png` of Preparations and the world map,
   the gates.

**Expected size:** about 700 production lines, plus tests and snapshots
(the hint list adds about 80 and removes the combination keys' text).
**Split:** past 2,000 changed lines, ship Preparations, the world map and
saves, and move the rest to a new ticket.

## Acceptance criteria

- [x] No view a player sees on the PC names a key in plain text: the grep
      of step 1 finds only `.plain()` for tests, the Playdate's skins and
      the debug tools.
- [x] A test per switched view finds the keys `key_name` gives, in order.
- [x] `grep -n "help.load_\|help.list_\|help.add_" assets/lang/en/ui.ron`
      finds nothing, and no view outside the battle, the debug tools and
      the story preview has a `help: String` field.
- [x] The Load Game list on the web (no saves folder to open) gives the
      desktop's hint list with the folder hint `shown: false`, not
      another text key (test).
- [x] With a controller in use, hints read as today.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the hint list's painter (shown and hidden hints, the separator,
  fitting); each switched view's key spans.
- Snapshot: Preparations and the world map with key caps.

## Completion notes

Done on 2026-11-18, in one PR (no split). The friction sweep of
2026-11-17 rewrote this ticket (hint lists) while its PR was open: the
key spans below were done first, the hint lists after.

- **Hint lists** (`widgets::help::HelpBar`): an ordered list of hints,
  each `(text key, shown)`, whose `line` joins the shown ones with the
  separator. One deviation from the scope's sketch: a hint is one text
  key holding its own key and verb (`hint.back`: `{Cancel} back`), as the
  battle's `battle.hint.*` already are, not an action plus a verb-only
  key, so a language can put the key after the verb. Twelve shared
  `hint.*` keys (English and the test pack). The combination keys are
  gone: `save.help.load*` (the Load Game list's `load_bar`: export,
  import and open folder shown as the state allows), `profile.help.
  list_export*` and `add_import` (`ProfileScreen::help_bar`, over the
  engine's own `profile.help.list`/`add`, which stay), and
  Preparations' `prep.help.tabs*`/`fight*` (Leave shown when the
  battle can be left). The other screens' help keys each name one
  state's verbs, not a combination, so they stay one styled key each;
  moving every bar onto `hint.*` keys is for a later sweep, as the
  scope allows for the battle's. *Fitting* is the engine's: a line too
  wide is cut at the row by `print_centred_styled` (no hint is dropped).

- **Switched to key spans**, each view's help line a `StyledText` built
  with `ctx.text_styled` (or `key_pair`, `text_with_keys`, `join_hints`
  where the line is built from parts) and painted with the engine's
  `print_centred_styled` or `print_styled`: title, mode select,
  familiarity, lead select (and its text box's fixed keys, now the
  engine's styled `text_keys_help`; the game's own copy is gone), the
  voices question, the save question and the save/load slots, the save
  import, game over, "To be continued" and the demo's end, credits, camp,
  the world map (both skins), Preparations, class change, results, the
  dialogue log, and the writer's preview's names page. In the battle: its
  yes/no questions (the battle's dialog boxes now take styled lines) and
  the forecast's `Left/Right weapon`. The rest the ticket listed in the
  battle (the menus' hints, objective page, info screen, EXP pages,
  notes) already show their keys only on the help bar 0254 switched.
  The Profile screen's own lines and the engine's questions (`Ask`) were
  already styled.
- The title's help line fades with the menu, its caps too; over the
  cinematic it keeps no strip behind its words, as before.
- **Playdate skins** read `.plain()`, as the ticket says.
- **Tests:** `widgets::help::{key_spans, spans_of}` (test and harness
  builds only) read a hint's keys and what `key_name` names for a list of
  actions; each switched view has a test that they agree, in order.
  `tests/it/key_caps.rs` adds Preparations and the world map (the test
  world map) with caps, as snapshots, and Preparations on a controller
  (plain text, no key spans).
- **Snapshots:** about 70 changed only in the key cells' colours (the
  palette's `key` and `key_bg`, as in 0254). About ten from tests
  that build the real game's context (key caps on) now draw the caps:
  the save slots and question, the title, the world map's help line.
  Two private ones (`it__world_map__act1_start`,
  `it__title_cinematic__title_cinematic_12_logo`) in
  `Zafnok/visions-of-shuyi-assets`, its PR linked in this PR, branched
  from the current pin (the private `main` has 0490's content, whose game
  PR is still open). One test (`save.rs`) read the slots' frame text and
  now reads the caps.
- **Picture:** `docs/screenshots/0257-preparations.png` (`cargo shuyi
  frame-png --keys "Down f"`, the Quick Battle's test units). The world
  map was not rendered with `frame-png`: its New Game plays the private
  story, whose map and scenes may not be committed (ADR-0118); the
  snapshot `it__key_caps__the_world_map_draws_its_keys_as_caps` shows the
  test world map's help line with caps instead.
- **Left plain:** the debug tools, as the ticket says; the battle's
  `help()` (its words, for tests).

Size: estimate 250 production lines (about 8 a screen for 20 screens and
the battle's two hints, judged by 0254's town and rewind switches);
measured 248 added, 131 removed (`code-lines`), 1.0×, before the hint
lists; with them 316 added, 161 removed against 330 (the ticket adds
about 80 for the hint list), 1.0×. PR size 1,234 changed lines.

*Claude's starting rules:* none (no gameplay).

For the Act 1 playtest (1064): whether the caps read well on the menus'
centred help lines.
