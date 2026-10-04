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

- **Animation speed: Fast** plays walks, fights and the EXP bar ×2
  (*tunable*). **Enemy phase speed: Fast** plays the enemy's and the other
  side's turn ×2 (*tunable*); with both Fast, ×4.
- **Holding Confirm plays at ×4, not on top.** With Fast animations a
  fight plays ×2, and holding Confirm makes it ×4, not ×8. If the settings
  already play ×4 (an enemy turn with both set to Fast), holding Confirm
  changes nothing (*Claude's reading* of "just change the 2x to a 4x").
- **Text speed:** Slow is half of Normal, Fast is twice it, Instant shows a
  whole text box at once (*Claude's starting rule*).
- **Combat animations: Off** skips every fight the way the Cancel key skips
  one; the EXP bar and level-up pages still show (*Claude's starting
  rule*).

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
