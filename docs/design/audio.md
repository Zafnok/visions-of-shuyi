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

Which story battle uses which `battle_*` cue, and which scene switches to
which mood, is story content. Each battle file and scene script names its
cue.

### Chapter 1's cues (ticket 0803, 2026-10-11)

| Moment | Cue | Decided |
| ------ | --- | ------- |
| The letter (`ch01_intro`) | `village_home` | *Claude's pick* |
| The Red Company (`ch01_prebattle`) | `dungeon_tense` | *Claude's pick* |
| The battle | `battle_bright` | Nick ("A") |
| After the fight (`ch01_victory`) | `scene_sad` | *Claude's pick* |
| Veyne Hall (`ch01_tbc`) | `talk_antagonist` | *Claude's pick* |

On the four scene picks Nick said "I'll have to feel them out": they are in
the build for him to hear in the playtest (0804), not approved. After playing
(2026-10-14) he said "I want to see how the cutscenes would play out with
this ambience": they wait for the conversations' backgrounds (decided in
ticket 0029, built by 0732–0734), and are asked about again with 0734's
sign-off. The battle's
own story scenes (Harl's lines, the death quotes) don't change the music.

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
  have more eastern themes". It needs its own round.
- **Places and moments without a cue yet:** capital city, world map, camp,
  shops. Nick hasn't been asked about these (tickets 0025–0028). Level up
  is Nick's own work.
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
