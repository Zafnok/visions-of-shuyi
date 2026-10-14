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

### Volumes

- Music and sound volume each go from **0 (silent) to 100**, starting at 80
  (*tunable*).
- **A slider:** left and right move it 5 at a time (*tunable*).
- **A box for the exact number:** Confirm on the row opens it; type the
  number, Enter keeps it, Backspace deletes, Escape leaves the volume as it
  was. More than 100 counts as 100; an empty box changes nothing. On a
  controller there are no number keys, so the box shows the volume and the
  cursor moves it 1 at a time, Confirm keeps, Cancel leaves (*Claude's
  starting rule*).

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

### Rules

- **A player profile** holds that player's options and saves: every
  option on the Options screen (text, animation and enemy-phase speed,
  combat animations, auto end turn, fullscreen, cursor style, map look,
  map zoom, color theme, music and sound volume, and later the language
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
- **Changes are kept at once**, in the current profile, as today: no
  save needed (Nick: fine now).
- **Options opened from the title** change the current profile's options;
  opened in a game, the same, since the profile is the one playing.
- **A New Game** uses the profile's options as they are; nothing is reset.
- **Players from before profiles:** their options, keys, tips seen, saves
  and suspend save become the first profile, `Player 1`. Nothing is lost.
- **The tutorial level row** (`Tutorial: Full / Short / None`) is on the
  Options screen while a campaign is loaded (in a battle, at Preparations),
  and changes that campaign's level at once, any time; it doesn't show on
  the title's Options. It doesn't ask first (a value row, changed back with
  one key). Lessons already skipped stay readable in *Guides*.

### Open sub-questions

- None.
