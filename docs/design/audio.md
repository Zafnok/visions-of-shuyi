# Music and sound effects

Decided: 2026-09-28
Source: ticket 0020

Nick chose by ear, over six rounds of a listening page
(https://claude.ai/artifact/68oDV3HuxB2Vc2zHBzadRc). The page's source,
including the recipes for the sounds we make ourselves, is in
[`assets-src/audio/sound-audition.html`](../../assets-src/audio/sound-audition.html).
His words, round by round, are in the [appendix](#appendix-nicks-words).

## The rules

1. **Music is recorded music by other composers.** It's free under CC0 or
   CC-BY 4.0 (ADR-0013); CC-BY 3.0 is fine too (Nick, on 0214: "idc which
   CC it is as long as I can freely use it"; ADR-0027). We make no music ourselves: Nick tried the code-made
   sketches and dropped them ("I guess we don't need these anymore").
2. **We make a few sounds ourselves:** the menu sounds, the dodge and the
   heal. Every other sound effect is a recording (CC0 or CC-BY 4.0).
3. **Credit everyone, even CC0** (Nick: "For any work we use, even if we can
   use for free, I still want to credit so we can have a nice credits
   screen"). The game gets a credits screen that lists every third-party
   work. This is stricter than ADR-0013, which only requires credit for
   CC-BY.
4. **Music comes in tiers.** Epic tracks are reserved for epic moments.
   cynicmusic's Battle Themes are "for TRULY EPIC battles like act bosses or
   something. They would be out of place against bandits."
5. **Music belongs to places and story moments, not just screens:** village
   size, a first visit to a big city, a dungeon, the first battle in a new
   area, a side quest, a companion's introduction.
6. **One track per battle, across both phases.** There is no enemy-phase music.
   "I would expect continuity between the track for the whole battle."
   Enemy units get movement sounds, the same as player units.
7. **Conversations use mood tracks, like Fire Emblem** (0020 Q1 "A"). A story
   scene can switch to one of a few mood tracks.
8. **Story battles each get one chosen theme, like Fire Emblem** (0020 Q2
   "A"). These themes sit above the skirmish pool and below the epic tier.
   Skirmishes ("inconsequential battles") **pick at random from a pool**.
9. **Asset handling** (Nick: "organize them with tags, attribution, and prefer
   looping / no vocal versions"). Every imported file records its title,
   author, source URL, license and tags. Where a track comes in several
   versions, use the looping and no-vocal ones.
10. **Movement sounds play for both sides.** The map cursor ticks as it moves,
   like Fire Emblem, "at a lower vol maybe": the menu move sound, quieter.

## Music cues

The cue names are the ones the game data uses. All links go to pages with
a player. **Version** is the file to import when the page has several.

| Cue | When it plays | Track | Composer | Source | License |
| --- | ------------- | ----- | -------- | ------ | ------- |
| `title` | Title screen | New Sunrise (V1, `New Sunrise.wav`) | nene | [OGA](https://opengameart.org/content/new-sunrise) | CC0 |
| `city_first_visit` | Entering or exploring a large city, e.g. the first time | New Sunrise (V2, `new_sunrise_V2.wav`) | nene | [OGA](https://opengameart.org/content/new-sunrise) | CC0 |
| `village_home` | A small village, e.g. the first one after the exile, where the home base is set up | Squirrel Village (the loop file) | SoManyWhales | [OGA](https://opengameart.org/content/squirrel-village) | CC-BY 4.0 |
| `village` | A small or medium village (not a capital) | Aria | Kistol | [OGA](https://opengameart.org/content/aria) | CC0 |
| `dungeon_tense` | Exploring a tense dungeon, uncovering a conspiracy | Dark Forest Theme | cynicmusic | [OGA](https://opengameart.org/content/dark-forest-theme) | CC0 |
| `graveyard_desert` | A graveyard or a desert | Dark Quest | Alexandr Zhelanov | [OGA](https://opengameart.org/content/dark-quest) | CC-BY 4.0 |
| `side_quest` | A side quest, or a young companion's introduction | Father's Scabbard (the loop file) | SoManyWhales | [OGA](https://opengameart.org/content/fathers-scabbard) | CC-BY 4.0 |
| `talk_calm` | Conversation mood: calm, everyday | Field – Orchestra | migfus20 | [OGA](https://opengameart.org/content/field-orchestra) | CC-BY 4.0 |
| `talk_antagonist` | A conversation with the antagonist, or a "god's eye" view of them | Classical Murder | Alexandr Zhelanov | [OGA](https://opengameart.org/content/classical-murder) | CC-BY 4.0 |
| `scene_sad` | A sad moment | Peractorum (from "Emotional Orchestral Music") | César da Rocha | [OGA](https://opengameart.org/content/emotional-orchestral-music) | CC-BY 4.0 |
| `scene_tragic` | A tragic scene | Hesitation (Orchestral Version) | Maarten Schellekens | [FMA](https://freemusicarchive.org/music/maarten-schellekens/free-music-made-for-screen/hesitation-orchestral-version/) | CC-BY 4.0 |
| `mythic_moment` | **Reserved** for a "pulling the sword from the stone" moment, e.g. unlocking magic; only if the story has one | Epic Endgame Cinematic (no-vocal version if one exists) | cynicmusic | [OGA](https://opengameart.org/content/epic-endgame-cinematic) | CC0 |
| `battle_bright` | Story battle with little tragedy | Hope (Orchestral battle music) | mintodog | [OGA](https://opengameart.org/content/hopeorchestral-battle-music) | CC0 |
| `battle_bittersweet` | Story battle that's half tragic | Battle | mla | [OGA](https://opengameart.org/content/battle-0) | CC-BY 4.0 |
| `battle_easy` | Story battle we expect to win easily | Sigil | Kistol | [OGA](https://opengameart.org/content/sigil) | CC0 |
| `battle_new_area` | The first battle in a new area (e.g. act 3's return to the starting region) | Ending Scene (the orchestral version) | nene | [OGA](https://opengameart.org/content/ending-scene) | CC0 |
| `battle_epic_a`, `battle_epic_b` | Act bosses, truly epic battles | Battle Theme A; Battle Theme B for RPG | cynicmusic | [A](https://opengameart.org/content/battle-theme-a), [B](https://opengameart.org/content/battle-theme-b-for-rpg) | CC0 |
| `battle_church_2`, `battle_church_3` | Battles against the Church | Fantasy Choir 2; Fantasy Choir 3 | César da Rocha | [OGA](https://opengameart.org/content/fantasy-choir-3-orchestral-pieces) | CC0 |
| `skirmish` pool | Skirmishes: one track picked at random per battle | Battle Themes 1, 2, 3 and 5; RPG - Battle Theme | Alexandr Zhelanov; jocolloman | [Zhelanov](https://opengameart.org/content/battle-themes), [jocolloman](https://opengameart.org/content/rpg-battle-theme-0) | CC-BY 4.0 |
| `victory` pool | A battle is won: plays **once** while the `VICTORY` banner is shown, one version picked at random each time (0022) | Win Jingle, 13 of its 17 versions (`.wav` files in `winjingle.zip`; list below) | Fupi | [OGA](https://opengameart.org/content/win-jingle) | CC0 |
| `victory_grand` | **Reserved** for a ceremony, or the win after an act boss "or some other big moment"; plays once (0022) | Victory Fanfare Short (`Heavy_ConceptB.wav`, about 12 s) | cynicmusic | [OGA](https://opengameart.org/content/victory-fanfare-short) | CC0 |
| `ceremony_intro` | **Reserved** for the start of "a royal crowning or some other regal ceremony"; only if the story has one; plays once (0022) | Victory Music Cue (13.3 s; the original `.wav` needs a freesound login, the public HQ preview does not) | Joshua Empyre | [freesound 404025](https://freesound.org/s/404025/) | CC-BY 4.0 |
| `defeat` | A battle is lost: starts when the `DEFEAT` banner is shown and **carries on through the Game Over screen** without starting again (0023). It **loops, with a pause of 10–15 s between rounds** (0024, 0809) | The Fallen (RPG Orchestral Essentials), the whole piece (`The Fallen MP3.mp3` in `The Fallen MP3_0.zip`, 77 s) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/content/the-fallen-rpg-orchestral-essentials-defeated-music) | CC-BY 3.0 |
| `scene_loss` | **Reserved** for "a story beat of a scripted loss of someone"; only if the story has one (0023) | Bittersweet (`bittersweet.mp3`, 34 s; it opens with 1.7 s of silence) | Emma_MA | [OGA](https://opengameart.org/content/bittersweet) | CC0 |
| `world_map` | The Act 1 world map, outside towns and battles; fades out, a quiet pause, fades back in (0025) | The World At Your Door (RPG Orchestral Essentials) (`The World At Your Door MP3.mp3` in `The World At Your Door MP3.zip`, 2:41) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/node/108343) | CC-BY 3.0 |
| `world_map_war` | The Act 3 world map (home at war); played like `world_map` (0025) | The Tread of War (RPG Orchestral Essentials) (`The Tread of War - MP3.mp3` in `The Tread of War MP3_0.zip`, 3:39) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/content/the-tread-of-war-rpg-orchestral-essentials-military-music) | CC-BY 3.0 |
| `world_map_somber` | **Reserved** for a world map in a somber stretch: after a catastrophe, "a lot of ppl died, and we're on a long quest", until "the pieces are put back together"; only if the story has one (0025) | World Map (`World Map_0.mp3`, 3:16) | Scott Elliott | [OGA](https://opengameart.org/node/91067) | CC-BY 4.0 |
| `battle_desperate` | Kell's Ford (Chapter 2): a race to save a burning village (0071) | Determined Pursuit (`determined_pursuit_loop.wav`, 1:48, the file is the loop) | Emma_MA | [OGA](https://opengameart.org/content/determined-pursuit-epic-orchestra-loop) | CC0 |
| `battle_rival` | Veyne Hall (Chapter 4): storming the lead's old home, Dace as boss (0071) | A Slave To No One (RPG Orchestral Essentials) (`A Slave To No One MP3.mp3` in `A Slave To No One MP3.zip`, 5:51) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/content/a-slave-to-no-one-rpg-orchestral-essentials-boss-music) | CC-BY 3.0 |
| `battle_night_siege` | The Envoy's House (Chapter 5): holding a house at night (0071) | Battle Music (`Battle_1.ogg`, 1:36; the page gives the loop: at 1:28.6 jump back to 0:04.6) | Peter Eastman (peastman) | [OGA](https://opengameart.org/content/battle-music-0) | CC-BY 3.0 |
| `battle_war` | The Ashfields (Chapter 6): between two armies (0071) | Laments of the War (the FLAC on the page; the MP3 preview is 3:48) | Cethiel | [OGA](https://opengameart.org/content/laments-of-the-war) | CC0 |
| `battle_finale` | Saltmere Docks (Chapter 8), Act 1's last battle (0071) | Heroic Demise (updated) (`Heroic Demise (New)_0.mp3`, 5:02) | Matthew Pablo | [OGA](https://opengameart.org/content/heroic-demise-updated-version) | CC-BY 3.0 |
| `scene_departure` | The ship sails at the end of Act 1, after Hollis dies (0071) | RPG - A Long Way From Home (`RPG - A Long Way From Home.ogg`, 4:06, loops) | Hitctrl | [OGA](https://opengameart.org/content/rpg-a-long-way-from-home) | CC-BY 3.0 |
| `scene_confide` | A long-kept secret told at last: Hollis's confession (Chapter 5) (0071) | Soliloquy (`Soliloquy_1.mp3`, 3:44) | Matthew Pablo | [OGA](https://opengameart.org/content/soliloquy) | CC-BY 3.0 |
| `scene_lamplight` | The king's closing scene at Varenhall (end of Act 1): tender, with a quiet unease (0071) | Snowfall (the looped version, `Snowfall (Looped ver.)_0.ogg`, 0:50) | Kistol | [OGA](https://opengameart.org/content/snowfall) | CC0 |
| `holy_city` | A holy city's streets: sacred, quiet, the Vigil's Ossary (Chapter 7) (0071) | Wendale Abbey (`WendaleAbbey.ogg`, 1:28; plainsong under heavy reverb, made from MIDI and samples; on import, check by ear that the chant has no words) | Yubatake | [OGA](https://opengameart.org/content/wendale-abbey) | CC-BY 4.0 |
| `capital` | The army is in a capital, any realm's: one track for every capital. The first visit to a big city plays `city_first_visit` instead (0026) | Adventures in Adventureland (`Adventures in Adventureland.mp3`, 4:21) | Kevin MacLeod | [incompetech](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN2100029) | CC-BY 4.0 |
| `camp` pool | The Camp screen is open (supports and camp events). **Rotates**: one track plays, fades out, a short quiet, the next fades in (0027) | Medieval: The Old Tower Inn (`The_Old_Tower_Inn.mp3`, 1:45); Woodland Fantasy (`Woodland Fantasy_0.mp3`, 2:30); Medieval: The Bard's Tale (`The_Bards_Tale.mp3`, 2:38) | RandomMind; Matthew Pablo; RandomMind | [Old Tower Inn](https://opengameart.org/content/medieval-the-old-tower-inn), [Woodland Fantasy](https://opengameart.org/content/woodland-fantasy), [Bard's Tale](https://opengameart.org/node/80558) | CC0; CC-BY 3.0; CC0 |
| `scene_dread` | Something wrong found under a holy place: the Candles' empty dormitory (Chapter 7) (0071) | Solemn (Opening) (`Solemn (Opening)_0.mp3`, 1:39; it lists a voice, so on import check by ear that it sings no words) | İlker Yalçıner (onemansymphony) | [OGA](https://opengameart.org/content/solemn-opening) | CC-BY 4.0 |

The `victory` pool's 13 versions are the same 3.8 s tune on different
instruments: Bandoneon, Brass, Doot, Fantasia, Fretless, Grand Piano, Jazz
Guitar, Muted Guitar, Nylon, Pizzicato, Steel Drums, Strings, Vibraphone.
Nick heard all 17 on the 0022 listening page
(https://claude.ai/artifact/VaGGdWgQ4Bo3gGUjLqJ3mS) and cut four: **Saw,
Square, Banjo and Harpsichord are not used.** No story moment uses `victory_grand` or `ceremony_intro` yet:
Chapter 1 has no act boss and no ceremony.

There is no separate defeat sting (0023, like Fire Emblem on the GBA): one
piece, `defeat`, starts at the banner and keeps playing on the Game Over
screen. The Game Over screen does not start a track of its own, and going
from the banner to that screen does not start the piece again. When the 77 s
run out while the screen is still open, the piece **loops**, after a pause of
10–15 s (0024, answered while building 0809). Nick heard seven candidates on
the 0023 listening page (https://claude.ai/artifact/SGaFmZLLWeHwJojM9ded1J).
No story moment uses `scene_loss` yet.

### The world map (ticket 0025, 2026-10-26)

Nick picked over two rounds of a listening page
(https://claude.ai/artifact/VLAU7eYPcqx7pWRmSsVLzw).

- **A new world map track per act**, and also **per story arc**: when
  something tragic happens mid-act, the map swaps to a somber track
  (`world_map_somber`) until the story turns again.
- **Act 1:** `world_map`. **Act 3** (home at war): `world_map_war`, from
  the same composer's set. **Act 2** (the Jade Reach) gets an eastern track
  in the wuxia round (*Open sub-questions*).
- **The map track doesn't loop back to back.** It fades out, the map stays
  quiet for a while, then the track fades back in. The same goes for a
  small map's track (below).
- Kept for other places and moments, as Nick heard them. None of these
  exists in the story yet; each row says where the track would fit if one
  comes:

| Kept for | Track | Composer | Source | License |
| -------- | ----- | -------- | ------ | ------- |
| Exploring a cave with "some crazy reward at the end", like unlocking the crystals' secret in Final Fantasy | A Tale of Peace (RPG Orchestral Essentials) (`A Tale of Peace - MP3.mp3` in `a_tale_of_peace_mp3.zip`, 3:34) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/node/115742) | CC-BY 3.0 |
| The same place's windy or rainy parts: the same music as A Tale of Peace with howling wind or raindrops, so it can cross over from one area to the next | A Tale of Wind and Water (RPG Orchestral Essentials) (`A Tale of Wind - MP3.mp3`, 3:42; `A Tale of Water - MP3.mp3`, 3:38) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/node/98432) | CC-BY 3.0 |
| A cutscene of an enemy mobilizing its forces | The Enemy Marches (`March.ogg`, 2:18) | Peter Eastman (peastman) | [OGA](https://opengameart.org/node/55086) | CC-BY 3.0 |
| Very sad scenes | Leap into Eternity (the mix, `leapintoeternitymix.ogg`, 1:53) | Matthew Pablo; Of Far Different Nature (mixed by glitchart) | [OGA](https://opengameart.org/content/leap-into-eternity-matthew-pablo-of-far-different-nature) | CC-BY 4.0 |
| Very sad scenes | Wipe Away Those Tears (RPG Orchestral Essentials) (`Wipe Away Those Tears - MP3.mp3`, 3:24) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/content/wipe-away-those-tears-rpg-orchestral-essentials-sad-happy-music) | CC-BY 3.0 |
| A small map section, e.g. a beginner area (not a whole world); same fade and quiet as above | RPG - Overworld - Melodies of the Future (the loop `.wav`, 1:13) | jocolloman (Joseph Collard) | [OGA](https://opengameart.org/node/129152) | CC-BY 4.0 |
| A royal march (heard again for 0026: "more like a royal march than a city") | Adventure Map (`Adventure Map.mp3`, 3:23) | controllerhead | [OGA](https://opengameart.org/node/53699) | CC-BY 3.0 |
| Travel scenes, an airship, or riding a mount ("chocobo music") | Beyond The Clouds (Orchestral Theme) (2:21) | Matthew Pablo | [OGA](https://opengameart.org/content/beyond-the-clouds-orchestral-theme) | CC-BY 3.0 |

Not used: Orchestra Loop by Brandon Morris (no comment) and Exploration by
johnclark ("doesn't sound too good to me").

Which story battle uses which `battle_*` cue, and which scene switches to
which mood, is story content. Each battle file and scene script names its
cue.

### Chapter 1's cues (ticket 0803, 2026-10-11)

| Moment | Cue | Decided |
| ------ | --- | ------- |
| The letter (`ch01_intro`) | `village_home` | *Claude's pick*; Nick: try others (0742) |
| The Red Company (`ch01_prebattle`) | `dungeon_tense` | *Claude's pick*; Nick: try others (0742) |
| The battle | `battle_bright` | Nick ("A") |
| After the fight (`ch01_victory`) | `scene_sad` | *Claude's pick*; Nick: try others (0742) |
| Veyne Hall (`ch01_tbc`) | `talk_antagonist` | Nick ("Fits", 2026-10-16) |

On the four scene picks Nick said "I'll have to feel them out": they are in
the build for him to hear in the playtest (0804), not approved. After playing
(2026-10-14) he said "I want to see how the cutscenes would play out with
this ambience": they wait for the conversations' backgrounds (decided in
ticket 0029, built by 0732–0734), and are asked about again with 0734's
sign-off.
Heard over the places in playtest round 3 (2026-10-16): Veyne Hall
"Fits"; the letter "the music is ok but not perfect", the Red Company
"it's ok", after the fight "we can experiment". Those three get other
candidates in ticket 0742, after the script rewrite (0737). The battle's
own story scenes (Harl's lines, the death quotes) don't change the music.

### Chapters 2–8's cues (ticket 0071, 2026-10-28)

Nick picked by ear over three rounds of a listening page
(https://claude.ai/artifact/PYL5bgcLWQx3C2CXnM5V3V). His words are in the
[appendix](#appendix-nicks-words).

| Chapter | Moment | Cue | Decided |
| ------- | ------ | --- | ------- |
| 2 | Kell's Ford (battle) | `battle_desperate` | Nick ("2A") |
| 3 | Greywater (battle) | `battle_church_2` | Nick ("battle_church_2 is too good") |
| 4 | Veyne Hall (battle) | `battle_rival` | Nick ("4C", over two rounds) |
| 5 | The confession, the night before the battle | `scene_confide` | Nick ("C3") |
| 5 | The Envoy's House (battle) | `battle_night_siege` | Nick ("5J") |
| 6 | The Ashfields (battle) | `battle_war` | Nick ("A") |
| 7 | Ossary, the city's scenes (the streets, the pilgrims, Maud's old master) | `holy_city` | Nick ("P1 streets") |
| 7 | The Candles' empty dormitory, before Crane appears | `scene_dread` | Nick ("P6 dormitory") |
| 7 | Crane's scenes | `talk_antagonist` | Nick ("Crane can probably use talk_antagonist") |
| 7 | The Undercroft (battle) | `battle_church_3` | Nick |
| 8 | Saltmere Docks (battle) | `battle_finale` | Nick ("8A heroic demise") |
| 8 | Hollis dies | `scene_loss` | Nick ("D works") |
| 8 | The ship sails | `scene_departure` | Nick ("maybe H") |
| 8 | The king's closing scene (Varenhall) | `scene_lamplight` | Nick ("K3") |

- **One theme per battle.** Every story battle in Act 1 has its own track,
  except the Church's two: Greywater and the Undercroft use the two
  Fantasy Choirs, the cues Nick set aside in 0020 for fighting the Church.
  Act 1's act boss battle is Saltmere Docks, and it uses `battle_finale`,
  not the epic tier (Nick picked Heroic Demise over Battle Theme A when
  asked); `battle_epic_a` and `battle_epic_b` stay unused in Act 1.
- **Hollis's death plays two cues:** `scene_loss` as he dies, then
  `scene_departure` as the ship sails. If Hollis already fell in Classic
  mode, the rewritten scene keeps the same two cues (*Claude's starting
  rule*: the music follows the moment, not who speaks it).
- **Every other scene** in Chapters 2–8 picks from the mood cues
  (`talk_calm`, `dungeon_tense`, `scene_sad`, `scene_tragic`,
  `talk_antagonist`, `side_quest` for a young companion's introduction
  such as Joss's), chosen by each chapter's script ticket as *Claude's
  pick*, the way Chapter 1's were, and heard in the Act 1 playtest.
- **Kept for later** (Nick liked them, no moment yet):

| Kept for | Track | Composer | Source | License |
| -------- | ----- | -------- | ------ | ------- |
| Another battle ("keep D for later"; heard for Kell's Ford) | Ringing of Steel (`ringing_of_steel.wav`, 1:37, loops) | Alexandr Zhelanov | [OGA](https://opengameart.org/content/ringing-of-steel) | CC-BY 4.0 |
| "Another time" (heard for Greywater) | Dark Descent (`Dark Descent_0.mp3`, 1:23) | Matthew Pablo | [OGA](https://opengameart.org/content/dark-descent) | CC-BY 3.0 |

- **Heard as not right for a battle:** Dark Quest (`graveyard_desert`) "doesn't fit as battle music more as exploration music".

### The capital city (ticket 0026, 2026-10-28)

Nick picked over four rounds of a listening page
(https://claude.ai/artifact/EBQJyYJ43taXowUeU8cyYh). His words are in the
[appendix](#appendix-nicks-words).

- **One track for every capital** (not one per realm): `capital`, Adventures
  in Adventureland, Nick's "general theme". Varenhall, Brennmark's capital
  and the Jade Reach's all play it.
- **A capital should sound bustling and grand** (like FF12's Rabanastre) or
  like **busy medieval streets** (like the Witcher 3's Novigrad). Nick picked
  both when asked. Village tracks, a festival, a march and an intro piece
  were all heard as "not for a capital".
- `city_first_visit` stays the first visit's track: Nick hears it as "an
  intro cutscene to the city".
- **Kept for other places and moments** ("Keep them in our backpocket for
  other music selection tickets"; camp and shops are 0027 and 0028). None
  of these has a cue yet:

| Kept for | Track | Composer | Source | License |
| -------- | ----- | -------- | ------ | ------- |
| A tavern, or a capital's bustling streets | Minstrel Guild (`Minstrel Guild.mp3`, 3:06) | Kevin MacLeod | [incompetech](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1200098) | CC-BY 4.0 |
| A capital's upper court | Court of the Queen (`Court of the Queen.mp3`, 3:20) | Kevin MacLeod | [incompetech](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1100728) | CC-BY 4.0 |
| A small village by the sea, or a port: "very port inspired or sea inspired" | Market on the Sea (RPG Orchestral Essentials) (`Market on the Sea - MP3.mp3` in `market_on_the_sea_mp3.zip`, 2:26) | Jonathan Shaw (InspectorJ) | [OGA](https://opengameart.org/node/115435) | CC-BY 3.0 |
| "A city on a feast day, like a celebration" | Enchanted Festival (`Enchanted Festival.mp3` in `Enchanted Festival.zip`, 2:06; the zip also has a loop `.wav`) | Matthew Pablo | [OGA](https://opengameart.org/node/21241) | CC-BY 3.0 |
| "An adventure on a road or something", "Witcher 3 type beat"; now also in the `camp` pool (0027) | Medieval: The Bard's Tale (`The_Bards_Tale.mp3`, 2:38; also `Loop_The_Bards_Tale.wav`) | RandomMind | [OGA](https://opengameart.org/node/80558) | CC0 |
| Another music ticket (liked, "not for a capital city") | Medieval: Market Day (`Loop_Market_Day.mp3`, the loop, 1:04) | RandomMind | [OGA](https://opengameart.org/content/medieval-market-day) | CC0 |
| Another music ticket (liked, "not for a capital city") | Town Theme (`TownThemeSingleFileLoopable.ogg`, 1:48, loops) | Wolfgang_ | [OGA](https://opengameart.org/content/town-theme-0) | CC-BY 3.0 |
| Another music ticket (liked, "not for a capital city") | Town Theme 3 (`Town Theme 3.wav`, 1:57, loops) | LarsG | [OGA](https://opengameart.org/content/town-theme-3) | CC-BY 4.0 |

Adventure Map (kept from 0025) is now kept as a royal march only.

Not used: Throne Room (Of Far Different Nature), Royal Strings (jukeri12),
Shakkar (zesona), Town and Castle (Alexandr Zhelanov), Peaceful Place
(Alexandr Zhelanov), medieval town and a triumphant theme (syncopika),
Bustling City (Bryan Wysopal), and Kevin MacLeod's Heavy Interlude and Our
Story Begins.

### Camp (ticket 0027, 2026-10-28)

Nick picked in one round of a listening page
(https://claude.ai/artifact/TkCSYxm6jv26PQNct8uRv7). His words are in the
[appendix](#appendix-nicks-words).

- **Camp has three tracks, not one** (`camp` pool): Medieval: The Old Tower
  Inn, Woodland Fantasy and Medieval: The Bard's Tale (kept from 0026).
- **They rotate.** Unlike a skirmish, which picks one track at random and
  keeps it for the whole battle, camp plays one track, fades it out,
  stays quiet a moment, then fades the next one in. Nick: "rotate between
  them with a brief fade out, pause, fade in between them."
- Because each track plays to its end and fades, the **full versions** are
  imported, not the loop files (rule 9's loop preference is for a track
  that repeats back to back).
- Heard and not picked: A Small Fire Will Do (Cal McEachern), Fantasy
  Music – Night Town (Angus Macnaughton), Solo Guitar – Celtic #32
  (Tri-Tachyon), Suonatore di Liuto (Kevin MacLeod); the other tracks kept
  from 0026 (Market Day, Town Theme, Town Theme 3, Minstrel Guild) and
  `village_home` stay as they were.

## Sound effects

| Cue | When it plays | Sound | Author | Source | License |
| --- | ------------- | ----- | ------ | ------ | ------- |
| `menu_move` | Menu cursor moves | "B": muted chip, one D4 pulse note, filtered | in-house | audition page, `MENU.move` B | ours |
| `cursor_move` | Map cursor moves one tile | `menu_move`, quieter (60 % volume, *tunable*) | in-house | as above | ours |
| `menu_select` | Confirm / select | "L": chip, two quick notes up (D4 → G4) | in-house | audition page, `MENU.select` L | ours |
| `menu_cancel` | Cancel / back | "B": muted chip, two notes down (D4 → A3) | in-house | audition page, `MENU.cancel` B | ours |
| `hit_sword` | A sword strike hits | Sword sound 1 | Merrick079 | [freesound 568170](https://freesound.org/s/568170/) | CC0 |
| `hit_spear`, `hit_axe` | A spear or axe strike hits (the axe shares the spear's sound "for now") | Knife Stab | Mixedupmoviestuff | [freesound 179222](https://freesound.org/s/179222/) | CC0 |
| `hit_bow` | An arrow hits | Arrow Impact | Twisted_Euphoria | [freesound 205938](https://freesound.org/s/205938/) | CC0 |
| `hit_gauntlet` | A gauntlet strike hits | Punch | EminYILDIRIM | [freesound 544680](https://freesound.org/s/544680/) | CC-BY 4.0 |
| `crit_physical` | A weapon strike crits (replaces the hit sound) | Deep Cut / Slash / Gash | SypherZent | [freesound 420674](https://freesound.org/s/420674/) | CC0 |
| `block` | A strike hits but deals **0 damage** (0020 follow-up Q1 "A", like FE's no-damage hit) | Combat Punch Metal Armor | EminYILDIRIM | [freesound 545021](https://freesound.org/s/545021/) | CC-BY 4.0 |
| `miss` | A strike misses | "W2": quick, airy whoosh | in-house | audition page, `SFX['miss:quick']` | ours |
| `cast_fire` | A fire spell goes off (before it lands) | Short-Fireball-Woosh | wjl | [freesound 267887](https://freesound.org/s/267887/) | CC0 |
| `cast_ice` | An ice spell goes off (before it lands) | Hard Glass Impact | (deleted freesound user) | [freesound 418194](https://freesound.org/s/418194/) | CC0 |
| `hit_magic` | Any spell lands on a target | Magic Earth Spell Impact & Punch | EminYILDIRIM | [freesound 541477](https://freesound.org/s/541477/) | CC-BY 4.0 |
| `crit_fire` | A fire spell crits (replaces `hit_magic`) | Fireball Impact (untrimmed) | EminYILDIRIM | [freesound 577450](https://freesound.org/s/577450/) | CC-BY 4.0 |
| `crit_ice` | An ice spell crits (replaces `hit_magic`) | Magic Ice Impact Skill Spell (untrimmed) | EminYILDIRIM | [freesound 550267](https://freesound.org/s/550267/) | CC-BY 4.0 |
| `heal` | A unit is healed | "HE5": warm C–E–G chord blooming over a low G, about 1.5 s | in-house | audition page, `HEAL.HE5` | ours |
| `step_foot` | Each tile a foot unit moves; **picks one of three at random each step** | Footstep_Dirt_00; dirt/gravel footstep 4; Footstep_Grass_5 | LittleRobotSoundFactory; Yoyodaman234; GiocoSound | [270415](https://freesound.org/s/270415/) (CC-BY 4.0), [223153](https://freesound.org/s/223153/) (CC0), [421135](https://freesound.org/s/421135/) (CC0) | as listed |
| `step_armored` | Each tile an armoured unit moves | Knight Right Footstep on Gravel 5 (With Chainmail) | Ali_6868 | [freesound 384890](https://freesound.org/s/384890/) | CC0 |
| `step_mounted` | A mounted unit moves | Maxheadroom's Galloping Horse (a 39 s gallop; cut into steps or loop while moving) | Podsburgh | [freesound 274898](https://freesound.org/s/274898/) | CC0 |

Magic plays in two beats (0020 follow-up Q2 "A"): the element's cast sound
as the spell goes off, then `hit_magic` as it lands, or the element's crit
sound instead of `hit_magic` on a crit. A spell with no element has no cast
sound. A spell that a target **absorbs** (heals from, e.g. an elemental hit
by its own element) plays `heal` instead of `hit_magic` (Nick, 0424
sign-off: "it should sound like a heal").

## Starting values (*tunable*, chosen by Claude)

- The map cursor tick plays at 60 % of the menu move sound's volume. Nick
  asked for it "at a lower vol maybe".
- When one track switches to another, the old one fades out over 0.5 s and
  the new one starts. A cue that's already playing doesn't restart.
- Every third-party file is volume-matched on import, so no cue is much
  louder than the others.
- Sound effects play at 40 % of that matched level (about 8 dB quieter);
  music stays at 100 %. Nick found the sounds "way too loud" at 100 (ticket
  0222). The map cursor tick is still 60 % of the menu move sound, so 24 %.
- The menu select/confirm and cancel sounds play at 15 %: at 40 % Nick
  still found them "much too high" next to the rest (ticket 0223).
- On the world map (1007), a town's own cue (`village`, `capital`, …)
  plays while the town is open, its list of shops or a shop (Nick: "while
  interacting with a town"); the map's track comes back when it closes.
- The world map's track (any `world_map*` cue, and a small map's) fades
  out over 3 s at its end, the map is quiet for 30 s, then the track
  starts again with a 3 s fade in. Nick set the quiet at "probably more
  like 30s pause or maybe longer" (0025): try longer after playing. The
  3 s fades are Claude's.
- The `camp` pool (0027): a track fades out over 3 s at its end, camp is
  quiet for 5 s, then the next track fades in over 3 s. Nick asked for "a
  brief fade out, pause, fade in"; the numbers are Claude's. The first
  track is picked at random each time camp opens, then the order goes
  round the pool (Old Tower Inn → Woodland Fantasy → Bard's Tale → …), so
  no track plays twice in a row. Leaving camp fades out as any switch does.
- The pause between two rounds of `defeat` is 13.8 s: the piece's own quiet
  ending (1.8 s) and 12 s of silence added to the file. Nick asked for
  10–15 s (ticket 0809).
- After the victory sting ends, nothing plays until the next scene's music
  or the next screen's (ticket 0809). The sting isn't cut short when the
  banner closes; the battle's track doesn't come back.
- `Rewind` on the Game Over screen (0822): `defeat` keeps playing while the
  player looks through the rewind points. Once a point is picked, the
  battle's track starts again from its beginning (the same track, also in a
  skirmish). `Retry Battle` starts the battle's music as a new battle does
  (a skirmish picks again).

## Open sub-questions (not needed for Chapter 1; Nick: "we have enough sounds for ch1 atp")

- **Banter conversation mood:** nothing picked from the round 4 list
  (K13–K19). Until one is picked, a banter scene uses `talk_calm`.
- **Crit of a spell with no element:** "we will find more later". Until then,
  it plays `hit_magic`.
- **Fliers' movement** (tier 3+, `progression.md`): two wing flaps were
  shortlisted (freesound 389634, 670509), but none has been picked.
- **Separate axe sound:** "same as lance is ok for now".
- **The wuxia continent** (a later act, `setting-and-tone.md`) "would probably
  have more eastern themes". It needs its own round, which also picks
  Act 2's world map track (0025).
- **Places and moments without a cue yet:** shops. Nick hasn't been
  asked about it (ticket 0028). Level up is Nick's own work.
- **Content ID** (decided 2026-10-01, ticket 0904): **no check before
  release.** The plan was to test the chosen tracks against YouTube's
  copyright system (cynicmusic and Alexandr Zhelanov sell or stream their
  catalogues). Nick: "tbh I think we just bite the bullet and if one these
  guys decides they wanna copyright CC-BY stuff then we can swap it out or
  add a streamer mode later..." So: if someone reports a claim on a track,
  swap that track or add a streamer mode then. New tracks still avoid
  composers already known to register with Content ID (as 0020 did).

## Appendix: Nick's words

**Round 1** (menu families A–D, code-made combat sounds, two code-made
instrument sets, the first library list):

> for 1 I like A B C for Move, A and B for Select, and B for Cancel. Don't like
> D at all
>
> for 2 I don't really like any of these but Dodge Weighty sounds ok rest sound
> bad.
>
> For 3 I like the chip better BUT I think I would like the ensemble if not for
> the plucked voice. It drowns out the flute and softer strings. If we can
> modify it to reduce plucked harshness it would be good.
>
> For 4 I like Cynicmusic's battle theme A and B, but they should be reserved
> for TRULY EPIC battles like act bosses or something. They would be out of
> place against bandits
>
> Fantasy Choir 2 and 3 by cesisco would be good battle tracks too if we're
> fighting against the church at any point
>
> Present some more options for 1, 2, 3, 4 and I can keep iterating with you.
>
> For any work we use, even if we can use for free, I still want to credit so
> we can have a nice credits screen.

**Round 2** (menu variants and a sequence player, dodge variants, recorded
sounds, music sketches on four instrument sets, 35 library tracks):

> I think ill go with move B, select L (two notes def sounds better), cancel B
>
> for 2 W2 sounds better than W
>
> I like cynicmusic's dark forest theme I think it would be good for exploring
> a tense dungeon like uncovering a conspiracy
>
> Peractorum from cesisco sounds the best for a sad moment
>
> cynicmusic's epic endgame cinematic I feel would be good when getting a
> mythical item almost like pulling the sword from the stone kind of vibe... if
> we have any moment similar to that like unlocking magic or something it
> should be reserved for that moment
>
> nene new sunrise (v1) sounds pretty good as a title screen!!
> and nene new sunrise (v2) sounds good for exploring a large city or entering
> it for the first time
>
> still need a good conversation / minor battle track
>
> for sword hit, sword sound 1 by merrick079 sounds best.
>
> for crit, I actually like deep cut from sypherzent (though this is best for
> physical crits, magic crits probably shouldn't sound like this)
>
> I don't like any block / parry sounds you gave me they sound too tinny and
> hurt my ears
>
> for arrow, arrow impact by twisted_euphoria sounds best
>
> for fire, short fireball woosh by wjl sounds best
>
> for ice, hard glass impact sounds best
>
> I don't like any healing spell options you gave me
>
> for lance/axe, knife stab by mixedupmoviestuff sounds best

**Round 3** (conversation, ordinary-battle and enemy-phase music; block, heal,
magic crit, magic hit and axe sounds):

> nene ending scene orchestral sounds like the first battle in a new area
> (like in act 3 when we come back to the original area... I guess wuxia area
> would probably have more eastern themes)
>
> squirrel village by somanywhales is a good song for a small village like
> maybe the first one we enter after exile where we establish a home base
>
> Aria by Kistol also sounds good for a smaller village, or medium size, not a
> capital city
>
> father's scabbard sounds like a sidequest theme or the intro of a young
> companion
>
> battle themes 1-3 and 5 from Alexandr Zhelanov's battle themes pack sound
> good for a random selection between when a skirmish takes place...
> inconsequential battles can use this pool
>
> RPG - battle theme by jocolloman could also be added to this pool
>
> The enemy phase should not have a distinctive track. I would expect
> continuity between the track for the whole battle. They would just have SFX
> for their movement that's all.
>
> we can use combat punch metal armor by eminyildirim for blocked hit
>
> I still don't like the healing options you gave me
>
> magic hit G1 sounds good

**Design questions** (conversation music, story-battle music, heal brief,
leftovers):

> 1A
> 2A
> 3 idk... something not high pitched, something fast like under 1s,
> something conveying healing...
>
> magic crit - had no faves.
> axe - same as lance is ok for now
> movement - on both sides
> code-made - I thought we still liked a few of them?

(1A: dedicated conversation mood tracks, like Fire Emblem. 2A: a few
story-battle themes, chosen per battle, like Fire Emblem.)

**Round 4** (heal made to the brief, magic crits, movement, conversation
moods, story battles):

> if HE was longer it would sound good. HC sounds OK.
>
> MC3 and MC4 are ok for fire/ice crit respectively dont need trim since magic
> crit probably rare
>
> probably we can vary between F4, F3, and F2 for footsteps they are all pretty
> quick and if we vary it won't get annoying (ie each step picks a different
> one)
>
> A2 sounds good for armored and C4 for horses
>
> Field - Orchestra by migfus20 sounds good for conversation
>
> classical murder by alexandr sounds good for a conversation with or god's eye
> of an antagonist

**Round 5** (longer heals, and sections 4–6 again):

> probably HE5
>
> hesitation by maarten sounds good for a tragic scene
>
> Dark quest by alexandr sounds like a graveyard/desert theme
>
> Hope by MintoDog can be used for a battle not involving much tragedy in the
> story
>
> Battle by mla the same as Hope but it can be used in a halfway tragic / not
> tragic sense.
>
> Sigil by Kistol same as Hope - use this one in a battle we expect to win
> easily...
>
> code made music - you're right I guess we don't need these anymore
>
> when you download creators tracks be sure to organize them with tags,
> attribution, and prefer looping / no vocal versions.

**Follow-up questions** (block, magic beats, gauntlet, plain-spell crit, map
cursor):

> 1A
> 2A
> 3 punch by emin
> 4 we will find more later but I think we have enough sounds for ch1 atp
> 5 fire emblem has it for the cursor so I think we can have it for the cursor
> as well at a lower vol maybe?

(1A: the block sound plays on a hit that deals 0 damage. 2A: the element's
sound as the spell goes off, then the magic hit as it lands.)

**The victory sting** (ticket 0022, 2026-10-12; listening page
https://claude.ai/artifact/VaGGdWgQ4Bo3gGUjLqJ3mS). Round 1 was seven
fanfares (A–G), round 2 six more short ones (H–M), round 3 all 17 versions
of the win jingle:

> I like elements of A, B, C but don't like any of them as standalone. G
> would be good as some intro to a royal crowning or some other regal
> ceremony

> I guess the win jingle sounds alright from Fupi, maybe if we get all the
> versions and randomly pick one? and M also sounds quite regal so ceremony
> like or after an act boss or some other big moment

> saw, square, banjo, and harpsichord can be cut out rest sound ok

(A: VICTORY by Spring Spring. B: Orchestral Victory Fanfare by Sheyvan. C:
Won! by spuispuin. G: Victory Music Cue by Joshua Empyre. M: Victory Fanfare
Short by cynicmusic. The win jingle was J and K, its brass and strings
versions. The second question, one sting or several, is answered by the
same words: the everyday win picks from the jingle's versions, and a big
win can use M.)

**The defeat sting** (ticket 0023, 2026-10-12; listening page
https://claude.ai/artifact/SGaFmZLLWeHwJojM9ded1J). Seven candidates (A–G),
and a second question: a sting and then Game Over music, or one piece from
the banner through the Game Over screen:

> probably E for lose, with the music carrying over to game over, so take
> the whole piece probably...
>
> G is also a really good track but fits more with a story beat of a
> scripted loss of someone

(E: The Fallen by Jonathan Shaw. G: Bittersweet by Emma_MA. Recorded as:
no separate sting; `defeat` is the whole of E, from the banner on. G is
kept as `scene_loss`.)

**The Game Over music** (ticket 0024, asked while building 0809,
2026-10-12). The question: when The Fallen ends and the Game Over screen is
still open, loop it or play it once and stay silent. Nick picked "Loop it",
then added:

> ensure the loop has a 10-15s pause between loops for defeat/game over, and
> transition b/w defeat and game over should not reset music playhead

**The world map music, round 1** (ticket 0025, 2026-10-26; listening page
https://claude.ai/artifact/VLAU7eYPcqx7pWRmSsVLzw). Six candidates (A–F),
three cues we already have (R1–R3), and a second question: one world map
track for the whole game, or one per act:

> A sounds quite good, the only thing I'd be wary about is that looping it
> over and over might not be good but having it fade out and be quiet for a
> bit and pick back up after a while would be a good idea
>
> B is good for when we're at an emotional part of the story but there's
> still an overworld section so like... think some apocalyptic event
> happened, and we're trying to undo it, but a lot of ppl died, and we're on
> a long quest, then until the pieces are put back together, this can be the
> world map music
>
> C would be decent as capital city music, or a royal march theme, doesn't
> really give off world map feels
>
> D is good as a map theme, perhaps not as a whole world theme, like if we
> had small map sections like a beginner area it might be good, with same
> note as A about pausing and fading in and out
>
> F could be like airship music..? if that makes sense..? like if we have
> any travel scenes it could work or if we're ever mounted this could be
> like chocobo music
>
> I think we can get a few more samples and solidify the choices just record
> what I said first
>
> as for question 2, I think probably a new track per act might make most
> sense, or if we get to a new arc of the story (like if some tragic event
> happens at act 2 midway then it can swap to a somber theme)

(A: The World At Your Door by Jonathan Shaw. B: World Map by Scott Elliott.
C: Adventure Map by controllerhead. D: Melodies of the Future by jocolloman.
E: Orchestra Loop by Brandon Morris. F: Beyond The Clouds by Matthew Pablo.)

**The world map music, round 2** (ticket 0025, 2026-10-26, the same page).
G–I for an ordinary map, J–M for a map at war or in mourning, and how long
the quiet between rounds is:

> G sounds like if we are in a cave for exploration, and there is some
> crazy reward at the end like unlocking the secret of the crystals or
> something in final fantasy... not really an overworld map music piece.
>
> H1 and H2 are the same main music of G but with accompanying wind
> howling and raindrops respectively. So I have the same notes as G but if
> you want they can transition if we move to different areas.
>
> I doesn't sound too good to me
>
> Otherwise I like A more than G, H1, H2, and I
>
> J is best for act 3, while K is great for a cutscene about an enemy
> mobilizing forces, and L and M are good for very sad scenes.

(G: A Tale of Peace. H1, H2: A Tale of Wind and Water, the "Wind" and
"Water" versions. I: Exploration by johnclark. J: The Tread of War. K: The
Enemy Marches by Peter Eastman. L: Leap into Eternity. M: Wipe Away Those
Tears. G, H, J and M are by Jonathan Shaw. Recorded as: `world_map` is A,
`world_map_war` is J, `world_map_somber` is B from round 1.)

The quiet between rounds wasn't answered on the page. Told it was set at
13 s, Nick said:

> probably more like 30s pause or maybe longer

**Story battles and scenes, round 1** (ticket 0071, 2026-10-28; listening
page https://claude.ai/artifact/PYL5bgcLWQx3C2CXnM5V3V). Two to four new
tracks per Chapter 2–8 battle, cues we already have, the finale's two
moments and three scene moods (the confession, the king's closing scene,
the Vigil):

> for world map im confused why we're revisiting I thought we had it
> settled already
>
> for ch2, A and D both sound alright
>
> for ch3, battle_church_2, though I do like dark descent here too but
> battle_church_2 is too good. Keep Dark Descent around for another time
> tho.
>
> for ch4, probably C, though none feel completely right, but C is best
> from these 3.
>
> for ch5 I don't like any of them they are all too high bpm or in case of
> A too low bpm (stealth oriented but this is a battle)
>
> for ch6 A, I do like the graveyard_desert but it doesn't fit as battle
> music more as exploration music
>
> for ch7 battle_church_3
>
> for ch8 A sounds good
>
> for Hollis death yea D works
>
> for ship sailing away maybe H
>
> for confession probably C3
>
> for king closing K3
>
> for  Crane can probably use talk_antagonist

(The world map wasn't asked again: the page's opening picture looked like
a map. 2A: Determined Pursuit; 2D: Ringing of Steel. 4C: A Slave To No One.
6A: Laments of the War. 8A: Heroic Demise. D: Bittersweet. H: A Long Way
From Home. C3: Soliloquy. K3: Snowfall. Round 1's Envoy's House tracks
were Midnight Creeping, Omens, Dark Lands and Tactical Pursuit.)

**Story battles and scenes, round 2** (the same page). Five new tracks for
Veyne Hall with a sadder, bitter melody, seven mid-tempo ones for the
Envoy's House, and three checks (2A or 2D; which "A" for Chapter 8;
Ossary's city scenes):

> 2A and keep D for later
> 8A heroic demise
> 5J
> 4C
> not a fan of any of these for Ossary

(5J: Battle Music by Peter Eastman. The Ossary tracks were Elegy Dm,
Specters of the Enclave and Cathedral. Asked whether to reuse a mood for
now or hear more, Nick chose "round 3 with new candidates + the moods we
already have".)

**Story battles and scenes, round 3** (the same page). Eight new tracks
for Ossary's city scenes (chant, bells and drones, low strings, dark
ambient) and four moods we already have. Nick first asked:

> to be sure, Ossary is - the whole enemy's city? or just a city that
> happens to have a conspiracy tied to it?

(Answered from the bible: the Vigil's holy city, full of real pilgrims and
keepers who believe in their work; the conspiracy is Crane's faction, and
the wrongness is underneath. Suggested a split: something sacred for the
streets, something colder for the empty dormitory.)

> P1 streets, P6 dormitory

(P1: Wendale Abbey by Yubatake. P6: Solemn (Opening) by İlker Yalçıner.)

**The capital city, round 1** (ticket 0026, 2026-10-28; listening page
https://claude.ai/artifact/EBQJyYJ43taXowUeU8cyYh). Five candidates (A–E),
`city_first_visit` as R1, and a second question: one track for every
capital, or one per realm:

> I really like C and D but not for a capital. C more for a small village,
> it does sound very port inspired or sea inspired. D sounds yea, like a
> city on a feast day, like a celebration. R1 really only sounds like an
> intro cutscene to the city like how fortunes weave introduces Dagda. A
> sounds yea more like a royal march than a city. Rest I don't like much

> One for every capital

(A: Adventure Map by controllerhead. B: Throne Room by Of Far Different
Nature. C: Market on the Sea by Jonathan Shaw. D: Enchanted Festival by
Matthew Pablo. E: Royal Strings by jukeri12. R1: New Sunrise V2.)

**The capital city, round 2** (the same page). Six town, city and castle
tracks (F–K):

> G sounds like an adventure on a road or something idk like witcher 3 type
> beat. Rest not really a fan of. esp not for a capital city.

(F: Shakkar by zesona. G: Medieval: The Bard's Tale by RandomMind. H: Town
and I: Castle by Alexandr Zhelanov. J: Peaceful Place by Alexandr Zhelanov.
K: medieval town by syncopika.) Asked which sound a capital should have,
Nick picked two of four: "Bustling and grand" (like FF12's Rabanastre) and
"Busy medieval streets" (like the Witcher 3's Novigrad). Not picked:
"Stately and elegant" (FF9's Alexandria), "Proud, with unease".

**The capital city, round 3** (the same page). Five busy or grand tracks
(L–P):

> I like L, N, and O, but not for a capital city. Keep them in our
> backpocket for other music selection tickets but not for this. Another
> round

(L: Medieval: Market Day by RandomMind. M: Bustling City by Bryan Wysopal.
N: Town Theme by Wolfgang_. O: Town Theme 3 by LarsG. P: a triumphant theme
by syncopika.)

**The capital city, round 4** (the same page). Five tracks by Kevin MacLeod
(Q–U):

> Q can work for a tavern or bustling streets, S for upper court, and T as
> a general theme

(Q: Minstrel Guild. S: Court of the Queen. T: Adventures in Adventureland.
Recorded as: `capital` is T, the one track for every capital; Q and S are
kept. R: Heavy Interlude and U: Our Story Begins weren't mentioned.)

**Camp, round 1** (ticket 0027,
https://claude.ai/artifact/TkCSYxm6jv26PQNct8uRv7). Six new tracks for a
camp at night (A–F), five kept from 0026 (G, L, N, O, Q) and
`village_home` (V):

> B, C, and G work. Probably we can use all 3. I'm thinking rather than our
> approach for skirmish where we keep the same one, chosen randomly at the
> start, we can rotate between them with a brief fade out, pause, fade in
> between them.

(A: A Small Fire Will Do by Cal McEachern. B: Medieval: The Old Tower Inn
by RandomMind. C: Woodland Fantasy by Matthew Pablo. D: Fantasy Music –
Night Town by Angus Macnaughton. E: Solo Guitar – Celtic #32 by
Tri-Tachyon. F: Suonatore di Liuto by Kevin MacLeod. G: Medieval: The
Bard's Tale. Recorded as: the `camp` pool is B, C and G, rotating.)
