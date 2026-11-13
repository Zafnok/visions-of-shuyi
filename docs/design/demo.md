# The Chapter 1 demo

Decided: 2026-11-11
Source: ticket 0914

What a player of the Chapter 1 web demo sees at its end, and what it
keeps of Chapter 1. Which files leave the demo and how is technical
(0914's ADR), not here.

## Nick's words

On what the player sees after winning Chapter 1:

> ch1 demo should allow a save to be exported, so probably a 2 step
> screen Thanks for playing ${name}, and then a selector Export Save?
> Yes/No and then work within our existing Export planned work. It
> should then be importable to main game.

On whether `ch01_tbc` (the rival's scene after the win) stays:

> if it's a ch1 ending scene and not a ch2 intro scene we can keep it.

On the "Save your progress?" question after Chapter 1:

> given saving the game does nothing to a chapter specified demo, I
> would prefer only having Export option on demos, NOT the save option
> on demos.

On which name the end card shows: the lead's name (the recommended
option).

## The rules

1. **The demo plays all of Chapter 1's scenes**, the after-battle ones
   too: `ch01_victory`, then `ch01_tbc`. `ch01_tbc` is listed in
   `assets-private/content/chapters/ch01.ron`'s `victory_scenes` and written in
   `ch01.dlg`: it is Chapter 1's ending, not Chapter 2's opening.
2. **No "Save your progress?" in the demo.** The full game asks it after
   Chapter 1 (0869); the demo goes from the last scene straight to the
   end card. The full game's question is unchanged.
3. **The end card:** "Thanks for playing, {the lead's name}" (the name
   the player typed at New Game). A press goes on.
4. **Then "Export save?" Yes / No**, a yes/no question like the game's
   others. *Yes* hands the player a file of their game as it stands
   after Chapter 1, through the save export 0872 builds (a download on
   the web). *No* goes on without one.
5. **The exported file is importable into the full game** (0873's
   import), and loading it there goes on to Chapter 2 as the full game's
   own Chapter 1 save does.
6. **Then back to the title.**

*The agent's starting rules* (Nick can veto):

- After *Yes*, the file is offered, then the game goes back to the
  title; the player isn't asked again.
- The question's cursor starts on *Yes*: exporting writes nothing over
  anything, so it is not the destructive choice the game's other
  questions keep the cursor away from.
- The end card's other words, its look and its music are the game's
  "To be continued" card's (silence, the same layout), with the heading
  replaced.

## Open

- Where Export and Import live in the full game, and what an exported
  file holds there: 0872's questions.
