# Controls: layouts and key bindings

Decided: 2026-09-25
Source: ticket 0015

## Nick's words

> "I want to be more interactive with our keybinds so pls ask me for each
> action what the key should be or for directionals etc. Also I'm envisioning
> an option for right or left handed play. Right handed might have cursor as
> arrow keys and left handed as wasd"
>
> **Q1. Which layouts should the game offer?** "A - I just used vim as an
> example meaning "no mouse" or "keyboard power user" but exact keybinds I
> don't care about. I think A should work fine."
>
> **Q2. Which layout on first launch?** "C" (ask on first launch)
>
> **Q3. Right-handed confirm / cancel.** "I think we can use the ASDF row for
> common actions. So maybe to start F is select/confirm and D is cancel."
>
> **Q4. A faster way to move the cursor?** "C for right now, with a note it
> might be revisited after playtesting a chapter with larger map fights"
>
> **Q5. Right-handed keys per action.** "confirm F, cancel D, I think cycling
> can take place on A and S, end turn can be space bar, with confirm by
> another spacebar (double tap space ends turn -- kinda satisfying), E can be
> unit info, W can be show danger zone, shift+space can be auto end"
>
> **Q6. Left-handed layout.** "A mirroring what I already said for 5"
>
> **Claude's three small calls** (Esc also cancels; literal finger mirror for
> previous/next unit; Rewind on R / U): "I think the small calls you made
> make sense"

## Rules

**Keyboard only.** The game is played entirely from the keyboard (no mouse
needed). Key choices are Nick's; the defaults below are his.

### Two layouts

- **Right-handed:** the right hand steers with the **arrow keys**; the left
  hand acts from the `A S D F` row.
- **Left-handed:** the left hand steers with **`W A S D`**; the right hand acts
  from the `J K L ;` row, an exact finger-for-finger mirror of right-handed.
- **First launch:** before anything else, a one-screen **"Pick your layout"**
  menu shows both layouts (with a small key diagram) and asks the player to
  choose. The choice is saved and can be changed later in Options.
- Individual keys stay rebindable in Options (tickets 0805, 0815), on top of
  the chosen layout: see *Rebinding keys* below.

### Bindings

| Action | Right-handed | Left-handed | Notes |
| ------ | ------------ | ----------- | ----- |
| Move cursor | arrow keys | `W A S D` | Held keys repeat |
| Confirm / select | `F` | `J` | Hold to fast-forward text and animations |
| Cancel / back | `D` | `K` | Skips a fight's playback (0418) |
| Previous ready unit | `A` | `;` | Finger mirror of `A` (*see note*) |
| Next ready unit | `S` | `L` | Finger mirror of `S` |
| Unit info (stat screen) | `E` | `I` | |
| Enemy danger zone on/off | `W` | `O` | |
| End turn | `Space` | `Space` | Double-tap: see below |
| Auto-end on/off | `Shift+Space` | `Shift+Space` | Replaces the `Shift+e` placeholder in `turn-structure.md` |
| Rewind (ticket 0307) | `R` | `U` | Proposed by Claude, approved by Nick: `R` was already planned; `U` is its mirror |
| Zoom map (ticket 0439) | `Q` | `P` | Nick, 2026-10-12: see *Zoom map* below |
| Open the dialogue log (ticket 0729) | `B` | `N` | Nick, 2026-10-14: see *Dialogue log* below. In its own group: may share a key |
| Map menu | `Cancel` with nothing to cancel, or `Confirm` on an empty tile | same | Unchanged from before |

*Proposed by Claude, approved by Nick:*

- `Esc` also works as Cancel in both layouts (and so opens the map menu when
  there's nothing to cancel), since players reach for it by habit. Since
  ticket 0217 it is a *fixed* key (no rule change): it isn't one of
  Cancel's key slots and can't be moved or removed; see *Rebinding keys*.
- "Mirror" is taken literally, finger for finger, so in the left-handed
  layout *previous* unit is the right-most key (`;`) and *next* is `L`. If
  that feels backwards, swap them.

### Zoom map

Playtest round 3 (2026-10-16): starting at 2× stays ("Otherwise 2x is ok
to start on"), with a zoom tip on the first turn (ticket 0466) and the
arrows at the view's edges (0449).

Decided 2026-10-12, ticket 0439 (the zoom itself: `look-and-feel.md`,
*Zoom*). Nick was shown Chapter 1's first turn at 1× and at 2× and asked
four things:

> **Which key toggles the zoom by default, in each layout?** Options:
> `Q` / `P` (finger mirrors, the top row beside the danger-zone key);
> `G` / `H` (beside Confirm); `Tab` in both; no default key. "Q / P"
>
> **Which controller button?** Options: right trigger; pressing the right
> stick; no default button. "what about bumper? or is it already bound"
> [told both bumpers are Previous / Next ready unit, his 2A pick, and
> offered the right trigger, the right stick's press, zoom on the right
> bumper with Next unit moved to the right trigger, or no button:]
> "Right trigger"
>
> **Must Zoom map always have a key?** Options: optional, like Unit info
> and Rewind; required. "Optional"

- **`Q`** (right-handed) and **`P`** (left-handed) switch the battle map
  between 1× and 2×. On a controller: the **right trigger**.
- It is **optional**: it may be left `! not mapped`; the zoom can also be
  set in Options (`Map zoom`, `options.md`).
- It only does something where the map is drawn with pictures. With the
  glyph look the key does nothing and the help doesn't list it.

*Claude's starting rules (Nick can veto):*

- **The key works whenever the map is what you are looking at**: browsing,
  with a unit selected or walking, while picking a target, and during the
  enemy's turn. It does nothing while a menu or a yes/no question is
  open, during a fight's playback, or on the rewind screen. Example: Mia
  is choosing who to attack and presses `Q`: the map zooms out and she is
  still choosing. With the action menu open, `Q` does nothing.
- **After a switch the cursor's tile is in the middle of the view**, or as
  near it as the map's edges allow.
- **The hint** is first on the toggles line above the key help: `q zoom:
  2× · w danger zone: OFF · Shift+Space auto-end: OFF` (ticket 0445
  decides where that line's hints sit). No sound and no message: the map
  changing is the answer.
- **On the Key bindings screen** `Zoom map` is the row before `Map menu`.
- **"Pick your layout"** lights `Q` and `P` on its keyboards like the
  other keys in use; its list of keys is unchanged (it has no free line).

### Dialogue log

Decided 2026-10-14, ticket 0729 (the log itself: `look-and-feel.md`,
*Dialogue log*). Nick first said the key is **"optional binding"**, then:

> "up will be default, it will be optional bind though, so players can
> clear it if they want but it will have a default."

Told that Up is already Cursor up, and that a key works for only one
action:

> "make it a deliberate entry in the log rather than Cursor Up / Log Up =
> up, make it 2 rows, and they are in different context menus, so they can
> overlap. We should have systems to support this behavior across other
> keys as well. Also, probably, if Log Up is bound and Log Down is not then
> that should error and vice versa"

Asked what to do at the lead's replies, where Up moves between them:

> "then maybe default shouldn't be cursor up, what does persona bind to? we
> can use the same. but if the player decides to bind to cursor up then
> replies win"

Told that Persona 5 Royal opens its log with one key (`B` on PC, per a
keyboard guide) and scrolls it with the normal up and down, he picked
**"One key, like Persona"**, and its own group on the Key bindings screen
(**"Own group, block leaving"**, which with one key has nothing to block).

- **Open the log** is an optional action: **`B`** right-handed, **`N`**
  left-handed (its finger mirror), and the **left face button** on a
  controller (`X` on Xbox, `□` on PlayStation, `Y` on Switch-style pads).
  It opens the log while a scene plays. In the log the cursor keys move,
  Confirm jumps to the line, Cancel closes; the log key itself does
  nothing there.
- **Keys are in groups, and only keys in the same group clash.** An action
  in one group may have a key or button that an action of another group
  has, because the two are used in different places. Binding a key moves
  it only from an action of the same group. Today there are two groups:
  everything before, and **In the dialogue log** (Open the log). So the
  pad's left face button is both Danger zone (on the map) and Open the log
  (in a scene). Later actions can get groups of their own the same way.
- **What the scene uses wins.** If the log's key is also a key the scene is
  using at that moment (Confirm, End turn and Cancel always; the cursor
  keys while the replies are up), pressing it does the scene's thing, not
  the log. Example: Mia binds Open the log to `Up`. Reading, `Up` opens
  the log; at the replies, `Up` moves between them.
- **On the Key bindings screen** the group is the last one, after
  Optional, headed `In the dialogue log`. (*Claude's starting rule:* to fit
  the extra group, the blank row between the keyboard / controller switch
  and the slot columns' headings is gone.)

### Fight playback: Cancel skips, hold Confirm speeds up

Decided 2026-09-28, ticket 0418, after playing the 0404 build (where a
Confirm tap skipped and holding it sped up):

> "I don't think skipping and fast playback of a fight should be bound to
> same hotkey, probably use the cancel hotkey (i.e. D) to skip and the
> confirm hold key (F) to playback faster"

- **Cancel** (`D` / `K`, or `Esc`) skips the rest of a fight's playback.
- **Holding Confirm** plays it ×4 *(tunable)*. A Confirm tap does nothing.
- With the Options screen's **Fast** animations (×2, ticket 0805), holding
  Confirm still plays ×4: it replaces the ×2, it doesn't multiply it
  (Nick, 2026-10-04: "holding confirm should not speed up on top, but just
  change the 2x to a 4x"; `options.md`).

### Dialogue: a tap turns the page, a hold speeds up

Nick, playing Chapter 1 (2026-10-12, ticket 0727):

> "the hold F to speed up text is too eager, I press F to advance to next
> dialogue and already half the next text is displayed bc it interprets a
> tap as a hold too quickly"

- Holding Confirm speeds the text up only once it has been held for
  **0.3 s on that page** *(Claude's starting rule, tunable)*. Example:
  Nick taps `F` to turn the page; the next line starts at the normal
  speed. He keeps `F` down through three pages; each one starts at the
  normal speed and goes fast a moment later.
  Nick, after playing with it (2026-10-14, 0804): "it's fine". Kept.

### End turn: double-tap Space

- `Space` opens the end-turn prompt (`End turn with N units ready?`).
  `Space` again ends the turn; Confirm also accepts and Cancel backs out.
- As `turn-structure.md` already says, when no units are ready there's
  nothing to warn about, so `Space` ends the turn at once.

### Cursor speed

- **No fast-cursor key** for now: no "jump ×5" and no hold-to-scroll-faster.
  Held keys repeat (first repeat after 300 ms, then every 55 ms, *tunable*; the first delay
  was 170 ms until ticket 0421: Nick found a single tap in menus often moved twice),
  and Next/Previous unit jump straight to your units.
- **Revisit after playtesting** a chapter with large-map fights (0804 or
  later). If crossing big maps feels slow, the options were: hold a key to
  scroll faster (FE's hold-B), or jump several tiles per press.

### Rebinding keys

Decided 2026-09-29, ticket 0030. Nick's words:

> "we should allow remapping keys in the options menu. […] 1) never
> hardcode keyboard input, draw from the config which can be set by user
> 2) keyboard mapping screen, take any keyboard input to remap, check it's
> not already taken, if it is, then overwrite but make the one that was
> taken before have a flag like ! not mapped"
>
> "only some keys are necessary to map like cursor, select/confirm, cancel,
> end turn"
>
> "we can have some that are optional like if user wants separate select
> (cursor) and confirm (action) keys, or separate end turn / confirm end
> turn keys (right now both can be space)"
>
> Follow-up answers:
> - Leaving the screen while a required action has no key: **"Block leaving"**.
> - Keys per action: **"3 each, only one needs to be filled out."**
> - Custom keys when switching right/left-handed: **"Each layout keeps its own"**.
> - Backing out of "Press a key…": **"Esc backs out, not bindable"**.
> - Emptying a slot: **"Delete only"**.

**Where:** a **Key bindings** screen opened from Options (ticket 0805).

**Slots.** Every action has **3 key slots**. An action works with any of the
keys in its slots. Only one slot needs a key.

**Required and optional actions.**

| Required (must keep at least one key) | Optional (may have no key) |
| ------------------------------------- | -------------------------- |
| Cursor up, down, left, right | Select *(new, see below)* |
| Confirm | Confirm end turn *(new, see below)* |
| Cancel | Previous / next ready unit |
| End turn | Unit info, danger zone, auto-end on/off, rewind, map menu, zoom map *(0439)*, open the dialogue log *(0729)* |

The developer Debug key is not on the screen.

**Binding a key.**

1. Pick an action's slot and press Confirm: the slot shows `Press a key…`.
2. The next key pressed goes in that slot. Any key the game can read counts,
   with or without `Shift`, except `Esc` and `Delete` (below).
   `Shift+Esc` and `Shift+Delete` are ordinary keys and can be bound
   (Nick, 2026-09-30, ticket 0217: only plain `Esc` and `Delete` have a
   job, so nothing conflicts with their `Shift+` chords).
3. **If that key is already in another slot, it moves:** the old slot is
   emptied. An action left with no keys at all shows **`! not mapped`**.
   (Same layout only; the other layout's keys are separate.)

**`Esc` is fixed.** `Esc` always backs out of `Press a key…` without
changing anything, and always works as Cancel everywhere, in both layouts.
It can't be put in a slot; the screen shows it as a fixed extra key on
Cancel.

**`Delete` empties** the highlighted slot. `Delete` can't be put in a slot.

**Leaving is blocked** while any *required* action shows `! not mapped`:
backing out of the screen shows a message naming the action and stays put.
Optional actions may be left `! not mapped`; that action then has no key
until the player binds one.

**Each layout keeps its own keys.** Custom keys are saved per layout.
Switching right/left-handed in Options loads that layout's keys (its own
custom keys if it has any, else its defaults); switching back brings the
first layout's custom keys back.

**Optional split keys.** Both start with no key, so the defaults behave
exactly as before:

- **Select (cursor)**: picking things *on the map with the cursor*
  (choosing a unit, its destination tile, a target; Confirm on an empty tile
  opening the map menu). With no key, Confirm does this. Once Select has a
  key, Confirm stops doing it on the map and only accepts menus, prompts
  and the forecast.
- **Confirm end turn**: accepting the end-turn prompt. With no key, pressing
  End turn again accepts it (today's double-tap Space). Once it has a key,
  End turn pressed again no longer accepts; the new key does. Confirm still
  accepts and Cancel still backs out, as before.
- **The split is per device** (*Claude's starting rule (Nick to veto at
  sign-off of 0230)*): a key for Select or Confirm end turn changes only
  what the keyboard's Confirm and End turn do. A controller's Confirm keeps
  picking on the map, and Start pressed twice keeps accepting the end-turn
  prompt, until that action has a *button* (ticket 0816). Example: Mia
  binds Select to `g`. On the keyboard `g` picks units and `f` only accepts
  menus; on her pad `A` still does both.

**Help bar and tips** show the player's current keys. An action with no key
shows as `! not mapped` there too.

**The screen's look** (ticket 0815, Nick picked from three rendered
mockups, 2026-09-30: "B: grouped panel", rows "Required first, then
optional"): one panel titled `Key bindings` (until ticket 0816 the title
also named the layout; it is now in the switch row under it, see
*Rebinding buttons*) with columns `Key 1`
`Key 2` `Key 3` and two groups. **Must have a key:** Cursor up, down, left,
right, Confirm, Cancel (`+ Escape` after its name), End turn. **Optional:**
Select, Confirm end turn, Previous ready unit, Next ready unit, Unit info,
Danger zone, Auto-end on/off, Rewind, Zoom map (since ticket 0439), Map
menu. Since ticket 0729 a third group, **In the dialogue log:** Open the
log. Then **Restore defaults**.
An empty slot is a small dot; `! not mapped` sits at the right of the row,
red for a required action and dim for an optional one. Screenshots:
[`0815-key-bindings.png`](../screenshots/0815-key-bindings.png),
[`0815-key-bindings-blocked.png`](../screenshots/0815-key-bindings-blocked.png).

**Restore defaults asks first** (Nick, 2026-09-30, on the 0815 build:
"have a confirmation screen for restore defaults"). Confirm on Restore
defaults opens `Restore the default keys for Right-handed?`; Confirm
answers yes, Cancel answers no and stays on the Key bindings screen with
nothing changed.

*Claude's starting rules (Nick to veto at sign-off of 0815):*

- `Esc` doesn't count as Cancel's required key: Cancel still needs one of
  its own slots filled, so it stays on the acting hand.
- Emptying a required action's last key with `Delete` is allowed (it then
  shows `! not mapped` and leaving is blocked), the same as losing it to
  another action.
- The blocked-leave message reads `Give <action> a key first`.
- A **Restore defaults** row puts the current layout's default keys back
  (the other layout is untouched).
- Which Select/Confirm presses count as "on the map" is Claude's reading
  of "select (cursor) and confirm (action)" above.
- The Key bindings screen is steered with the keys the player had when
  they opened it; changes take effect when they leave. This way moving
  every cursor key elsewhere can't trap them on the screen.
- On that screen, up/down wrap round (the row after Map menu is Restore
  defaults, then Cursor up again); left/right stop at the first and third
  slot.
- A key the game keeps for itself (only the developer debug key, in builds
  that have the debug menu) shows `That key can't be used` and the slot
  keeps waiting. Example: Mia presses the debug key at `Press a key…`;
  the message appears and she presses `g` instead.
- When a key moves, the action that lost it lights up white for 1.5
  seconds (*tunable*), so the player sees where it came from.
- If the key pressed for a slot is one of the player's old cursor keys
  and they keep it held, the highlight stays put until they let go.
- When two required actions have no key, the blocked-leave message names
  the one listed higher on the screen.
- The question's wording, `Restore the default keys for <layout>?`, and
  its look (the end-turn question's box, `f yes / d no` with the player's
  own Confirm and Cancel keys).

## Controller

Decided: 2026-09-30
Source: ticket 0032 (built by 0219 controller input, 0220 button names,
0816 rebinding buttons)

A controller is one more way to play: it produces the same actions as the
keys, and the keyboard keeps working alongside it. Mockups shown to Nick:
[`0032-controller-defaults.png`](../screenshots/0032-controller-defaults.png)
(the chosen layout) and
[`0032-button-names.png`](../screenshots/0032-button-names.png) (button
naming styles).

### Nick's words

> **Q1. Which button confirms and which cancels?** Options: A bottom
> confirms (Xbox / PlayStation / PC), B the Nintendo way (right confirms),
> C follow the controller. "1C"
>
> **Q2. The other buttons.** Options: A shoulders cycle units and Start ends
> the turn (pressed twice), like Wargroove; B triggers to look and End turn
> only from the map menu, like Fire Emblem; C the right trigger ends the
> turn and Start opens the menu. "2A"
>
> **Q3. Moving the cursor.** Options: A D-pad and left stick, one tile at a
> time with the held-key repeat, nothing on the right stick; B faster when
> the stick is pushed further; C D-pad only. "3A"
>
> **Q4. Button names in help bars and tips.** Options: A Xbox letters on
> every pad; B follow the pad (Xbox letters, PlayStation shapes, Switch
> letters); C position words. "4B"
>
> **Q5. Switching between keyboard and controller.** Options: A whatever
> was pressed last, like Wargroove and Into the Breach; B a setting in
> Options, like XCOM 2 on PC. "5A". (The "Pick your layout" sub-question
> went unanswered, so the recommended option, *skipped for controller
> players until they touch the keyboard*, is recorded; Nick may veto.)
>
> **Q6a. Rebinding rules.** Options: A same as keys, one controller setup
> for both keyboard layouts; B same, but per layout; C one button per
> action. "6a A"
>
> **Q6b. Backing out and emptying a slot with only a controller.** Options:
> A Start and Back fixed, like Esc and Delete; B hold any button to back
> out, and a Clear choice on the slot; C a countdown. "6b B with a hint
> showing this"
>
> **Q7. Rumble.** "for now, no rumble, but we might add it later"

### Default buttons

Buttons are named by **position**; the table gives the Xbox letter (and
the PlayStation / Switch name) for each position.

| Action | Button (position) | Xbox | PlayStation | Switch-style | Notes |
| ------ | ----------------- | ---- | ----------- | ------------ | ----- |
| Move cursor | D-pad **and** left stick | D-pad, stick | same | same | Both work; see *Cursor* below |
| Confirm / select | bottom face | `A` | `✕` | right face `A` | Nintendo pads swap: see below |
| Cancel / back | right face | `B` | `○` | bottom face `B` | Nintendo pads swap: see below |
| Previous ready unit | left shoulder | `LB` | `L1` | `L` | |
| Next ready unit | right shoulder | `RB` | `R1` | `R` | |
| Unit info (stat screen) | top face | `Y` | `△` | `X` | |
| Enemy danger zone on/off | left face | `X` | `□` | `Y` | |
| End turn | Start | `Start` (☰) | `Options` | `+` | Press twice, like `Space` |
| Auto-end on/off | Back | `Back` (⧉) | `Create` / `Share` | `−` | The toggle shortcut `turn-structure.md` asks for |
| Rewind (ticket 0307) | left trigger | `LT` | `L2` | `ZL` | |
| Zoom map (ticket 0439) | right trigger | `RT` | `R2` | `ZR` | Nick, 2026-10-12 (*Zoom map* above) |
| Open the dialogue log (ticket 0729) | left face | `X` | `□` | `Y` | Shared with Danger zone: another group (*Dialogue log* above) |
| Map menu | *no button*: Cancel with nothing to cancel, or Confirm on an empty tile | | | | Same as the keyboard |
| Select *(optional, 0218)* | *no button* | | | | Starts empty, as on the keyboard |
| Confirm end turn *(optional, 0218)* | *no button* | | | | Starts empty: Start pressed again accepts |
| Debug | *no button* | | | | Keyboard only |

Unused by default: right stick, pressing either stick. (The right trigger
was unused until ticket 0439 put Zoom map on it.)

**Confirm and Cancel follow the controller (Q1 C).** On Xbox, PlayStation,
Steam Deck and unknown pads the **bottom** button confirms and the
**right** one cancels. On **Switch-style pads** (Nintendo Switch Pro
controller and similar) the two swap: the **right** button (labelled `A`)
confirms and the **bottom** one (`B`) cancels, as in Fire Emblem on Switch.

### End turn on the pad

Same as `Space`: `Start` opens the end-turn prompt (`End turn with N units
ready?`), `Start` again ends the turn, Confirm also accepts and Cancel backs
out. With no units ready, `Start` ends the turn at once.

### Cursor

- The **D-pad and the left stick both move the cursor, one tile per
  step**. A stick pushed past its dead zone counts as a held arrow key and
  repeats with the **same timings as held keys** (first repeat after
  300 ms, then every 55 ms, *tunable*, shared with the keys). How far the
  stick is pushed doesn't change the speed.
- The stick moves in 4 directions only (no diagonals), like the keys.
  *Claude's starting rules (ticket 0219, Nick can veto):*
  - A stick pushed diagonally moves the cursor only the way it is pushed
    **furthest**. Example: pushed up and a little to the right, the cursor
    goes up, never up-and-right.
  - The dead zone: a stick counts as pushed from **half way** out, and
    stops counting once it falls back to about **a third** (*tunable*,
    `stick` in `keymap.ron`), so a stick resting near the edge doesn't
    stutter.
- **The right stick does nothing by default**, but the player may bind
  its 4 directions to any action (see *Rebinding buttons*). No fast
  cursor button either (the
  keyboard's *Cursor speed* note applies to the pad too: revisit after the
  large-map playtest).

### Button names on screen (Q4 B)

Help bars and tips name buttons the way **the pad in use** labels them:

| Pad | Names shown |
| --- | ----------- |
| Xbox, Steam Deck, unknown or generic pads | `A B X Y`, `LB RB LT RT`, `Start`, `Back` |
| PlayStation | `✕ ○ □ △`, `L1 R1 L2 R2`, `Options`, `Create` (PS4: `Share`) |
| Switch-style | `A B X Y` (Nintendo positions), `L R ZL ZR`, `+`, `−` |

- Example (battle help bar, Xbox pad): `A select · Y info · RB next unit ·
  LT rewind · Start end turn`; the same on a PlayStation pad: `✕ select · △
  info · R1 next unit · L2 rewind · Options end turn`.
- The PlayStation shapes `□` and `△` (and a bolder `✕`, `○`) aren't in
  the font: they are drawn as new glyphs in its style (ticket 0220, mockup
  first; the look Nick picked is under *Notes from building it (ticket
  0220)* below).
- An action with no button shows `! not mapped`, as for keys.
- **Moving the cursor** names both the D-pad and the stick (Nick: "just
  render both somehow"): `D-pad/L-stick move` with the defaults (`L-stick`
  since ticket 0816, when Nick asked for both sticks to be named). If the
  player rebinds the cursor, the help names whatever it's bound to,
  like the keyboard's `arrows` / `wasd`.

### Switching between keyboard and controller (Q5 A)

- Help bars and tips show **whatever was pressed last**: press a pad
  button and they name buttons; press a key and they name keys again.
  No setting.

### Pick your layout with a controller

Nick (2026-09-30):

> "If they press keyboard key --> show the layout picker if they have not
> already decided. If they press controller --> skip the layout picker.
> Also skip it if the player pressed keyboard then presses controller
> after. But if they go back and forth, then until they select the layout
> they prefer, it should pop up once they press a keyboard key."
>
> When a key is pressed mid-battle while undecided (A straight away, over
> the battle / B at the next quiet moment): "A"

Every build now starts with `Press any key or button` (`title-screen.md`).

1. **That first press decides.** A **key** opens "Pick your layout" (if
   no layout has been chosen yet). A **controller button** skips it.
2. **A button while "Pick your layout" is open closes it** and play goes
   on with the controller.
3. **Until a layout is picked**, pressing a key after using the controller
   opens "Pick your layout" **straight away, wherever the player is**
   (over a battle too; the battle waits behind it). The key that opened
   it does nothing else.
4. **Once a layout is picked** it is saved and never asked again, however
   often the player switches.

Example: Mia presses `A` on her pad at the title, so no layout screen. Two
battles later she presses `F` and "Pick your layout" opens over the map.
She picks right-handed, and it never shows again.

*Claude's starting rules (ticket 0226; Nick can veto):*

- **Until a layout is picked, no key does anything but open "Pick your
  layout"**, even one that would otherwise work. Example: Mia closed the
  layout screen with her pad; an hour later she presses `Space`, and the
  layout screen opens again instead of ending her turn.
- **The button that closes "Pick your layout" does nothing else**, and
  nudging a stick counts as a button. Example: Mia presses `A` while the
  layout screen is over a battle: it closes, and no unit is selected.
- **A controller player can still pick a layout on purpose** from Options:
  opened there, the screen is steered with the pad as before and a button
  doesn't close it.
- **Typing with no layout picked** (the lead's name) is a key press like
  any other: the layout screen opens first.

### Rebinding buttons (Q6)

Same rules as *Rebinding keys* above, on the same Key bindings screen:

- Every action has **3 button slots**; only one needs a button. The
  defaults fill 2 slots of each cursor direction (D-pad and left stick).
- A button already in another slot **moves**; an action left with no
  button shows **`! not mapped`**.
- **Required** (must keep a button): cursor up, down, left, right, Confirm,
  Cancel, End turn. Leaving the screen is **blocked** while one shows
  `! not mapped`. The rest are optional.
- **One controller setup for both keyboard layouts**: switching right/left-
  handed changes only the keys, never the buttons.
- **No fixed buttons**: every button can be rebound (unlike `Esc` and
  `Delete` on the keyboard). With only a controller:
  - **Backing out** of `Press a button…`: **hold any button** for about a
    second (*tunable*). Nothing changes.
  - **Emptying a slot**: Confirm on the slot offers **`Clear`**.
  - A **hint** on the screen says so while capturing, e.g. `Press a
    button… · hold any button to cancel`.
- **Every button and both sticks' directions can be bound**: the left
  stick's 4 directions and the right stick's 4 directions each count as a
  button (the right stick starts with nothing on it).
- **Rebinding by tapping:** a button goes into the slot when you let go of
  it. Held for a second, it backs out instead (so a hold never binds).
- The **`Clear`** choice appears only when the pad was used last. With the
  keyboard, Confirm on a slot still goes straight to `Press a key…` and
  `Delete` empties it, as decided in 0030.

**The look and the names** (ticket 0816, Nick picked from rendered
mockups, 2026-10-03). Screenshots of the built screen:
[`0816-controller-buttons.png`](../screenshots/0816-controller-buttons.png),
[`0816-change-or-clear.png`](../screenshots/0816-change-or-clear.png),
[`0816-press-a-button.png`](../screenshots/0816-press-a-button.png).

> **How the screen shows buttons.** Options: A it shows whatever you
> pressed last, no switch; B a `Keyboard | Controller` switch row at the
> top; C a small menu first, then one screen each (Celeste, Hollow
> Knight). "B: switch row"
>
> **Names of stick directions and presses.** Options: keep `stick ↑` /
> `R-stick ↑`; name both sticks; short `L↑` / `R↑`. "Name both sticks"
>
> **The two lines of the choice on a slot.** Options: `Change` / `Clear`;
> `Rebind` / `Clear`; a choice only on a filled slot. "Change / Clear"

- **A switch row** under the title: `Keyboard · <layout>` and
  `Controller`. The side shown is highlighted. It is one row up from the
  first action; there, left shows the keys and right the buttons. The rest
  of the screen is the same panel for both: columns `Button 1` `Button 2`
  `Button 3`, `Must have a button`, `Optional`, `Restore defaults`.
- **Stick names:** the left stick's directions are `L-stick ↑ ↓ ← →`, the
  right stick's `R-stick ↑ ↓ ← →`; pressing a stick in is `LS` / `RS`
  (PlayStation: `L3` / `R3`). The help bar says `D-pad/L-stick move`.
- **With a controller, Confirm on any slot opens `Change` / `Clear`** (an
  empty slot too, so the screen behaves one way). `Change` goes to `Press a
  button…`; `Clear` empties the slot; Cancel closes the choice.

*Claude's starting rules (ticket 0816; Nick can veto):*

- **The screen opens on the side you pressed last**: buttons after a
  button press, keys after a key press.
- **Confirm on the switch row** also flips the side, like left / right.
- **Hold time:** 1 second (*tunable*) backs out of `Press a button…`. Let
  go sooner and the button goes in the slot. Example: Mia picks `Change`
  on Unit info, taps `RT`, and `RT` is Unit info; had she held `RT` for a
  second, nothing would have changed.
- **Leaving is blocked for either side.** The message reads `Give Confirm
  a button first` (or `a key first`), and the screen switches to the side
  that lacks it. Example: on the keyboard side, Mia presses Cancel while
  Confirm has no button: the screen flips to the controller side and says
  `Give Confirm a button first`.
- **Restore defaults restores only the side shown**, after asking
  `Restore the default buttons?` (the keys' question is unchanged).
- **With the keyboard on the controller side**, Confirm goes straight to
  `Press a button…`, `Esc` backs out and `Delete` empties the slot, as for
  keys. A key pressed there isn't a button and does nothing.
- **With a controller on the keyboard side**, Confirm opens `Change` /
  `Clear` too; `Change` waits for a key, and holding any button backs out.
- **Until a controller is used**, the controller side shows Xbox names.
- **Rebound buttons count from the moment you leave the screen**, like
  keys; it is steered with the buttons you had when you opened it.

### Rumble

**None for now** (Q7). Nick: "we might add it later"; that would be its own
ticket.

### Follow-up answers (2026-09-30)

Claude proposed six small rules; Nick's replies:

> - Switch pads swap only Confirm and Cancel: "as long as it's consistent
>   with fire emblem that's ok"
> - Rebinding by tapping (bind on release, hold to back out): "ok"
> - `Clear` only when the pad was used last: "yes"
> - Stick directions bindable: "ok for left stick, but right stick should
>   be available to use if wanted"
> - Help bar naming the D-pad only: "just render both somehow"
> - Several pads at once: "sure"

- **Switch-style pads swap only Confirm's and Cancel's buttons**, which is
  exactly Fire Emblem on Switch (right `A` confirms, bottom `B` cancels).
  The other buttons keep the 2A layout on every pad, so on a Switch pad
  Unit info is the top button (labelled `X` there) and the danger zone the
  left one (`Y`). If you rebind, the swap still applies: whatever is on the
  bottom button of an Xbox pad sits on the right button of a Switch pad.
- **Several pads at once** all drive the game, as one player.

### Notes from building it (ticket 0219)

Controllers work on every build: the default buttons above, D-pad and left
stick, the Switch-style swap, several pads, plugging in and unplugging while
playing. Help bars and tips named keys until ticket 0220 (done, below);
rebinding buttons is ticket 0816 (done); the `Press any key or button` prompt and when "Pick
your layout" shows are ticket 0226 (done: *Pick your layout with a
controller* above, and `title-screen.md`).

- A pad counts as **Switch-style** when it says Nintendo made it. Other
  makers' Switch-shaped pads (8BitDo, PowerA…) count as ordinary pads:
  bottom confirms. Some browsers don't say who made a pad at all (Safari;
  Chrome for Xbox-type pads), and then it's an ordinary pad too.
- *Claude's starting rule (Nick can veto):* on the web build a controller
  button already dismisses `Press any key` (the line said only "key"
  until 0226), so a controller player isn't stuck on the title.

### Notes from building it (ticket 0220)

Help bars, prompts and tips now name the controller's buttons once a
button is pressed, and keys again once a key is pressed.

**The look of the PlayStation shapes.** Nick picked from four rendered
options ([`0220-ps-glyphs.png`](../screenshots/0220-ps-glyphs.png): A thin
and small, B thin and tall, C bold, D big and two cells wide), 2026-10-01:
**"A. Thin, small"**. Each shape is one cell, as tall as a lower-case
letter, drawn with thin lines: `✕ ○ □ △`.

*Claude's starting rules (the design didn't say; Nick can veto):*

- **Only a press that does something switches the text.** A key or button
  with no job doesn't. Example: Mia plays on her pad and bumps `q` on the
  keyboard, which does nothing: the help bar keeps naming buttons. She
  presses an arrow key: it names keys.
- **Two pads of different kinds:** the text names the buttons of the pad
  pressed last.
- **The D-pad's directions are shown as arrows** where a single direction
  is named. Example: choosing an attack's target and Combat Art reads
  `←/→ target · ↑/↓ art`.
- **PlayStation 4 pads show `Share`** for the left centre button, and
  PlayStation 5 pads `Create` (ticket 0229); every other name is the same
  on both.
- **The developer debug hint** still names its key on a controller (it has
  no button).
- **The "Pick your layout" key list** always shows keys, since it is about
  the keyboard.
- **The Key bindings screen** names buttons in its help line on a pad, but
  still only rebinds keys, and its `Escape` / `Delete` hints stay until
  rebinding buttons (0816, done: see *Rebinding buttons*).
- **Names nobody sees until buttons can be rebound (0816)**, chosen so
  every button has one: a stick's single direction is `stick ↑` (left
  stick; **`L-stick ↑` since 0816**, Nick's pick) or `R-stick ↑`; pressing
  a stick in is `LS` / `RS` (PlayStation:
  `L3` / `R3`); a cursor moved to the right stick reads `R-stick move`; a
  cursor on four unrelated buttons lists them, e.g. `Y/X/A/B move`.

**Found while building it:** giving the optional **Select** action a key
stops the controller's Confirm from picking on the map (and likewise
**Confirm end turn** and Start). The help bar then honestly says `! not
mapped select` on a pad. Ticket 0230 fixes it.

## Open sub-questions

- Fast cursor movement: revisit after a large-map playtest (above).
- Rumble: none for now; a later ticket if Nick wants it.
