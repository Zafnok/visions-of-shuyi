# Roadmap

Goal of the first phase: **a playable, story-driven Chapter 1** on Windows and
in the browser, then publish to itch.io, then Steam.

All work is in [`tickets/`](../tickets/README.md). Status is read from the
folders: `tickets/open/` vs `tickets/done/`.

> **The critical path now is Act 1** (2026-10-21, ticket 0129): Chapters
> 1–8 (`docs/story/outline.md`) playable on the web and on the Playdate,
> then **one** playtest of the whole act by Nick (**1064**). It replaces
> Chapter 1 playtest round 4 (0804) and the Chapter 1 play-through on the
> Playdate (1050), both folded into 1064. Nick, the same day: "any tickets
> that say 'check after merge' aren't gonna happen -- I will save all my
> checking for either design (i.e. 00 tickets) or for the full act 1
> playtest itself." So he answers design questions, does setup steps, and
> plays once at the end; the bots (0506, 0509, 0512) check balance. The
> engine carve is done (1118); the plan, in phases, is *Act 1 critical
> path* below.

## Milestones

| Block | Milestone | Outcome | Tickets |
| ----- | --------- | ------- | ------- |
| `00xx` | Design decisions | Nick's answers recorded in `docs/design/` | 0001–0029 |
| `01xx` | M0 Foundation | Workspace, CI on 3 OSes, security scanners, SonarCloud, mutation gate, release + Pages pipelines | 0101–0108 |
| `02xx` | M1 Engine | Coloured glyph console in a window and browser, keyboard input with right/left-handed layouts, screens + test harness | 0201–0208 |
| `03xx` | M2 Core rules | Maps, units, pathfinding, combat, turns, items, rewind, gold/shops, spells, terrain magic, class skills, Combat Arts — pure and heavily tested | 0301–0312 |
| `04xx` | M3 Battle UI | Cursor, move/attack loop, forecast, combat playback, info screens, tips, item menus, preparations, shops, spell menus, battle notes, skill menus, combat scene art, arts menu | 0401–0414 |
| `05xx` | M4 Enemy AI | Enemies that fight back, animated enemy phase, boss arts; automated playtest bots, autobalancing | 0501–0509 |
| `06xx` | M5 Progression | EXP and class points, level-up screen, promotion and reclass, level-up sounds | 0601–0605 |
| `07xx` | M6 Story & dialogue | Story bible, dialogue engine, two-portrait scenes, Chapter 1 art + script, lead reply choices | 0701–0708 |
| `08xx` | M7 Chapter 1 | Full flow, saves, Chapter 1 content, **Nick's playtest**, options, colour themes, music, credits, end-of-battle music, results, title art, chapter card, transitions | 0801–0813 |
| `09xx` | M8 Release | itch.io, Windows polish, Steam readiness | 0901–0903 |
| `11xx` | The engine | Shuyi Engine, an open-source 2D engine carved out of this codebase; the game built on it ([`docs/engine/`](engine/README.md), [its repository](https://github.com/Zafnok/shuyi-engine)) | 1101–1118, 1131, 1151–1153, 1157, 1158–1187 (the rest moved to the engine's repository) |

**Audio** (decided in 0020, [`docs/design/audio.md`](design/audio.md)) runs
across the blocks: **0212** playback plumbing → **0213** our own sounds and
**0214** the imported music and sounds → **0424** battle sounds, **0425** menu and
cursor sounds, **0710** `@music` in scenes, **0807** title and battle music,
**0808** credits screen, **0809** victory/defeat stings and Game Over music
(picked in **0022–0024**); later places get their music picked in **0025**
world map, **0026** capital, **0027** camp, **0028** shops (played by 1007,
1003, 0409). Level-up and EXP sounds are Nick's own work (**0605** plays
them once he supplies them). The Content ID check of the music (**0904**) was
closed without doing it: Nick chose to swap a track or add a streamer mode
only if a claim ever happens (`audio.md`).

**Title intro cinematic** (Nick, 2026-10-01: a cinematic on the title as
long as the title song, looping with it, ending on the logo): **0036**
decided 2026-10-03 (`design/title-screen.md`, *Intro cinematic*: one
press to the menu, the cinematic after 15 idle seconds, a press skips to
the logo; "Trailer cuts" storyboard; whole-step zoom; it did not wait for
**0811** title art, the logo shot shows whatever 0811 makes) →
**0228** pan and zoom over a glyph scene → **0817**
cinematic file format and player (also needs **0227**, the music clock,
which has no open dependencies and can start any time) → **0818** character and conversation shots and
**0819** the title plays it in time with the music, and **0827** shots
where units march and fight → **0820** the real shots (after Chapter 1's
map, faces and script) → **1010** the overworld shot (after the world map,
1007; Nick places it). **0041** picks the skip sound (any time). **0724**
rewrites the Chapter 1 script with fewer quips; the cinematic shows that
script's opening lines, whatever they are. On the Chapter 1 critical path
since 2026-10-12 (see *Playtest round 2* below).

**Playtest round 2** (Nick, 2026-10-12, after playing `v0.1.0`: the
feedback "impacts my first impression more than I thought"; 0804 is parked
until it is built). 0804 now also waits for: **0727** a Confirm tap no
longer fast-forwards the next page of dialogue · **0724** the script
rewrite (American English, lines that read plainly) · **0447** the
player's blue outline easier to see · **0444** the cursor easier to see (done) ·
**0055** → **0448** which enemies can attack the tile you are moving to ·
**0442** Combat Arts in their own menu · **0439** the 1× / 2× zoom (and
which one the game starts at) · the title's logo and cinematic (**0012** →
**0811** → **0819** → **0820**, with **0817**, **0818**, **0827**) ·
**0830** the Game Over animation with the logo (after 0811). All of these
are on the critical path now.

**Playtest round 3** (Nick, 2026-10-14, after playing `v0.2.0`: "it's my
top priority to get all this feedback done and then replay ch1 over and
over to get the feel correct"). 0804 now also waits for: **0607** the lead's
portrait on level-up · **0451** point at a hurt
ally to heal · **0452** forest tiles that show where the forest ends ·
**0429** what a skill or art does, shown while choosing it · **0728** the
Chapter 1 script reworked as scenes (0719, 0722 and 0726 wait for it) ·
**0729** the dialogue log · **0730** Maud's softer eyes · **0832** a
combat-scene shot → **0833** the title cinematic moving like the game
(also after 0728) · **0834** Tamsin less dominant · and the decisions
**0056** Cinder Oath's range (decided 2026-10-15: range 1, the enemy
can't counter; built by **0317**), **0057** a tutorial (decided 2026-10-14:
New Game asks how familiar you are, a Fox Miko guide, checklist lessons in
Chapter 1, Guides in the battle menu; built by **0455** the guide's face in
the tip box, **0835** the familiarity question → **0456** checklist lessons
→ **0457** the lessons → **0458** new tips and Guides → **0731** the guide's
name and lines), **0058** which options
belong to a save (decided 2026-10-14: player profiles hold every option
and their own saves; built by **0836** profiles under the options and saves
→ **0837** the title's Profile screen (also after **0839**, leaving
Options asks whether to keep the changes), and **0838** the tutorial level
in Options, after 0835 and 0839), **0029** backgrounds behind conversations (decided 2026-10-15:
the talk over its place, darkened; a short moving opening where the script
asks; the battle map for talk around a battle; built by **0732** the
`place` directive and the darkened place → **0733** Chapter 1's yard and
study → **0734** the moving openings, whose sign-off asks again about the
scene music), each with the tickets it creates. All of these are on the critical path now.

**Playtest round 4** (Nick, 2026-10-16, after playing `v0.3.0`: "generally
it's an 8/10, where before it was 3 or 4/10... we can park playtest 4 til
after engine carve and playdate port"). **The engine carve (to 1118) and
the Playdate port (to 1050) come first**. 0804 and every round 3 ticket
wait for both, except **0464**, **0467** and **0841** (tutorial bugs the
Playdate shares), which wait for nothing and come before 1050.
Then 0804 waits for: **0736** research what makes
fantasy dialogue compelling → **0737** the Chapter 1 script written again
from the beats (0719, 0722 and 0726 now wait for it) → **0742** other music
for three scenes · **0738** the places' round 3 notes (longer openings, no
outline on actors, a lighter study, the party at the toll-fort) · **0739**
how much faces dim over a place · **0740** narration left-aligned (after engine#2305) · **0741**
the pre-battle place reads as the chapel green · **0843** the intro's fixes
(longer news, shorter fight hold, hit sounds, the logo on the chord, a
start through black) · **0844** a setting to turn the intro off · **0446**
Tamsin's combat picture · **0449** arrows where the map goes past the view (done)
· **0466** a zoom tip · **0464** lessons only when they can be done →
**0465** the end of the lessons points at Guides and offers a fresh start ·
**0467** tips lost before they show · **0841** Tutorial None stops what's
queued · **0463** the forecast's weapon swap easy to find · **0468** Vix's
ears · **0842** Vix's profile nudge on the title · **0250** key names that
stand out · **0847** a better dialogue log key · **0461** the road up to a
bridge · **0462** aiming starts on an enemy · **0318** Cinder Oath can't
counter · **0845** the objective becomes defeating Harl → **0846** the lead
a little stronger. Parked, not before round 4: **0063** rework the intro's
shots, **0064** a story role for Vix, **0065** naming Vix and the lead,
**0469** pictures in Guides, **0908** polish before the Steam release
(sparks, arrows; 0903 waits for it).
**Folded into the Act 1 playtest (2026-10-21, ticket 0129):** round 4
was never played; 0804 and 1050 closed into **1064**, which waits for
every round 4 ticket above, and none of them waits for 1118 or 1050 any
more (Lane G of *Act 1 critical path*).

**Replaceable graphics** (Nick, 2026-10-01: swapping the map's glyphs for
sprites from image files must be a small job; ADR-0038): **0231** pictures
in the frame as sprite items (no open dependencies; 0711 and 0413 now draw
bought art with it, so it is on the Chapter 1 critical path) → **0232**
`cargo shuyi frame-png`, a picture of any scripted frame without a window.
**0432** the battle map is painted by a map skin from a plain-data map
scene, look unchanged (no open dependencies) → **0433** a sprite skin read
from a tileset file, behind the debug menu, and **0434** battle tests read
the scene instead of cells. Bots and play records already never touch the
look (ADR-0033); 0505 and 0508 now check it.

**Bought art on the battle map** (Nick, 2026-10-02, ticket 0038: he bought
the whole Mega Tiles bundle and chose its tiles and map sprites for the
battle map, with per-map lighting and a zoom key): after **0433** and
**0110**, **0436** units as the bought map sprites (→ **0440** they step
on the spot and walk along their path) → **0437** terrain from
the bought tilesets (auto-tiling; done 2026-10-03, ADR-0052) → **0438**
per-map lighting (done 2026-10-12, ADR-0058); **0439** the
1× / 2× zoom toggle (after 0433; Nick picks its key when it is built).
Nick wants the bought-art map in his Chapter 1 playtest, so **0432, 0433,
0436, 0437 and 0440 are on the critical path** (0438 is not; 0439 is since
2026-10-12, for playtest round 2).
**0039** (done 2026-10-02) settled the rest: sprites step and walk (0440,
which the playtest waits for), an up or down arrow marks a unit under an
effect (0436), dialogue shows the busts at 4× (0711), and players may
pick the glyph look in Options (**0824**, after 0436 and 0805; not on the
critical path).

**Machine-made now, replaceable by people later** (Nick, 2026-10-03,
[`docs/design/voices-languages-and-script.md`](design/voices-languages-and-script.md):
AI voices with an off switch, machine-translated Japanese and Chinese
options (Chinese added 2026-10-04), and a
script a hired writer can take over; **not** a blocker for Chapter 1 or
Act 1). Shared first step: **0717** every dialogue line gets an id (no
open dependencies).
*Japanese* (ADR-0045): **0042** Nick picks the font and how a language is
chosen → **0233** screen text by key (no open dependencies) → **0234** the
rest of the screens (the battle and class change screens: **0243**) and **0235** data names, tips and dialogue (also needs
0717); **0236** wide glyphs (after 0042 and engine#2301) → **0237**
Japanese line breaking (after engine#2302); **0825** the Options row (after 0805); **0718** the translation
pipeline → **0719** Chapter 1 and the screens in Japanese (after 0716).
*Chinese (Simplified)*, on the same engine work: **0045** Nick gives the
characters for "Shuyi", the Chinese title and font, and says who checks
it (after 0042) → **0239** a wide font per language and Chinese line
breaking (after 0236, 0237 and engine#2303); **0725** the translation pipeline (after
0235) → **0726** Chapter 1 and the screens in Chinese (after 0716, 0825).
*Voices* (ADR-0046): **0043** Nick picks the tool, what is voiced and each
voice by ear; **0238** playback plumbing (after 0717) → **0720** the
dialogue screen plays them, **0721** the generation tool, **0826** the
Options rows (after 0805) → **0722** Chapter 1's voices.
*Script:* **0723** a scriptwriter's kit (after 0715).
*Disclosure:* **0907** credits and store text say what is machine-made
(0901 and 0903 use it).

**The engine: Shuyi Engine** (Nick, 2026-10-14: split an open-source,
general 2D engine from the game, which stays paid; networking too, "if it
turns out feasible without jeopardizing our own velocity").

- **The study.** Ticket 1101: [`docs/engine/feasibility.md`](engine/feasibility.md)
  is the report and ADR-0063 the split.
- **Nick's answers.** Plan B. The engine's own repository,
  [`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine), is public under MIT OR Apache-2.0 and
  holds its ADRs, tickets, decisions and roadmap. The extra game styles
  and networking are parked until the engine reaches parity with the game.
- **The critical path.** From 2026-10-16 the carve was the critical
  path; it is done (1118). Since 2026-10-21 the critical path is Act 1
  (the note at the top, *Act 1 critical path*).

The order:

1. **Done or any time:**
   - ~~**1102**: the `engine/` folder and the boundary gate~~ (done).
   - **0059**: whether the tactics kit is open, and who may contribute.
     Any time.
   - **0060**: does the army ever walk around freely? After 0804.
2. **Now, the carve, one at a time (the critical path):** **1103** → **1104** →
   **1105** → **1106** → **1107** → **1108** → **1109** → **1110** →
   **1111** → **1112** → **1113**. **1114** (the tactics kit in `core`)
   runs beside them. Then **1115** and **1116** → **1117** → **1118**,
   the first example. **1151** (the engine-friction rule: game code far
   bigger than the job triggers a review) can start once 1118 is done.
   Then ~~**1131**, the code moves to the engine repository~~ (done
   2026-10-19: the game pins it, `shuyi-engine.rev`), then
   **1152** (the engine-change skill for two repositories) → **1157**
   (the engine parts of open game tickets move to the engine repository)
   → **1153** (the first friction sweep of the game).
   - **What the sweep filed** (`docs/engine/friction.md`, *Sweeps*):
     game tickets that delete the game's own versions once the engine
     has the piece, each blocked by its engine ticket: **1158** and
     **1159** (screen results, engine 2306), **1160** (list cursor,
     2307), **1162** (menu values, 2308, after **1161**), **1163**
     (questions, 2309), **1164** (the profile screen, 2310), **1165**
     (drawing helpers, 2311), **1166** (frame time, 2312), **1167**
     (map skins, 2313), **1168** (option rows, 2314), **1169** (content
     loaders, 2208 and 2209), **1170** (the Playdate front end, 3010,
     3028 and 3029). Ready now, no engine needed: **1161** (one list
     mode in the battle), **1171** (one owner for the campaign
     settings), **1172** (`code-lines --tree` for sweeps), and **0470**
     (the forecast's kill mark as a sprite). None is on the Chapter 1
     critical path.
   - **What the engine's own tickets need of the game:** **1175** (the
     game's `main` and tests on the engine's platform boundary, engine
     3003), done by whichever PR first moves the pin past 3003. 1170
     builds on it.
   - **What the 0506 sweep filed** (2026-10-22): **1180** (the game's
     playtesting on the engine's: engine#2129–2132, #2532, #2533,
     #2021).
   - **What the retro of the last 50 PRs filed** (2026-10-22): **1183**
     (0509's autobalance on `shuyi autobalance`, engine#2541; done by
     0509's PR, since **1188** pinned engine#2541),
     **1184** (the engine's one-bit look replaces the game's one-bit
     skins of engine screens, engine#2371, after 1077), **1185** (the
     first sweep's leftovers: frame time, yes/no prompts, cursors,
     cuts), **1186** (one rule for a broken content file,
     engine#2233), **1187** (friction estimates against a reference,
     the signals in the PR). None is on the Act 1 critical path.
   - **Then the second friction sweep**, when the `11xx` block closes:
     **1174** (an area column in the friction log, `cargo xtask
     friction-check` in the gates, net lines for refactors) writes the
     sweep ticket, blocked by the block's other open tickets, so it is
     scheduled, not left to whichever session notices.
3. **Then the engine's own tickets, in its repository:** its MVP
   (parity), then what comes after, and the parked tickets. Our game's
   own multiplayer stays **0053**'s decision, after launch.

## Act 1 critical path

Written 2026-10-21 (ticket 0129, Phase 0: the backlog moved off the
Chapter 1 playtest gates onto 1064). Only Chapter 1 exists as content
(`assets/chapters/ch01.ron`); the chapter flow already chains chapters
with `next`. Phases run in order, but Phase 1's questions and Phase 2's
lanes overlap: a lane starts as soon as its own tickets are ready.
Within a lane, → is order and · is "side by side". The only engine
tickets Act 1 waits for are the playtest bots' (engine#2125–2128, Lane
C; re-gated 2026-10-21 by 0130; engine#2128 added by 0506).

**Phase 1: Nick's design queue**, in this order (each unblocks the
tickets in brackets):

1. **0037** building tiles, capturing, healing tiles (done: house, gate,
   throne; seize only; no tile heals → 0322, 0471)
2. **0044** the Preparations shop's basics and tiers (0443, 1070) →
   **0073** four armour weights (0321, 0320, Chapter 1's loadouts) →
   **0074** the Steel armour sold from Chapter 5 (its data ticket)
3. **0028** shop music (0473)
4. **0069** Act 1's battle rules: the escape objective, Dace fleeing,
   a boss who can't die, Hollis's death and its Classic fallback, the
   kinds of side-quest reward, what each early chapter introduces
   (0319, 0848, 0849, 0750, 1072, 1083, 1084)
5. **0025** world map, **0026** capital and **0027** camp music (1007,
   1003) · **0071** story-battle themes and scene music for Chapters
   2–8 (0848–0854)
6. **0070** faces and combat pictures for the new cast (0848–0854,
   1069); **0076** the two it left open, Hedda and the Hound lieutenant
   (0848, 0851, 1069)
7. **0606**'s question, where to promote and reclass (asked inside 0606
   before it is built)
8. **0013** the number scale, decided before Act 1 is balanced
   (0848–0854, 1070–1072, 1083, 1084; done 2026-10-22: FE-sized, built by 0323
   and 0608)
9. ~~**0018** higher-rank arts and special weapons~~ ✅ decided
   2026-10-26 (0325–0329, then 0848, 1072, 1083 and 1084)
10. **1007**'s world-map glyph mockup (asked inside 1007 before it is
    built)

Optional, any time: **0812** chapter title card, **0813** screen
transitions, **0067** the bought tiles and sprites on the Playdate's
map, **0063**–**0065** (the intro's shots, a story role for Vix, naming
Vix and the lead). Asked when their ticket is picked up: the questions
inside Lane G's tickets (0250, 0449, 0463, 0739, 0741, 0742, 0846,
0847) and 1062's; **1009** (quest recruitment) only if 1083's runaway
Candle joins outside battle.

**Phase 2: systems**, in lanes that run side by side:

- **A. World map:** **0322** the house, gate and throne tiles (after
  0037) → **0313** save the village · **0471** a tileset for poor
  houses and a throne (after 0322; Nick buys) · **0409** the shop
  screen → **0473** its music (after 0028) · **0443** the Preparations shop tab (after
  0044) → **0474** a negative bonus reads `Res -3` · **0606** promote and reclass · **1003** supports and camp
  screens (after 0027) → **1007** the world map (after 0409, 0025,
  0026; mockup first) → **1008** skirmishes → **1065** the act ends
  (closing scene, "Act 1 ends", the army carried over) · **1074** a
  discovered side quest shows when it closes (after 1007) · **1081** Vix
  explains the world map (after 1007, 1070) · **1082** icons for the
  world map's picture look (after 1007) · **1198** the world map's
  shortcuts on the Key bindings screen (after 1007, engine#2395).
- **B. Battle rules:** **0319** escape, a boss who retreats, a boss
  who can't die, a scripted death (after 0069) → **0324** the escape
  objective names its exit · **0511** bosses use their
  debuff and stance arts · **0513** green units carry their own
  consumables (0044) · **0321** the four armour weights in the data
  (after 0073) → **0320** specialty stores and **0855** Chapter 1
  back in its playtest bands · **0323** extra strikes need
  a gap or a share of the enemy's speed, ceilings HP 200 / stats 100
  (0013) · **0608** the level pace, fast early and slow late (0013).
- **C. Bots**, engine first (0130): each step is an engine PR in
  `Zafnok/shuyi-engine`, then the game PR that moves the pin and adds
  the game's parts. engine#2125 the rollout bot (`Determinize`, flat
  Monte Carlo) · engine#2126 the playtest runner with pass bands (both
  done) → engine#2128 the turn planner (whole-turn plans; one command at
  a time the personas stalled on Chapter 1) → **0506** Casual / Normal /
  Hardcore (the personas' order and scoring, the battle's `Determinize`,
  measures and targets; done 2026-10-22, all three in their bands on
  Chapter 1) → **0516** the personas score an escape and a downed boss
  right (before Chapter 6's escape map, 0852) · **0515** playtest CI on
  `main` plays every balance change
  · engine#2127 the autobalance search (after engine#2126) → **0509**
  autobalance (the battle's knobs) → **0510** skirmishes to order.
  engine#2541 (`shuyi autobalance`, pinned by **1188**): 0509
  implements its plan hooks instead of a command of its own, and 0509's
  PR moves **1183** to done.
- **D. Playdate, Chapter 1's base:** **1178** arrows at the map's
  edges · **1075** weapon and spell lines that fit (asks Nick first) ·
  **1039** → **1041** →
  **1044** · **1042** → **1043** · **1046** · **1058** · **1060** →
  **1062** · **1051** the AI plans once per phase · **1057** → **1059**
  · **1056** (done) · **1076** the exit area and a downed enemy get
  looks of their own · **1077** one-bit skins wrap their own notes and
  scroll by the engine's rule (after 1039).
- **E. Playdate, Act 1:** **1068** one chapter's assets at a time
  (ready; measure first) · **1196** saves survive a power cut (ready) · **1066** world map, towns and shop (after
  1007, 1008, 0409, 1038) · **1067** camp, supports, promotion and the
  Preparations shop (after 1003, 0606, 0443, 1046) · **1069** one-bit
  faces and pictures of the new cast (after 0070, 1042, 1041).
- **F. Optional engine work** (in the engine's repository; nothing
  waits for it): engine#2305 (done; **1177** picks it up), engine#3045,
  engine#3043, engine#3052, engine#3008 → engine#3009. The bots'
  engine tickets aren't here: Lane C waits for them. engine#3009
  (slicing a bot's thinking over frames) isn't the bots': they run on
  a PC. Its first user is the enemy AI on the Playdate (**1051**,
  which doesn't wait for it). The MCP server's fixes, engine#2545
  (done), engine#2546 and engine#2542, and the skill pack, engine#1134,
  → **1189** moves the pin past them. **1190** (done) replaced pin
  tickets with a watcher on the engine's `main`
  ([ADR-0096](adr/0096-a-watcher-on-the-engines-main-moves-the-pin-when-its-public-api-changes.md)):
  1177, 1188 and 1189 were the last. The engine starts it on each merge
  (engine#2031, a token Nick sets up once; until then it runs hourly),
  and an `engine-pin` issue follows a public API change (engine#2030's
  label, or a `pub` line until then) within minutes. The `engine-pin`
  skill works it: each move reviews what the engine PRs give the game
  and writes `docs/engine/pins.md`, which CI checks. No more pin
  tickets. **1192** (done) took the skill pack into `.claude/skills/`
  at the pin (ADR-0097) → engine#2559 (`check-skills` asks a worked
  example only of the pack's skills) → **1194** makes `check-skills` a
  gate.
- **G. Chapter 1's round 4 fixes:** **0736** first, because its craft
  research governs every Act 1 script → **0737** the Chapter 1 script
  → **0742** · **1177** the engine pin (moved by 0740) · **0449** (done) · **0845** →
  **0846** · the rest of round 3's tickets: **0250**, **0318**,
  **0446**, **0461**, **0462**, **0463**, **0465**, **0466**, **0468**,
  **0738**, **0739**, **0741**, **0842**, **0843**, **0844**, **0847**.
  Cheap Chapter 1 gaps, any time: **0450**, **0453**, **0454**,
  **0470**, **0472** (a downed unit's panel; the objective line fits),
  **0735**.

**Phase 3: content.** **0743** the new cast's character sheets (after
0736) → the scripts, one chapter at a time because the ledger makes
them sequential: **0744** (Chapter 2, after 0737) → **0745** → **0746**
→ **0747** → **0748** → **0749** → **0750** (Chapter 8, also after
0069). Each chapter's battle after its script, 0509, 0323, 0608, 0070 and
0071: **0848** Chapter 2 (also 0313, 0409, 0443, 0069, 0329) → **0858**
Chapter 2's art (after Nick buys Hendry's picture and builds his face)
and **0520** the bots play a chapter with the army the chapters before
it leave → **0849** Chapter 3 (also 0037, 0069), **0850**–**0854**
Chapters 4–8 (also 1007, 1008, 0319, 1070). The world map's content: **1070** nodes,
towns and the random pool (after 1007, 1008, 0044, 0323, 0608) → **1071** the
eight fixed skirmishes, with Gil (after 0746–0749) and **1072**, **1083** and **1084** the five
side quests (after 0018, 0329, 0069 and their chapters' scripts). The
arts for ranks C–S and the special weapons (0018 decided them): **0325**
the arts that change numbers → **0326** stances that react, **0327**
strikes at several units, **0328** Disarm, Marked Shot and Unbroken
Stride, **0329** special weapons (0848, 1072, 1083, 1084); **0075** decides the
Moonblade's questline (with Act 2's outline); **1073**
supports and camp events (after 1003, 0743, 0750).

**Phase 4: proof.** **0512** the bots play all of Act 1 (softlocks,
balance bands, save and load at every node, a Playdate memory and
timing run; after Phases 2 and 3) → **1064** Nick plays Act 1 on the
web and on his Playdate.

**Off the path** (nothing in Act 1 waits for them): engine 3010, 3028,
3029 and game **1170** (the Playdate front end on the engine's
backend: cleanup, the game's own front end already runs on the
device); languages and voices (0042, 0043, 0045, 0236, 0237, 0239,
0718–0726, 0825, 0826, 0907, 1048, 1049); mods (0054, 1011–1024);
replayability decisions **0046**–**0052** and **0060** (after 1064) and
**0053**; release work (0901, 0903, 0906, 0908, 1052–1054); **0857**
(the battle's mode and its own data set in one place); and the rest of
the `01xx` and `11xx` tooling tickets (among them **0131**, ticket
numbers checked against open PRs, and 1183–1187).

## Nick's queue (answer these first; any order within a row)

Design answers unblock most of the rules work. Suggested order:

1. **0001** stats & combat · **0002** turn structure · **0003** weapons & items · **0004** magic · **0006** death & difficulty · **0007** setting, tone & story beats
2. **0005** level ups & classes (after 0001) · **0008** world structure · **0009** Chapter 1 scope (after 0007) · **0014** Combat Arts (after 0003) · **0016** the lord's unique class line (after 0005, 0009)
3. **0011** look & feel sign-off (after the font ticket 0203 shows real pixels)
4. Before the Chapter 1 playtest: **0021** bought portraits and battle sprites (done: Tiny Tales) · **0035** Harl's combat picture (done: Shironejiya's bearded axe bandit) · **0040** more packs for fighters with no fitting art (done) · **0413** how the combat scene looks · **0022** victory sting (done: Fupi's Win Jingle, a random version each win) · **0023** defeat sting (done: no sting; The Fallen plays from the banner through Game Over) · **0024** Game Over music (done: The Fallen loops, with a pause between rounds) · **0810** how battle rewards are shown
5. Anytime, low priority: **0039** sprite units: walking, the effect mark, the glyph look as an option, face or bust · **0012** title (then **0811** title art) · **0041** the sound when a press skips the title cinematic · **0025**–**0028** world map / capital / camp / shop music · **0812** chapter card · **0813** transitions
   · **0059** Shuyi Engine: is the tactics kit open, and who may contribute (`docs/engine/feasibility.md`)
   · **0042** Japanese: font, picking a language, the machine-translation label · **0043** AI voices: the tool, what is voiced, each voice by ear · **0045** Chinese (after 0042): the characters for "Shuyi", the Chinese title, the font, who checks it
6. Now, for Act 1 (*Act 1 critical path*, Phase 1, re-gated 2026-10-21): **0013** number scale (done) → **0323** strike rule & ceilings, **0608** level pace · ~~**0018** higher-rank Combat Arts & special weapons~~ ✅ decided 2026-10-26 (built by 0325–0329) · **0037** more building tiles (village, gate, throne), capturing, healing tiles (done) · **0044** the Preparations shop's basic items and tiers · **0069** Act 1's battle rules · **0070** the new cast's faces → **0076** Hedda's and the Hound lieutenant's · **0071** Chapters 2–8's music · **0073** four armour weights (done) → **0074** the Steel armour
   · After the Act 1 playtest (1064): **0060** does the army ever walk around freely (towns, camp, dungeons)? (decides whether the engine's free movement is game work)
7. After the Act 1 playtest (1064), replayability (`design/replayability.md`): **0046** stars, Replay and "Move on?" · **0047** Visions and heat for random skirmishes · **0049** Ironman · **0050** the run mode (after 0047) · **0051** the randomizer · then **0048** what stars and Visions pay (after 0046, 0047) · **0052** replays, the veteran's ghost, sharing (after 0046) · **0054** the mod policy: what mods may change, modded saves, Workshop, the licence
8. ~~**0061** the Playdate version of Chapter 1~~ ✅ decided 2026-10-15 (`design/playdate.md`) · **0062** how the Playdate's one-bit pictures look (portraits, combat picture, launcher card) · **0066** the guide's tip and lesson box on the Playdate's screen · **0067** the bought tiles and unit sprites on the Playdate's map, in one-bit (instead of 0061's glyphs?)
9. After the single-player launch: **0053** multiplayer (modes, armies, fairness, servers)

No sign-offs after merge any more (Nick, 2026-10-21): he checks design
(the `00xx` tickets and questions asked before building) and plays the
Act 1 playtest (1064); `tickets/README.md`, *No checks after merge on
gameplay tickets*. Setup steps (accounts/secrets): 0103, 0104, 0106,
0108, 0116, 0901, later 0903.

## Chapter 1 critical path

**History.** Since 2026-10-21 the critical path is *Act 1 critical
path* above. What's below is the history of what led up to Chapter 1
playtest rounds 1 to 3; round 4 was folded into the Act 1 playtest
(1064).

What's still open between now and Nick's playtest (0804), by dependency depth
(tickets on the same row can run in parallel sessions/worktrees). Updated
2026-10-01 from the tickets' `blocked_by` lists, after a dependency check:
done tickets were dropped, 0711 now waits for 0110 (it needs the bought
files), and 0803 now waits for 0710 and 0807 (it sets Chapter 1's music).
0231 (pictures as sprite items, ADR-0038) was added to row 1: 0711 draws
the bought faces with it.
Nick also put the combat scene (0413), Harl's picture (0035) and 0316
(non-attack skills cost uses per battle) in front of the playtest, and
0822 (Game Over offers Rewind while charges are left; "yes", PR #140).
2026-10-02: 0714 (dialogue speakers who aren't units) was added to row 1
and the script (0707) moved to row 2 behind it; without it the script
couldn't pass its check until 0803, which waits for the script.
Later that day the script (0707) was done and left the table. It found
that a companion who dies in Classic still talks in the victory scene, so
**0715** (script lines that depend on who is still in the army) joined row
1 and **0716** (the Chapter 1 scenes rewritten with it) row 2, both in
front of the playtest unless Nick says they can wait.
Also 2026-10-02 (ticket 0038): Nick bought the art and wants the battle
map drawn with it before he plays, so the map skin (0432, then 0433), the
bought unit sprites (0436, which also needs 0110) and the bought terrain
(0437) joined the table. Ticket 0039 added 0440 (walking sprites) the
same day: Nick said the playtest waits for it.

```
 1  0022 0023 0024 0413 0706 0822
 2  0803 0809
 3  0804  ◄── Nick plays Chapter 1
```

The table was brought up to date on 2026-10-04 (ticket 0121): every other
ticket named above is done.

**0035** (Harl's picture) left row 1 on 2026-10-04: Nick chose it in 0040
(Shironejiya's bearded axe bandit), so 0413 and 0706 no longer wait on it.
The private assets repo is
done (0110 and 0116, 2026-10-02, ADR-0040): Nick uploaded it and added its
build key, so 0711 and 0436 no longer wait on it.

**0435** (found by 0411) adds the `PLAYER PHASE` banner missing at the
start of a battle, which `turn-structure.md` asks for.

The `01xx` gates (0103–0106) aren't needed by the game itself but should land
early so every later PR is checked by them.

## After Chapter 1 (`10xx`+, tickets created when relevant)

- World map (0008 chose FE Sacred Stones-style, after the linear opening
  chapters): 1007 nodes/travel/towns/save/camp, 1008 fixed and random skirmishes.
- Supports (FE GBA-style, earned in battle) and camp events: 1002, 1003.
  Parked far-future ideas: hub activities (1004), pair abilities (1005).
- **Playdate port of Chapter 1** (Nick, 2026-10-15: "rush" it; unparked
  ahead of the engine carve. Since 2026-10-16 the carve comes first and the
  port second: the note at the top): 1026 technical test (do the rules build for
  the device, how long an enemy turn takes) and 0061 Nick's decisions
  (decided 2026-10-15, `design/playdate.md`) ran side by side; 1025 wrote
  the build tickets (2026-10-15). The rules run on Nick's Playdate
  (ADR-0070: stable Rust and the SDK's C build; a 14-enemy turn thinks for
  1.2 s, start-up loads in 3.7 s). Not on the Chapter 1 critical
  path; nothing on the desktop or web game changes for it. The build
  tickets in order (tickets on one row can run side by side):
  1. **1027** `core` without `std` (+ the `no-std` CI job) · **1030** the
     console size passed in (`ConsoleSpec`, ahead of carve ticket 1108) ·
     **0062** Nick decides the one-bit pictures (portraits, combat
     picture, launcher card)
  2. **1028** `content` without `std` → **1029** `ui` without `std`
  3. **1031** `crates/playdate`, the game on its own stack,
     `cargo xtask playdate` and its CI job · **1032** the one-bit frame
     (1-bit font, Playdate palette, renderer)
  4. **1033** the front end runs the game (loop, D-pad/A/B, saves)
  5. **1034** button chords → **1035** the crank rewinds, the system menu,
     map menu entries · **1036** load only what the Playdate uses, loading
     card · **1037** the one-bit map skin · **1055** the one-bit render
     fast enough on the device (1033 measured ~360 ms a frame; now ~17 ms)
  6. **1040** one-bit images → **1061** the bought art in the Playdate's
     bundle · **1047** sound and music · **1038** the
     battle screen (after 0242) → **1039** forecast, unit info, level-up,
     rewind (after 0248)
  7. **1041** the combat picture · **1042** one-bit portraits (both after
     1061) · **1045**
     New Game and Options screens (after 0247) · **1046** preparations,
     results, saves, game over
  8. **1043** the dialogue screen (after 0249) · **1044**
     the title screen and cinematic · **0066** Nick decides the guide's
     tip and lesson box on the Playdate → **1058** builds it (Nick saw
     the PC's box run off the screen, 2026-10-16) · **1060** lessons
     whose action has no Playdate button can be finished (after 1035) →
     **1062** the guide's words without colours or keys (Nick decides
     the keys lesson on the Playdate)
  9. ~~**1050** Nick plays Chapter 1 on his Playdate~~ (folded into
     **1064**, the Act 1 playtest, on 2026-10-21) · **1051** the AI plans
     once per phase (done before 1064: Act 1's battles are bigger) · **1057** the
     battle's own frame time on the device (1055 measured 18 ms a frame
     at least; either order with 1050) → **1059** the battle map at 50
     frames a second (the screen's most; after 1038) · **0067** Nick
     decides whether the map uses the bought tiles and sprites (its build
     tickets follow, after 1037 and 1040)
  10. **1052** package and sell on itch.io (after 0901 and 1064) →
      **1053** the Catalog application
  11. When their inputs exist: **1048** voices (after 0722) · **1049** one
      build per language (after 0719, 0726)
- Controller support on every build (web, Windows, Linux, macOS; needed for
  Steam Deck): 0032 decide buttons → 0219 input → 0220 button names in help
  bars → 0816 rebinding (after 0815). Not on the Chapter 1 critical path;
  0219 can land any time after 0032. Steam-specific gaps: 0903.
- Class tiers 4 and up; tier-3 class skills (1001).
- Claude-drawn 16×16 class icons (1006): parked. Nick chose the bought map
  sprites for units (0038), so this waits until he asks for icons.
- Audio still open after 0020: a banter conversation track, the crit sound of
  a spell with no element, fliers' movement (`docs/design/audio.md`).
  World-map / capital / camp / shop music: 0025–0028.
- Colour-blind palette variant; text size options.
- Difficulty modes; more chapters (story pipeline repeats per chapter).
- Fuzzing the content parsers (`cargo-fuzz`).
- Automated playtesting bots (Nick, 2026-09-30): 0033 decide the player
  types and targets → 0504 legal moves and luck reseeding in `core` → 0505
  `cargo xtask playtest` runner (after 0801) → 0506 Casual/Normal/Hardcore
  bots (after 0803, engine#2125, engine#2126 and engine#2128) → 0507 trained
  AlphaZero-style bot (research) and 0508 recording Nick's play to
  calibrate the bots (after 0802) → 0509 autobalancing battles with the
  bots (after engine#2127) → 0510 generating skirmishes to order (after
  1007, 1008). Targets: `docs/design/playtest-bots.md`; how to run them
  and read the report: [`docs/playtesting.md`](playtesting.md). 0506,
  0509 and 0510 are on the Act 1 critical path (Lane C), engine first
  since 0130.
- Replayability (Nick, 2026-10-04/05, `docs/design/replayability.md`):
  fixed stars on story battles and fixed skirmishes, with Replay and a
  "Move on?" question after a win (0046); Visions, an opt-in pick of three
  on random skirmishes (0047); their rewards (0048); an Ironman mode (0049);
  a roguelike run mode on the title menu (0050); a randomizer (0051);
  replays, the veteran's ghost and sharing a battle (0052). Multiplayer
  after the single-player launch (0053). Modding and dev tools (a mod
  folder, a map editor, a small scripting language, the bots as a balance
  tester): Nick said yes. 0054 the mod policy (Nick) → 1011 ADR on how
  mods work (its engine parts: engine#2207, engine#2101, engine#1129) →
  1012 mod layers in the asset bundle (after engine#2207) → 1013 data
  entries merged by id and 1014 the mods folder (also after engine#2207)
  → 1015 Mods screen and modded saves, 1016 `vos-modkit check`/`new` →
  1017 scripted skill effects on the engine's script runtime (after
  engine#2101) → 1018
  scripted events and objectives; 1019 → 1020 → 1085 the map editor; 1021
  `vos-modkit playtest` (after 0506); 1022 mod campaigns; 1023 guide and
  example mod; 1024 Steam Workshop (after 0903).
