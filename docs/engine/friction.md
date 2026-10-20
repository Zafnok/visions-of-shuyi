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
| 2026-10-19 | 1159 | the battle's rewind screen and Guides list as screens on the stack (the rewind pops with its effect, `covered`/`uncovered` for the tips' flag) | 150 | 311 | 2.1× | accept: the battle lost the 130 lines the sweep counted, but each pop-up now pays what every screen does (a `Screen` impl, a whole-screen view and its painter, ADR-0054), and the rewind screen carries the battle's camera and look to paint its own map (`MapFrame`); `trpg-ui` rose by 67 net. Not the engine's: a screen that shows another screen's map with another state is this game's alone | — |
| 2026-10-19 | 1161 | the battle's five lists as one `Mode::List` with a `ListKind`, its five target pickers stepped by one `step_target` over a `Picker` trait | 100 | 313 | 3.1× | 4: accept, a game-side refactor with no engine glue (production lines fell by 151); a changed line counts as added and most added lines are match arms rewritten for `Mode::List` (464 removed). Next time a refactor's estimate counts the lines it rewrites, not only the new ones | — |
| 2026-10-19 | 1162 | the battle's menus carry their values (`Menu<MenuEntry>`, `Menu<MapEntry>`, `Menu<UnitId>`, `Menu<Line>`) and the action and map menus hold text keys; the pick-one, title and reply menus carry their values; the pin moved past engine 2308–2311 | 30 | 313 | 10.4× | 4: accept, no engine glue (production lines fell by 68, 381 removed). The estimate counted only new lines again, though every list builder, target picker and menu match changes shape (the 1161 note, not applied). A list of data names (arts, spells, items, units) can't hold keys, since a key names only screen text, so the lists are written in the player's language when they open and `step`/`tick` take the language (about 20 lines); an engine label resolved by the game would remove that, too small to be worth a ticket now | — |

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
