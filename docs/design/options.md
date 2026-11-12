# The Options screen

Decided: 2026-10-04
Source: ticket 0805 (Nick's comments on its pull request)

## Nick's words

On the first version of the screen (PR #193), 2026-10-04:

> "holding confirm should not speed up on top, but just change the 2x to a
> 4x"
>
> "lets make volume a 100 step with input box for precision as well as
> slider"
>
> "reset tips should have confirmation screen -- in general these kinds of
> things should have confirmation screens if they do something somewhat big"
>
> "you should not be able to switch to casual mid battle I think... but you
> can at the prep screen which is before battle and the return point if you
> select restart battle"

Everything marked *tunable* / *Claude's starting rule* is a default Claude
chose where Nick's words didn't say; Nick may veto it. Unmarked rules are
Nick's.

## Rules

### Where it opens

- From the **title** menu, from the **map menu** in a battle, and from the
  **Preparations** screen before a battle (an `Options` tab between `Pack`
  and `Fight!`; the tab's place is *Claude's starting rule*).

### Speeds

- **Animation speed: Fast** plays fights and the EXP bar ×2 (*tunable*).
  **Enemy phase speed: Fast** plays the enemy's and the other side's turn
  ×2 (*tunable*): the camera moving to each unit, the mark on it, and its
  fights; with both Fast, ×4.
- **Walking has its own speeds** (`look-and-feel.md`, *How fast a sprite
  walks*, Nick, 2026-10-04): a sprite unit walks 6 tiles a second, **8
  with Animation speed: Fast**, and 12 while Confirm is held in the
  enemy's turn, never more. The glyph look's walk stays at 12 whatever
  the settings. Enemy phase speed doesn't change a walk (*Claude's
  reading* of "it's just 12 max").
- **Holding Confirm plays at ×4, not on top.** With Fast animations a
  fight plays ×2, and holding Confirm makes it ×4, not ×8. If the settings
  already play ×4 (an enemy turn with both set to Fast), holding Confirm
  changes nothing (*Claude's reading* of "just change the 2x to a 4x").
- **Text speed:** Slow is half of Normal, Fast is twice it, Instant shows a
  whole text box at once (*Claude's starting rule*).
- **Combat animations: Off** skips every fight the way the Cancel key skips
  one; the EXP bar and level-up pages still show (*Claude's starting
  rule*).

### Map zoom

- **Map zoom: 1× / 2×** (ticket 0439; `look-and-feel.md`, *Zoom*): how big
  the battle map's pictures are drawn. It starts at **2×** (Nick,
  2026-10-12). The Zoom map key in battle (`controls.md`, *Zoom map*)
  changes the same setting, so the row always shows what the map is at.
- The row is under `Map look`, and shows only when the game has pictures
  to zoom (*Claude's starting rule*). `Restore defaults` puts it back to
  2×.

### Intro

- **Intro: On / Off** (ticket 0844; `title-screen.md`, *Intro cinematic*):
  whether the title plays its cinematic when left alone. It starts **On**
  (Nick, 2026-10-16: "Yes, an on/off setting"; not "only the first time").
  Off, the title stays on the logo, backdrop, sparks and menu, and the
  song keeps looping.
- The row is under `Map zoom` (above `Colors`), on the title's Options
  and in battle alike (it does nothing in battle) (*Claude's starting
  rule*). The Playdate leaves it out until it has the cinematic (1044).
  `Restore defaults` puts it back On.

### Open towns on arrival

- **Open towns on arrival: On / Off** (ticket 1007, Nick: "towns open
  on arrival with option in options menu to only open when you
  explicitly click confirm first"): On (the default), a town on the world
  map opens its shops when the army arrives there; Off, only Confirm on
  the town the army is in opens it.
- The row is under `Intro` (above `Colors`), on every Options screen
  (*Claude's starting rule*). The Playdate leaves it out until it has the
  world map. `Restore defaults` puts it back On.

### Volumes

- Music and sound volume each go from **0 (silent) to 100**, starting at 80
  (*tunable*). Since the engine pin move of 2026-11-10 (engine #318) the
  numbers are steps of loudness (0.6 dB each) rather than a share of full
  volume: the same starting loudness is **97**, which the row shows, and a
  save from before is read where it sounds the same.
- **A slider:** left and right move it 5 at a time (*tunable*).
- **A box for the exact number:** Confirm on the row opens it; type the
  number, Enter keeps it, Backspace deletes, Escape leaves the volume as it
  was. More than 100 counts as 100; an empty box changes nothing. On a
  controller there are no number keys, so the box shows the volume and the
  cursor moves it 1 at a time, Confirm keeps, Cancel leaves (*Claude's
  starting rule*).

### Voices in the dialogue log

- Ticket 0729 (Nick, 2026-10-14: the log plays a line's voice "also on
  hover with option to disable this behavior"; `look-and-feel.md`,
  *Dialogue log*): a row **Voice on hover in the log: On / Off**, starting
  **On** (*Claude's starting rule*). It comes with the voice options
  (ticket 0826), under the voice volume, and is kept in the player profile
  like them; until the game has voices there is nothing for it to turn
  off, so it isn't shown.

### Auto-advance

- Ticket 0859 (Nick, 0043: "Same, plus auto-advance";
  `voices-languages-and-script.md`, *Voices*): a row **Auto-advance: On /
  Off** under the voice rows, starting **Off**, kept in the player profile.
  On, a scene's text box moves on by itself when its voice clip ends, after
  a pause of 0.5 s; a line with no clip (or with voices off) waits as long
  as reading takes, 1 s plus 0.05 s per character (*Claude's starting
  rules*, *tunable*). A press still works as usual; a reply choice always
  waits for the player.

### Asking first

- **Anything that does something somewhat big asks yes/no first.** On this
  screen: `Reset tips`, `Restore defaults`, and the switch to Casual. This
  is a rule for every screen, not only this one.
- A value row (a speed, a volume, the cursor) doesn't ask: it is changed
  back with one key.

### Game mode

- The row shows the campaign's mode. See `death-and-difficulty.md`, *Mode
  changes*: Classic → Casual only, and only at Preparations.

## Where options are kept

Decided: 2026-10-14
Source: ticket 0058

### Nick's words

In the Chapter 1 playtest, round 2 (0804), 2026-10-14:

> "they should probably be per-save-file. I have never saved and yet the
> browser cached my options settings which feels weird to me."

Asked in ticket 0058 (per computer, per save, a split, or player profiles),
Nick picked **player profiles**, like *Into the Breach*, and then the
recommended answer to each follow-up:

- A profile holds **every option**.
- At launch the game opens on **the last profile used**; the title menu
  switches.
- Each profile has **its own save slots and its own suspend save**.
- The first profile is **made for you** and can be renamed later.
- Options being kept before any save is **fine now**: they belong to a
  profile you can see, switch and delete, not to the browser.
- The **tutorial level can be changed later, in Options**, for the save
  being played.
- A New Game starts with **the options last used** (with profiles, the
  profile's own).

On the pull request (PR #264), 2026-10-14:

> "ensure it's deliberate when a player overwrites their options, or
> presses cancel when backing out of profile screen and they didn't save,
> etc. if they don't want to overwrite, let them make that choice."
>
> "for controller, we can keep the on screen letter grid, but when we have
> steam launch or if we ever do a console launch etc then we should use the
> native input keyboard thing"

### Rules

- **A player profile** holds that player's options and saves: every
  option on the Options screen (text, animation and enemy-phase speed,
  combat animations, auto end turn, fullscreen, cursor style, map look,
  map zoom, intro, color theme, music and sound volume, and later the language
  (0825) and voices (0826)), the hand layout and key bindings, the tips
  already seen, the 30 save slots and the suspend save.
- **What stays with the campaign** (in its saves, not the profile): the
  game mode (`death-and-difficulty.md`) and the tutorial level
  (`tutorial.md`), and Ironman if 0049 adds it.
- **What stays with the computer** (no profile): which profile was last
  used, and the title screen's backdrop order (*Claude's starting rule*;
  neither is an option).
- **First launch** makes a profile called `Player 1` without asking, then
  runs the first-launch setup (the hand-layout picker) as today. A player
  who never shares the computer never has to see profiles.
- **At launch** the game opens on the last profile used.
- **A `Profile` item on the title menu** (*Claude's starting rule*: below
  `Options`) lists the profiles with the current one marked, and can
  **switch** to one, **add** one, **rename** one and **delete** one.
  - **Adding** asks for a name, then runs the first-launch setup for the
    new profile; a new profile starts from the default options
    (*Claude's starting rule*: each person has their own hands and keys).
  - **Deleting asks yes/no first** (the *Asking first* rule), naming that
    its saves go with it, with the focus on No. The only profile left
    can't be deleted (*Claude's starting rule*).
  - A name is up to 12 characters (*tunable*); two profiles can't share a
    name (*Claude's starting rule*). Up to 8 profiles (*tunable*).
  - **Typing a name:** on a keyboard, typed letters. On a controller, an
    on-screen letter grid for now; **on Steam (and any console release),
    the platform's own on-screen keyboard** instead (Nick, PR #264; on
    Steam, its gamepad text input, ticket 0903). The same goes for every
    place the player types, the lead's name too.
- **Keeping changed options is the player's choice** (Nick, PR #264).
  A change on the Options screen shows at once (as today, so the player
  sees and hears it), but it is **kept in the profile only when the
  player says so**. Leaving Options with anything changed asks
  `Keep these changes?` with three answers: **Keep** (kept in the
  profile), **Discard** (everything goes back to how it was when the
  screen opened) and **Back** (stay on the Options screen). The cursor
  starts on Back, so neither keeping nor throwing away happens by a stray
  press (*Claude's starting rule*). Leaving with nothing changed asks
  nothing. No save game is needed (Nick: fine now).
- The same goes for the **Profile screen** and anything else that edits
  the profile: backing out of a name box with a name typed and not kept
  asks whether to keep it, throw it away or go back, cursor on going back
  (*Claude's starting rule*, from Nick's "let them make that choice").
- **The Zoom map key** in battle changes the zoom and keeps it at once:
  pressing it is already the player's choice, and it changes back with one
  key (*Claude's starting rule*).
- **Options opened from the title** change the current profile's options;
  opened in a game, the same, since the profile is the one playing.
- **A New Game** uses the profile's options as they are; nothing is reset.
- **Players from before profiles:** their options, keys, tips seen, saves
  and suspend save become the first profile, `Player 1`. Nothing is lost.
- **The tutorial level row** (`Tutorial: Full / Short / None`) is on the
  Options screen while a campaign is loaded (in a battle, at Preparations),
  and changes that campaign's level, any time; it doesn't show on the
  title's Options. It has no yes/no of its own (a value row); like every
  row it is kept or discarded when leaving Options. Lessons already
  skipped stay readable in *Guides*.

### Open sub-questions

- None.

## Another window in front

Decided: 2026-11-06
Source: ticket 1246

### Nick's words

Asked what the game does when the player switches to another window
(keep running with sound; keep running muted; pause and mute; a choice in
Options), Nick picked **"Pause and mute"**.

### Rules

- When the game's window loses the focus (alt-tab, a click on another
  app, another browser tab), the game **pauses**: the picture stays on
  its last frame, nothing moves, and keys and buttons do nothing.
- It is **silent** meanwhile: music, sounds and voices. The music keeps
  its place.
- When the window has the focus again, everything continues where it
  stopped, at the player's volumes; the time away is not caught up.
- It is the same for every player: **no Options row**.
- Not on the Playdate, which has its own pause when the system menu opens.

### Open sub-questions

- None.

## Saves outside the game

Decided: 2026-11-11
Source: ticket 0872 (export), for 0873 (import) and 0914 (the demo's
export)

### Nick's words

Asked where *where saves are*, Export and Import live (the Profile
screen, a Saves page in Options, or the Load Game list), what one file
holds, and what an import does over a profile with saves:

> "as long as Vix gives a tutorial for it, I wouldn't mind both the
> Profile and the Load Game options. Profile --> when hovering, has option
> to export. Load Game --> when hovering, the same. When Load Game -->
> export is selected, it asks if it should export associated
> Options/Keybinds as well and can export a trimmed Profile containing
> only the 1 save. When Profile export is selected it will export the
> whole profile and all saves (more bytes). When importing, probably thru
> either channel, with a warning if they Import a Profile wrapped save(s)
> thru the Load Game channel, saying it will add a new profile"

What one file holds: "depending on export channel". Importing over a
profile with saves: **as a new profile**, never touching the one in use.

On Vix's part: "both [a tip the first time on each screen] but also a
quick, persistent Vix note when hovering profile or load game, that
doesnt draw cursor focus or require any player tutorial input. just a
note like "Did you know? You can export your [profile / save] from this
menu!" something like that". He checked the mockup of the note and of the
Load Game question ("Yes, as drawn").

On a one-save file imported through Load Game: "first empty slot, if none
empty, slot under cursor with confirmation default to no".

On where the saves are shown (mockups of three places): "the save should
never default to AppData probably should be in Documents or My Games
folder which is more accessible... otherwise probably both but the
browser storage one doesn't make a whole lot of sense it would be better
if it only displayed the helper text in Desktop build & for browser when
we export I imagine it would become a download".

### Rules

- **Export from the Profile screen:** with the cursor on a profile, the
  export key exports **the whole profile**: its 30 slots, its suspend
  save, its options, key bindings and layout, and the tips it has seen.
- **Export from Load Game:** with the cursor on a save, the export key
  asks `Export your options and key bindings too?` (Yes / No, the cursor
  on No), then exports **a trimmed profile holding only that save**, plus
  the options, key bindings and layout if the answer was Yes. An empty
  slot, or one that can't be read, has nothing to export.
- **One kind of file:** both are a profile's file, the Load Game one
  with less in it, so either opens through either screen's Import (0873).
- **On the web an export is a download** (the browser's downloads
  folder). On the desktop it is written in the `exports` folder beside
  the saves. A line after it says which: "Downloaded …", or the file's
  path with the action that opens its folder.
- **Where the saves are** is shown **on the desktop only**: a line under
  the Profile list and under the Load Game list with the folder's path
  and the action that opens it. The web shows no such line.
- **The desktop's saves folder** belongs in `Documents\My Games`, never
  `AppData` (Nick: "more accessible"). Ticket 0875 moves it, and the
  saves already in `AppData` with it.
- **Importing:**
  - Through **Profile**, a file always becomes **a new profile**.
  - Through **Load Game**, a file holding **one save** goes into **this
    profile**: the first empty slot; if all 30 are full, the slot under
    the cursor, after a yes/no question with the cursor on No.
  - Through **Load Game**, a file holding **more than one save** (a whole
    profile) warns first that it will be added as a new profile.
  - **Before anything is written** (0873) a report lists each save in the
    file (its chapter and playtime, as Load Game shows them) with what
    the game can do with it: "Ready" ("goes in slot 3" / "replaces slot
    3" through Load Game), "From an incompatible version (save version 1,
    this game reads 2)", or "Can't be read"; a line says when another
    version of the game made the file. Then "Import these saves?" with
    Import / Cancel, the cursor on Cancel. Every save marked Ready is
    imported; the others are left out, and a notice after says how many
    of each.
- **Vix** (ticket 0874): a tip the first time each screen opens with
  something to export, and on the title, while the cursor is on `Load
  Game` or `Profile`, a note beside the menu ("Did you know? You can
  export a save from this menu!" / "… your profile …"), every time, never
  taking the cursor and asking for nothing.
- **The Playdate** has no export or import (no files a player reaches).
- *The agent's starting rules:* the export key is the **Unit info** key
  (on menus it has no other use; `E` / `I` by default, the top face
  button on a pad), named in each screen's help line. The file is called
  `visions-of-shuyi-<profile name>-<n>.ron` for a profile and
  `visions-of-shuyi-<profile name>-<n>-slot-<NN>.ron` for one save (the
  name in lower case, spaces as `-`, any character but `a`–`z` or a
  digit left out; `<n>` the profile's number, so a name with no such
  letters still gives a name), and the desktop never writes over an older
  file (`… (2).ron`).
- *The agent's starting rules for import* (0873):
  - The import key is the **End turn** key (`Space`, the pad's `Start`),
    named in each screen's help line. On a desktop the player drops the
    file on the window; on the web the browser's file picker opens.
  - A new profile is named from the file's name (`visions-of-shuyi-odo-kai-2.ron`
    gives `Odo Kai`), with a number after it if that name is taken
    (`Player 1 2`), or the next `Player <n>` if the file names none.
  - "A file holding one save" is a file with one save slot and no
    suspended battle; anything else counts as a whole profile.
  - A save imported into this profile through Load Game comes without
    the file's options and key bindings (the report says so): this
    profile keeps its own.

### Open sub-questions

- None.
