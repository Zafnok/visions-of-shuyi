# Engine friction log

One row per friction review or 2×–3× note (ADR-0092, the
`engine-friction` skill): what the job was, how big we expected it to be,
how big it was, and what we did about it. Newest last. Three or more
"accept" rows on the same engine area trigger a review; sweeps (1153)
read this table.

| Date | Ticket | The job | Estimate | Measured | Ratio | Outcome | Follow-up |
| ---- | ------ | ------- | -------- | -------- | ----- | ------- | --------- |
| 2026-10-18 | 1151 | count production Rust lines per crate in a diff (`cargo xtask code-lines`) | 180 | 312 | 1.7× | under 2× (the log's first row, asked for by the ticket; not a review) | — |
| 2026-10-19 | 1153 (sweep) | run one battle on screen: commands, AI, fights, banners, tips, lessons, rewind, help (`screens/battle/mod.rs`) | 1700 | 2009 | 1.2× | 1: a screen's result on pop and a resumed hook (the rewind and Guides overlays are fields, not screens; `set_under_scene` rebuilds "uncovered"); 3: the overlays become screens | engine#2306, 1159 |
| 2026-10-19 | 1153 (sweep) | load, check and play authored timed shots: pans with walks and strikes, logo, talk, faces, a fight (`ui/cinematic*`, `content/cinematic*`) | 1050 | 1614 | 1.5× | 1: word wrap in the assets crate with a fits-the-box check (`page_lines` copies the engine's wrap); 1: a folder loader on the bundle (8 copies in `content`); 4: named action lists (one user of the engine's cinematic file so far); 4: an action shot repaints the whole map each frame (unmeasured; moving items over a cached backdrop is parked 1120) | engine#2208, engine#2209, 1169 |
| 2026-10-19 | 1153 (sweep) | show the units on the kit's grid scene in three looks (`map_view*`) | 750 | 1266 | 1.7× | 2: the kit's grid skin lacks zoom, a held walk speed and a per-frame painter, so the game has its own `MapSkin` trait; the one-bit map skin copies the glyph skin's paint order, tiles and terrain because the glyph skin has no style options; 4: a sprite outline is four sprite items (PC skin only, no budget missed) | engine#2313, 1167 |
| 2026-10-19 | 1153 (sweep) | the move loop as pure transitions: one key in a mode gives the next mode and an effect (`screens/battle/mode.rs`) | 925 | 1243 | 1.3× | 3: five list menus written as five open/step pairs and three near-copy target pickers (one list mode, one picker); 1: menu items that carry their value and labels translated when painted (`told()` and `IN_PLAY` re-translate because `MenuItem.label` is a baked `String`; 7 parallel lookup lists) | 1161, engine#2308, 1162 |
| 2026-10-19 | 1153 (sweep) | play a scripted conversation: portraits, typed text, replies, a place behind it, a backlog (`screens/dialogue*`, `ui/dialogue.rs`) | 700 | 1098 | 1.6× | 1: a scroll window (the log is one of 6 copies); 2: the text box's confirm and hold-to-speed step; 4: the player newtype and the owned talk box (about 100 lines, cheaper kept) | engine#2307, engine#2312, 1160, 1166 |
| 2026-10-19 | 1153 (sweep) | edit loadouts in three nested lists, move items between stock and pack, start the battle (`screens/preparations*`) | 640 | 981 | 1.5× | 1: a list cursor with wrap and a scroll window (one of 13 copies); 1: a question widget (one of 6 copies); 4: per-frame row rebuilds and a clone of the whole preparations (unmeasured on the Playdate) | engine#2307, engine#2309, 1160, 1163 |
| 2026-10-19 | 1153 (sweep) | list, use, rename, delete and add player profiles (`screens/profile*`) | 100 | 831 | 8.3× | 1: the screen names no game concept: it and the name entry widget move into the engine as options and Key bindings did (it also copies the engine's private question painter and `stepped`); the estimate is the game's 20-line opener, like `key_bindings.rs`, plus its 80-line one-bit skin | engine#2310, 1164 |
| 2026-10-19 | 1153 (sweep) | compare promotion or reclass targets side by side and confirm (`screens/class_change*`) | 430 | 759 | 1.8× | 1: list cursor and question (engine#2307, engine#2309); 3: a 42-line debug test unit lives in the screen | engine#2307, engine#2309, 1160, 1163, 1171 |
| 2026-10-19 | 1153 (sweep) | turn level-up and class events into timed, skippable pages (`screens/battle/progress*`) | 570 | 679 | 1.2× | 2: the frame time isn't promised finite and positive (13 cleanups in the game, 4 in the engine) and there is no clock a held key speeds up and a press skips (3 copies: progress, playback, AI phase) | engine#2312, 1166 |
| 2026-10-19 | 1153 (sweep) | a save prompt, a 30-slot save and load picker that asks before overwriting, save files (`screens/save*`, `ui/save.rs`) | 410 | 675 | 1.6× | 1: list cursor, question, a pick-one screen that pops with its answer (engine#2307, engine#2309, engine#2306); 4: a save-slots module (about 70 generic lines with one user) | engine#2306, engine#2307, engine#2309, 1158, 1160, 1163 |
| 2026-10-19 | 1153 (sweep) | lay out the battle's map, panel, menus, dialogs, banner and bars (`screens/battle/glyph*`, `view*`) | 500 | 651 | 1.3× | 1: a titled box, a meter bar (7 copies) and text centred or cut to a width (16 copies); 3: two helpers re-write the engine's `print_fg` and `cut` | engine#2311, 1165 |
| 2026-10-19 | 1153 (sweep) | an attract-mode title: menu, idle fade, a cinematic timed to the song, skip (`screens/title*`) | 560 | 636 | 1.1× | 1: the `stale`/`refresh` workaround for a missing resumed hook (engine#2306); 4: `song.rs` follows the song by the clock because the audio backend can't seek (43 lines; ADR-0037) | engine#2306, 1158 |
| 2026-10-19 | 1153 (sweep) | take New Game or Load through the chapters, Game Over, Retry and Suspend (`flow.rs`) | 390 | 539 | 1.4× | 1: the flow hosts its child screens in an 11-stage enum and polls each one's `result()` because a pop carries none (8 screens store a result); 3: two copies of the campaign settings kept in step | engine#2306, 1158, 1171 |
| 2026-10-19 | 1153 (sweep) | a full unit sheet (`screens/battle/info*`) | 490 | 537 | 1.1× | under 2× (its meter bar is one of the copies engine#2311 covers) | engine#2311, 1165 |
| 2026-10-19 | 1153 (sweep) | the engine's options screen with the game's rows, shown by platform and campaign (`screens/options*`) | 300 | 519 | 1.7× | 2: an enum setting needs a `Named` trait, 5 impls and a lookup (84 lines) because `RowValue::Choice` takes plain function pointers; 3: test-only copies of engine row helpers | engine#2314, 1168 |
| 2026-10-19 | 1153 (sweep) | two fighters' pictures moved over a backdrop with name plates (`screens/battle/stage*`) | 430 | 495 | 1.2× | under 2×; 3: `cells_px` copies the kit's `px_rect`; its HP bar is a meter copy (engine#2311) | 1165 |
| 2026-10-19 | 1153 (sweep) | the Playdate front end: start the game with the device's console, buttons, storage, sound and pictures (`crates/playdate`) | 100 | 1735 | 17× | 1: the backend, mostly held by engine 3003 and 3010 (several PRs: why not fixed first); 1: the game runs on a heap stack of its own (15 KB used, about 10 KB on the device), no ticket held it; 1: the audio player is private to the engine's app crate, so the game re-wrote it, and Playdate audio was only a "write it later" line; 3003 and 3011 gained the loading phase and the on-device report; 4: choosing a screen's skin by target (`Skins`, 1 to 4 lines a screen; the one-bit skins are the game's look; revisit with engine 3001 and 3010) | engine#3028, engine#3029, 1170 |
| 2026-10-19 | 1160 | the game's lists on the engine's `ListCursor` (save, class change, Guides hold one; the lists that keep a plain index step through `wrap_step`; four scroll windows) | 30 | 82 | 2.7× | accept: a changed line counts as added, and most are `self.focus` becoming `self.cursor.focus()` (130 lines removed); the Playdate's centring rounds the other way from `Scroll::Centred` on even windows, so it stays | — |
| 2026-10-19 | 1158 | the game flow and the title on the engine's screen results (pop with a value, `uncovered`), its eight pick-one screens and the battle's Game Over moved onto the stack | 200 | 313 | 1.6× | under 2× (production lines fell by 96); 4: a pushed screen hears nothing until its first update, so the flow pushes its first screen a frame after it is pushed, naming and drawing it that frame (6 lines, invisible to the player; an engine call on push would remove it) | — |
| 2026-10-20 | 1169 | six content folder loops on the engine's `load_each`, and the cinematic page check on `lines_at_longest` | 30 | 71 | 2.4× | accept: production lines fell by 111; a changed line counts as added; the chapter and battler loaders check the files that parsed together (`check_next`, the default battler) even when another file broke, which `load_each` (all or nothing) can't give, so the game keeps a 17-line `bundle::parsed` and its two users report each file's errors themselves. A third such loader would make it an engine variant that keeps what parsed | — |
| 2026-10-19 | 1159 | the battle's rewind screen and Guides list as screens on the stack (the rewind pops with its effect, `covered`/`uncovered` for the tips' flag) | 150 | 311 | 2.1× | accept: the battle lost the 130 lines the sweep counted, but each pop-up now pays what every screen does (a `Screen` impl, a whole-screen view and its painter, ADR-0054), and the rewind screen carries the battle's camera and look to paint its own map (`MapFrame`); `trpg-ui` rose by 67 net. Not the engine's: a screen that shows another screen's map with another state is this game's alone | — |
| 2026-10-19 | 1161 | the battle's five lists as one `Mode::List` with a `ListKind`, its five target pickers stepped by one `step_target` over a `Picker` trait | 100 | 313 | 3.1× | 4: accept, a game-side refactor with no engine glue (production lines fell by 151); a changed line counts as added and most added lines are match arms rewritten for `Mode::List` (464 removed). Next time a refactor's estimate counts the lines it rewrites, not only the new ones | — |
| 2026-10-19 | 1162 | the battle's menus carry their values (`Menu<MenuEntry>`, `Menu<MapEntry>`, `Menu<UnitId>`, `Menu<Line>`) and the action and map menus hold text keys; the pick-one, title and reply menus carry their values; the pin moved past engine 2308–2311 | 30 | 313 | 10.4× | 4: accept, no engine glue (production lines fell by 68, 381 removed). The estimate counted only new lines again, though every list builder, target picker and menu match changes shape (the 1161 note, not applied). A list of data names (arts, spells, items, units) can't hold keys, since a key names only screen text, so the lists are written in the player's language when they open and `step`/`tick` take the language (about 20 lines); an engine label resolved by the game would remove that, too small to be worth a ticket now | — |
| 2026-10-19 | 1163 | the save, class change, dialogue and preparations questions held as the engine's `Ask` (the save's box painted by its skin) | 30 | 154 | 5.1× | 4: accept, no engine glue (178 lines removed); most added lines are rewritten match arms (the dialogue's skip block moved out a level). Three of the four questions keep their own look (a third line, the message row, the text box), so only the save's painter went. Next time the estimate counts the lines a move rewrites | — |
| 2026-10-20 | 1165 | the game's glyph skins on the engine's `panel`, `meter`, `print_centred` and the kit's `px_rect` (7 meters, 6 boxes, 5 centred labels, 30 cut lines) | 40 | 157 | 3.9× | 4: a changed line counts as added (163 removed): 30 calls of `put`/`panel_line` renamed one for one, several split over lines by rustfmt; 3: in this PR, `meter_cells` (the HP and EXP bars round by their own tested rules, half up and down, which `meter_fill`'s edge rule would change) and `print_clipped` (the PC skins cut hard, the engine's `print_cut` adds `…`, a look change); estimate a sweep of call sites by the call sites, not the helpers removed | 0068 |
| 2026-10-20 | 1167 | the battle map in three looks on the kit's grid skins (engine 2313): the game's `MapSkin` trait, its forwarding skins, the one-bit skin's copied paint order and `RangeKind` go; the pin moved past engine 2312 too | 40 | 208 | 5.2× | 4: accept, no engine glue (404 removed, trpg-ui 196 smaller). About 100 added lines are call sites renamed one for one (`Rc<dyn MapSkin>` → `MapSkin`, `RangeKind::Move` → `range::MOVE`, `input.dt` → `input.dt()` for 2312); the rest is the `MapSkin` struct (a kit skin and which unit painter, ADR-0094), the one-bit painter moved out of the old `paint`, and the range constants. The estimate counted new lines only, the 1162, 1163 and 1165 lesson again: count the call sites a rename touches | — |
| 2026-10-20 | 1173 | the game's loaders read through the engine's format registry: the registry built once with the game's formats, the data tables as kinds, maps, cinematics and dialogue read file by file | 150 | 208 | 1.4× | under 2× (138 removed); the signal is one workaround: `formats::load_in`, because `Formats::load_under` gives nothing when one file fails (hiding the checks of the others) and reads subfolders. 2: engine ergonomics, `Formats::load_each`, shipped with the workaround marked; also 3: in this PR, `Table<T>` and `DataTable`, since a `trpg_core` table can't be an `AssetKind` itself (orphan rule) | engine#2219, 1176 |
| 2026-10-20 | 1171 | one copy of the campaign's mode, tutorial level and lessons, and the class change screen's debug unit moved to the debug tools | 15 | 49 | 3.3× | 4: accept: 42 of the 49 are the debug unit moved verbatim (a moved line counts as added; production lines fell by 7); next time a move goes in the estimate | — |
| 2026-10-20 | 1176 | the map, cinematic and dialogue loaders on the engine's `Formats::load_each`; the game's `formats::load_in` goes | 3 | 9 | 3× | 4: accept, no engine glue (14 removed): the three call sites, which rustfmt splits over lines now that the call names the registry and the bundle, and one import. The workaround 1173 marked is gone | — |
| 2026-10-22 | 0506 (sweep) | play a battle's bots many times on every core and report against the targets, with a history and summary lines (`crates/xtask/src/playtest.rs`) | 60 | 532 | 8.9× | 1: the engine's `shuyi playtest` takes a finished report from the game, so it can't spread tries over threads, time them, keep a history or print lines from measures; the game wrote its own command | engine#2532, engine#2533, 1180 |
| 2026-10-22 | 0506 (sweep) | turn the targets file into a battle's checks by its tier (`crates/bots/src/playtest.rs` `Targets`) | 5 | 46 | 9.2× | 1: targets by tier (success rules, bands per tier, floors, medians) | engine#2129, 1180 |
| 2026-10-22 | 0506 (sweep) | score a planned turn as named counts times a persona's weights, personas read from a file (`persona.rs` `Weights`, `Persona`, `Tally`, `Tally::score`) | 35 | 108 | 3.1× | 1: weighted features and personas as data (each new count is written in four places) | engine#2130, 1180 |
| 2026-10-22 | 0506 (sweep) | the danger each player unit stands in, as the danger zone and forecasts show it, and how far each walks to the objective (`persona.rs` `threats`, `danger`, `Goals` and helpers) | 40 | 130 | 3.3× | 1: threat areas, the danger at a tile from a forecast hook and a walking distance to goals, in the tactics kit; 3: the caps, `at_risk` and the lord rules stay | engine#2131, 1180 |
| 2026-10-22 | 0506 (sweep) | the commands a persona tries first (`persona.rs` `order`, `advance`, `hold`, `attack_key`, `once_per_action`) | 50 | 174 | 3.5× | 1: a candidate order (ranked groups with a kept last command in the core; strikes, advances and holds in the tactics kit); 3: the classifier stays | engine#2132, 1180 |
| 2026-10-22 | 0506 (sweep) | run every battle's playtest in CI and fail only on a floor (`.github/workflows/playtest.yml`, YAML, not counted by `code-lines`) | 15 | 94 | 6.3× | 1: a reusable workflow | engine#2021, 1180 |
| 2026-10-22 | 0506 (sweep) | what a try measures (`measures.rs`, `BattleMeasure`) | 110 | 117 | 1.1× | 4: accept, the game's events and modes (what a fall is, which items count); the turn cap's off-by-one is 3 lines | — |
| 2026-10-22 | 0506 (sweep) | an entrant: the battle in its mode, the persona on the Player phase, the enemy AI elsewhere (`playtest.rs` `BattleEntrant`, `BattleStage`); the talk rule (`PersonaBot`) | 80 | 108 | 1.4× | under 2× | — |
| 2026-10-22 | 1039 | the Playdate's forecast, unit info, fight box, EXP and level-up pages and rewind screen: five paints of existing views, and the views' few new fields | 450 | 776 | 1.7× | 1: a crank step in the engine's frame-png script (the game added `--crank` and `--then-pad` to its own options), follow-up | engine#2540, 1181 |

## Sweeps

A sweep measures the whole game once, not a diff: the `engine-friction`
skill, *Sweep*, says when one runs and how. Its rows are in the table
above with `(sweep)` after the ticket; the next sweep compares each
candidate's measured lines with its row here.

### 1153, 2026-10-19: the first sweep

**Candidates**: the 15 largest screens, skins, scenes and flows by
production lines, plus the combat picture stage and the Playdate front
end (ticket 1153 named those; the dialogue screen, the map view, the
cinematics and the options screen were in the 15 already). Measured on
`main` at `eb93fc89` with the rules of `cargo xtask code-lines`, per
file, summed per screen (its logic, view and skins). The Playdate count
leaves out `chapter_1.rs`, a test module that `code-lines` counts
because its name isn't `tests`, and the 127 lines of `shim/shim.c`.

**The estimates** were written by readers who had seen the code, from
the job and a reference (the engine's options screen, 924 lines; the
game's 27-line `key_bindings.rs` over the engine's screen; the game's
66-line `crates/app/src/main.rs`; Ren'Py's `say` and history screens).
A sweep's estimates lean toward the code more than a ticket's do, so
read 1.5× here as "about right".

**Findings, ranked** by ratio × recurrence: the ratio of the candidate
it was found in (the highest, when it was found in several, not
counting the profile screen, which engine#2310 replaces whole), times
how many screens or loaders would shrink.

| Rank | Finding | Outcome | Ratio | Shrinks | Score | Tickets |
| ---- | ------- | ------- | ----- | ------- | ----- | ------- |
| 1 | A list cursor (wrap, skipping disabled rows, a clamp after a removal) and a scroll window: 13 hand-written cursors and 6 scroll windows by 4 rules | 1: engine | 1.8× | 15 | 27 | engine#2307, 1160 |
| 2 | The Playdate backend's gaps no engine ticket held: the game's own stack, the audio player, the loading phase and the on-device report | 1: engine (3003, 3010 and 3011 hold the rest) | 17× | 1 | 17 | engine#3028, engine#3029, 1170 |
| 3 | A screen's result when it pops, and a hook when the screen above it pops | 1: engine | 1.6× | 10 | 16 | engine#2306, 1158, 1159 |
| 4 | A frame time promised finite and not negative, and a clock a held action speeds up and a press skips | 2: engine | 1.6× | 8 | 12.8 | engine#2312, 1166 |
| 5 | A folder loader on the bundle: parse every file of a folder, collect the errors | 1: engine | 1.5× | 8 | 12 | engine#2208, 1169 |
| 6 | Text-UI drawing: a box with a title in its border, a meter bar, text centred or cut to a width | 1: engine | 1.3× | 8 | 10.4 | engine#2311, 1165 |
| 7 | A question widget: a line of text and N answers, with focus and an answer for Cancel | 1: engine | 1.8× | 5 | 9 | engine#2309, 1163 |
| 8 | The profile screen and the name entry widget name no game concept | 1: engine | 8.3× | 1 | 8.3 | engine#2310, 1164 |
| 9 | Menu items that carry their value, with labels translated when painted | 1: engine | 1.3× | 4 | 5.2 | engine#2308, 1162 |
| 10 | The kit's grid skin: zoom, a held walk speed, a per-frame painter, and style options for the glyph skin | 2: engine (tactics kit) | 1.7× | 3 | 5.1 | engine#2313, 1167 |
| 11 | Option rows for an enum setting | 2: engine | 1.7× | 1 | 1.7 | engine#2314, 1168 |
| 12 | Word wrap in the assets crate, with a check that a line fits its box | 1: engine | 1.5× | 1 | 1.5 | engine#2209, 1169 |
| 13 | The battle's five list modes and three target pickers | 3: game | 1.3× | 1 | 1.3 | 1161 |
| 14 | One owner for the campaign settings; the class change screen's debug unit moves to the debug tools | 3: game | 1.4× | 2 | — | 1171 |

Found on the way, not friction: the battle forecast's kill mark is drawn
as one rectangle per pixel row, against ADR-0038 (ticket 0470).

### 0506, 2026-10-22: the playtest area

**Why**: not a trigger of the skill. Ticket 0506 (the persona bots)
measured 1.8×, under 2×, but Nick (2026-10-22) asked that the generic
parts of the game's playtesting move into the engine, so other games
get playtest personas and a playtest command out of the box. So this
sweep covers one area only: `crates/xtask/src/playtest.rs`,
`crates/bots/src/{persona,playtest,measures}.rs` and
`.github/workflows/playtest.yml`, measured at 0506's merge
`b5982843` with the rules of `cargo xtask code-lines` (blank, comment and
test lines left out), about 1,300 production lines in all. The
estimates are what the game would keep with the engine pieces, by the
same job in hello-grid (`examples/hello-grid/src/tools.rs`, about 75
lines for its playtest, measures and entrants included) and
0506's own split into the game's logic and the rest.

**Findings, ranked** by ratio:

| Rank | Finding | Outcome | Ratio | Tickets |
| ---- | ------- | ------- | ----- | ------- |
| 1 | Targets by tier: success rules, bands per tier, floors, medians, turned into a battle's checks | 1: engine | 9.2× | engine#2129 |
| 2 | `shuyi playtest` plays a game's plan itself: stages, entrants and groups, threads, defaults, time per try | 1: engine | 8.9× | engine#2532 |
| 3 | The playtest's history (`was …`) and summary lines from named measures | 1: engine | 8.9× | engine#2533 |
| 4 | A reusable playtest workflow | 1: engine | 6.3× | engine#2021 |
| 5 | A planner's candidate order: ranked groups (core), strikes, advances and holds (tactics kit) | 1: engine | 3.5× | engine#2132 |
| 6 | Threat areas, the danger at a tile and walking distance to goals | 1: engine (tactics kit) | 3.3× | engine#2131 |
| 7 | Weighted features and personas as data | 1: engine | 3.1× | engine#2130 |

The game moves onto all seven in one ticket, 1180 (blocked by them).
After it, the game's playtesting should be about 450 production lines
(the counting, the hooks, the measures, the entrants and the plan).
