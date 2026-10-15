# Roadmap

Goal of the first phase: **a playable, story-driven Chapter 1** on Windows and
in the browser, then publish to itch.io, then Steam.

All work is in [`tickets/`](../tickets/README.md). Status is read from the
folders: `tickets/open/` vs `tickets/done/`.

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
| `11xx` | The engine | Shuyi Engine, an open-source 2D engine carved out of this codebase; the game built on it ([`docs/engine/`](engine/README.md), [its repository](https://github.com/Zafnok/shuyi-engine)) | 1101–1118, 1131 (the rest moved to the engine's repository) |

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
**0056** Cinder Oath's range, **0057** a tutorial (decided 2026-10-14:
New Game asks how familiar you are, a Fox Miko guide, checklist lessons in
Chapter 1, Guides in the battle menu; built by **0455** the guide's face in
the tip box, **0835** the familiarity question → **0456** checklist lessons
→ **0457** the lessons → **0458** new tips and Guides → **0731** the guide's
name and lines), **0058** which options
belong to a save (decided 2026-10-14: player profiles hold every option
and their own saves; built by **0836** profiles under the options and saves
→ **0837** the title's Profile screen (also after **0839**, leaving
Options asks whether to keep the changes), and **0838** the tutorial level
in Options, after 0835 and 0839), **0029** backgrounds behind conversations (Nick's idea:
a short moving scene, then the portraits over it, darkened), each with the
tickets it creates. All of these are on the critical path now.

**Replaceable graphics** (Nick, 2026-10-01: swapping the map's glyphs for
sprites from image files must be a small job; ADR-0038): **0231** pictures
in the frame as sprite items (no open dependencies; 0711 and 0413 now draw
bought art with it, so it is on the Chapter 1 critical path) → **0232**
`cargo xtask frame-png`, a picture of any scripted frame without a window.
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
0717); **0236** wide glyphs (after 0042) → **0237** Japanese line
breaking; **0825** the Options row (after 0805); **0718** the translation
pipeline → **0719** Chapter 1 and the screens in Japanese (after 0716).
*Chinese (Simplified)*, on the same engine work: **0045** Nick gives the
characters for "Shuyi", the Chinese title and font, and says who checks
it (after 0042) → **0239** a wide font per language and Chinese line
breaking (after 0236, 0237); **0725** the translation pipeline (after
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
- **The critical path.** None of it is on the Chapter 1 critical path,
  and nothing that moves code starts before 0804.

The order:

1. **Now:**
   - **1102**: the `engine/` folder and the boundary gate. It doesn't
     conflict with open work.
   - **0059**: whether the tactics kit is open, and who may contribute.
     Any time.
   - **0060**: does the army ever walk around freely? After 0804.
2. **After 0804, the carve, one at a time:** **1103** → **1104** →
   **1105** → **1106** → **1107** → **1108** → **1109** → **1110** →
   **1111** → **1112** → **1113**. **1114** (the tactics kit in `core`)
   runs beside them. Then **1115** and **1116** → **1117** → **1118**,
   the first example. Then **1131**, the code moves to the engine
   repository.
3. **Then the engine's own tickets, in its repository:** its MVP
   (parity), then what comes after, and the parked tickets. Our game's
   own multiplayer stays **0053**'s decision, after launch.

## Nick's queue (answer these first; any order within a row)

Design answers unblock most of the rules work. Suggested order:

1. **0001** stats & combat · **0002** turn structure · **0003** weapons & items · **0004** magic · **0006** death & difficulty · **0007** setting, tone & story beats
2. **0005** level ups & classes (after 0001) · **0008** world structure · **0009** Chapter 1 scope (after 0007) · **0014** Combat Arts (after 0003) · **0016** the lord's unique class line (after 0005, 0009)
3. **0011** look & feel sign-off (after the font ticket 0203 shows real pixels)
4. Before the Chapter 1 playtest: **0021** bought portraits and battle sprites (done: Tiny Tales) · **0035** Harl's combat picture (done: Shironejiya's bearded axe bandit) · **0040** more packs for fighters with no fitting art (done) · **0413** how the combat scene looks · **0022** victory sting (done: Fupi's Win Jingle, a random version each win) · **0023** defeat sting (done: no sting; The Fallen plays from the banner through Game Over) · **0024** Game Over music (done: The Fallen loops, with a pause between rounds) · **0810** how battle rewards are shown
5. Anytime, low priority: **0039** sprite units: walking, the effect mark, the glyph look as an option, face or bust · **0012** title (then **0811** title art) · **0041** the sound when a press skips the title cinematic · **0025**–**0028** world map / capital / camp / shop music · **0029** dialogue backgrounds · **0812** chapter card · **0813** transitions
   · **0059** Shuyi Engine: is the tactics kit open, and who may contribute (`docs/engine/feasibility.md`)
   · **0042** Japanese: font, picking a language, the machine-translation label · **0043** AI voices: the tool, what is voiced, each voice by ear · **0045** Chinese (after 0042): the characters for "Shuyi", the Chinese title, the font, who checks it
6. After the Chapter 1 playtest: **0013** number scale · **0018** higher-rank Combat Arts & special weapons · **0037** more building tiles (village, gate, throne), capturing, healing tiles · **0044** the Preparations shop's basic items and tiers
   · **0060** does the army ever walk around freely (towns, camp, dungeons)? (decides whether the engine's free movement is game work)
7. After the Chapter 1 playtest, replayability (`design/replayability.md`): **0046** stars, Replay and "Move on?" · **0047** Visions and heat for random skirmishes · **0049** Ironman · **0050** the run mode (after 0047) · **0051** the randomizer · then **0048** what stars and Visions pay (after 0046, 0047) · **0052** replays, the veteran's ghost, sharing (after 0046) · **0054** the mod policy: what mods may change, modded saves, Workshop, the licence
8. Now, at Nick's request (2026-10-15): **0061** the Playdate version of Chapter 1 (screen, buttons, crank, what's left out)
9. After the single-player launch: **0053** multiplayer (modes, armies, fairness, servers)

Sign-offs come later as screens land (0402 cursor feel, 0404 combat, 0408
preparations, 0409 shops, 0410 spells, 0411 battle notes, 0502 enemy
phase, 0602 level up, 0701 story gates, 0704 dialogue, 0706 portraits, 0707
script, 0804 playtest). Setup steps (accounts/secrets): 0103, 0104, 0106,
0108, 0116, 0901, later 0903.

## Chapter 1 critical path

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
  ahead of the engine carve): 1026 technical test (do the rules build for
  the device, how long an enemy turn takes) and 0061 Nick's decisions
  (400 × 240 one-bit screen, buttons and crank, what is left out) run side
  by side → 1025 writes the build tickets. 1026 is done: the rules run on
  Nick's Playdate (ADR-0070: stable Rust and the SDK's C build; a 14-enemy
  turn thinks for 1.2 s, start-up loads in 3.7 s). Not on the Chapter 1 critical
  path; nothing on the desktop or web game changes for it.
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
  bots (after 0803) → 0507 trained AlphaZero-style bot (research) and 0508
  recording Nick's play to calibrate the bots (after 0802) → 0509
  autobalancing battles with the bots → 0510 generating skirmishes to order
  (after 1007, 1008). Targets: `docs/design/playtest-bots.md`; how to run
  them and read the report: [`docs/playtesting.md`](playtesting.md). Not on
  the Chapter 1 critical path; 0504 can start any time.
- Replayability (Nick, 2026-10-04/05, `docs/design/replayability.md`):
  fixed stars on story battles and fixed skirmishes, with Replay and a
  "Move on?" question after a win (0046); Visions, an opt-in pick of three
  on random skirmishes (0047); their rewards (0048); an Ironman mode (0049);
  a roguelike run mode on the title menu (0050); a randomizer (0051);
  replays, the veteran's ghost and sharing a battle (0052). Multiplayer
  after the single-player launch (0053). Modding and dev tools (a mod
  folder, a map editor, a small scripting language, the bots as a balance
  tester): Nick said yes. 0054 the mod policy (Nick) → 1011 ADR on how
  mods work → 1012 mod layers in the asset bundle → 1013 data entries
  merged by id and 1014 the mods folder → 1015 Mods screen and modded
  saves, 1016 `vos-modkit check`/`new` → 1017 scripting runtime → 1018
  scripted events and objectives; 1019 → 1020 the map editor; 1021
  `vos-modkit playtest` (after 0506); 1022 mod campaigns; 1023 guide and
  example mod; 1024 Steam Workshop (after 0903).
