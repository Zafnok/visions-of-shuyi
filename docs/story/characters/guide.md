# Vix, the guide

Id: `guide` · name in the [name registry](../names.md) (ticket 0731).

Decided by ticket 0057 (`docs/design/tutorial.md`, *The guide*); named and
given a voice by ticket 0731.

## Role

- **Role:** the tutorial mascot. She speaks every lesson and tip, to the
  player.
- **Outside the story for now.** She talks to the player, not to the
  party. She never appears in a scene, never fights, and nothing in the
  plot (deaths, Chapter 8) touches her. No one in the cast knows she
  exists.
- **Later, maybe in the story.** Nick: "outside for now, with decision much
  later if we want to tie it in (think - Angel from borderlands 2)". Angel
  guides the player in *Borderlands 2* and later turns out to be part of
  the plot. Until Nick decides that, write nothing that rules it in or out:
  no backstory, no claims about the world.

## Name

**Vix** (from "vixen", a she-fox). Chosen in ticket 0731, 2026-10-15.
Shown candidates (Kit, Rin, Sable, Pip), each with the first tip of
Chapter 1 rendered in that voice, Nick answered:

> "ok we can try Sable personality but I don't like the name it sounds like
> a grandma... and probably use Rin's facial expression in this screenshot"

Shown Vix, Fen, Vesper and Kit for Sable's voice:

> "vix, fen, kit are all ok... you can pick one I guess these are all data
> file entries now so it should be cheap to rename her whichever is fine"

Claude picked Vix. The name is only in the names table; the box title reads
it from there, and no line says it, so a rename is one line.

## Face

The **Fox Miko** hero from the bought Tiny Tales *Heroes 2: Rebellious
Souls* pack, with its 8 expressions (neutral, smile, stern, sad, surprise,
thinking, sly, unique). Imported by ticket 0455. No one else in the cast
uses this face.

Her usual look is **neutral**, the calm face (Nick: "Rin's facial
expression"). Every tip and lesson names its face (`face:`): `neutral` for
plain instructions, `sly` when she points out a trick or a way out,
`thinking` for something to weigh, `stern` for danger, `smile` only for a
level up.

## Voice

The dry insider (Nick picked "Sable's personality"): a teammate who has
seen this before and is glad to save you the trouble. Like Angel in
*Borderlands 2* or Mimir in *God of War*, she talks to you, not about
herself.

- **Short, easy sentences, with contractions.** "Over to you." "That's
  everyone." A question to open a tip is fine ("Made a mess?"); one per
  tip at most.
- **Calm.** No exclamation marks, no "Hey!", no gushing. The warmest she
  gets is "Nice." at a level up.
- **Says what to do, plainly, then stops.** Every line still has to teach
  the rule exactly; the voice is in the first few words and the verbs, not
  in extra sentences. Same length limits as plain text.
- **Talks to "you" about "your units", "one of yours".** Never names a
  character, never a gendered word.
- **Keys:** only through the player's keymap (`{Action}` placeholders,
  `keyboard-input` skill). Don't start a sentence with a key: a key name
  is lowercase ("s finds..."); write "Press {NextUnit} for..." instead.
- **Won't say:** her own name (it lives in the names table only), anything
  about the world, the plot, the cast or herself (no backstory, see
  *Role*), anything about a death. No jokes and no teasing the player: the
  tips show during battles, and the story keeps punchlines out of battles
  (`docs/design/setting-and-tone.md`, *Where the jokes go*). Dry is fine;
  a gag is not.

**How she sounds** (0043; `assets-src/audio/voice-refs/guide.flac`). Nick: "for Vix
I want a more sexy but upbeat voice". An adult woman's voice, smooth and velvety,
bright and playfully teasing. That is the sound; the wording above stays. Her
spoken lines never name a key (0475).

**Sample lines**

- "Over to you. Steer the cursor with {Cursor}; {Select} on one of your
  units picks it up."
- "Made a mess? Rewind undoes turns, for one of a few charges per map.
  Just looking is free."
- "One of yours is badly hurt. Pull it back, or have it drink a Potion
  from its item menu."
