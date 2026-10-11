# Writer's guide

For a writer who has never seen this repository. It covers what a script
file is, every cue you can write in one, how to check your work and watch
a scene, what in the story is fixed, and what a rewrite costs.

You write **scenes**: cutscenes and conversations. Everything else (maps,
battles, which scene plays when) is data Claude maintains; ask and it is
wired for you.

The full reference for the script format is
[`assets/dialogue/README.md`](../../assets/dialogue/README.md). This guide
teaches the format; that file settles every detail. What to read about the
story itself, and in what order, is in [`README.md`](README.md), "For a
human writer".

Every example in a `dlg` block below is a complete scene that passes the
checker (a test keeps it so). Copy one to start a scene.

## 1. What a scene file is

Scripts are plain text files ending in `.dlg`, in `assets/dialogue/`.
Chapter 1 is `assets/dialogue/ch01.dlg`. A file holds any number of
scenes; a scene starts with `@scene <id>` and ends with `@end`.

The screen a scene plays on has two portraits (one left, one right), a
caption at the top for the place and time, and a text box at the bottom.
The player presses a key to move from one text box to the next.

```dlg
# A comment: a whole line starting with #. The game ignores it.

@scene guide_first
@caption A rented house, before dawn
> Rain on the shutters. Nobody in the house has slept.
@left  lead neutral
@right messenger neutral
messenger: A letter. No seal on it, and no name. I was paid to
  bring it, not to read it.
lead: Who paid you?
messenger[surprised]: ...I didn't ask.
@end
```

What the player sees: the caption appears; a text box with the narration
(no portraits yet); the lead appears on the left and the messenger on the
right; the messenger's line (the two script lines are one text box: a line
that starts with spaces continues the line above); the lead's line; the
messenger's portrait changes to surprised and they speak.

The rules to know from the start:

- **Lines starting with `@` are cues.** They begin at the very first
  column. Anything else is speech (`<who>: <text>`), narration
  (`> <text>`), a continuation (indented) or a comment (`#`).
- **One speech or narration line is one text box**, at most **200
  characters**. A longer thought is two lines.
- **Ids are lowercase with underscores**: `guide_first`, `messenger`.
  A scene id is unique across all files. Chapter scenes are named
  `ch<NN>_<what>`: `ch01_intro`.
- **Plain typewriter punctuation.** `'` and `"`, not curly quotes; `...`
  not the single ellipsis character; `--` for a dash. Word processors
  curl quotes on their own: write in a plain text editor (VS Code,
  Notepad++, BBEdit, TextEdit in plain-text mode), or the checker will
  list every one with its fix.
- **A speaker must be on screen**: put them there with `@left` or
  `@right` before their first line.

## 2. The cues

### Portraits: `@left`, `@right`

`@left <character> <expression>` puts a character on the left;
`@right` the same on the right. Whoever stood on that side leaves.
`@left clear` or `@right clear` empties the side. There are never more
than two people on screen.

A character changes expression either with the cue again, or in passing
on a speech line: `<who>[<expression>]: <text>`. The new expression stays
until the next change.

```dlg
@scene guide_portraits
@left  lead neutral
@right red_captain happy
red_captain: You came. I had money on you running.
red_captain[angry]: I lost that money.
@right clear
> He goes back up the stairs without waiting for an answer.
@right messenger sad
messenger: I'd run, if I were you.
@end
```

What the player sees: the captain smiling on the right for his first
line; his face turns angry for the second; he leaves and the right side is
empty during the narration; the messenger takes his place.

**Who can appear.** The cast is listed in each chapter's beat sheet
(`docs/story/chapters/ch01.md`, "Cast on screen") with the id to write.
The lead is always `lead`. To bring in somebody new, even a nameless
villager, ask Claude for an id first: a character needs an entry in the
game's data before a script can name them.

**Expressions.** Every character has these five: `neutral`, `happy`,
`angry`, `sad`, `surprised`. A character whose portrait has more (say
`sly`) lists them in their portrait file, `assets/portraits/<id>.ron`;
the checker tells you the ones a character has if you use one they
don't. Each character's sheet (`docs/story/characters/<id>.md`) says how
they carry themselves, which is the guide to which expression fits.

### Captions: `@caption`

`@caption <text>` shows a place and time at the top of the screen, from
that line until the next `@caption`. Use one when the place or time
changes.

```dlg
@scene guide_caption
@caption The toll-fort, noon
> The gate stands open. Nobody has closed it in years.
@caption The toll-fort, dusk
> By evening somebody has closed it.
@end
```

What the player sees: "The toll-fort, noon" over the first text box,
replaced by "The toll-fort, dusk" for the second.

### Music: `@music`

`@music <cue>` changes the music when the scene reaches that line: the
old track fades out and the new one starts. `@music stop` fades to
silence. The music **keeps playing after the scene ends**, into whatever
follows, so end a scene with `@music stop` if silence should follow it. A
scene with no `@music` leaves the music as it was.

The mood cues for conversations:

| Cue | Use it for | It sounds like |
| --- | ---------- | -------------- |
| `talk_calm` | Calm, everyday talk. Also banter, until that gets its own track. | A light orchestral field theme |
| `talk_antagonist` | A conversation with the antagonist, or a scene that watches them | A slow, menacing classical piece |
| `scene_sad` | A sad moment | A soft, emotional orchestral piece |
| `scene_tragic` | A tragedy | A slow, hesitant orchestral piece |
| `dungeon_tense` | Something tense being uncovered: a conspiracy, a dark place | A dark forest theme |

Place cues exist too (`village`, `village_home`, `city_first_visit`,
`graveyard_desert`, `side_quest`), and `mythic_moment` is reserved for one
moment in the story. The whole list, with a link to listen to each track,
is the table in [`docs/design/audio.md`](../design/audio.md), "Music
cues". Battle music (`battle_*`) belongs to battles, not scenes.

```dlg
@scene guide_music
@music talk_calm
@left  lead neutral
@right messenger neutral
messenger: Quiet road. I like a quiet road.
@music scene_sad
messenger[sad]: It wasn't quiet at the ford. I counted nine graves.
@music stop
> Neither of them says anything for a while.
@end
```

What the player hears: the calm track under the first line; it fades and
the sad one starts as the second line appears, not before; silence for the
narration, and after the scene.

### Reply choices: `@choice`

The lead is the player's character, and speaks mostly through **reply
choices**: two or three replies in different tones. The others react in a
line or two, and then the scene goes on **the same way whichever was
picked**.

```dlg
@scene guide_choice
@left  lead neutral
@right red_captain neutral
red_captain: Last chance. Walk away and I tell them you were never here.
@choice
* earnest: These people did nothing to you.
  red_captain[sad]: No. They didn't.
* wry: You'd lie for me? I'm touched.
  red_captain[happy]: Ha! I'd lie for anyone, for the right coin.
  > A few of his men laugh.
* blunt: No.
  red_captain[angry]: Short and stupid. Fine.
@endchoice
@right red_captain neutral
red_captain: Then we do it the old way.
@end
```

What the player sees: the captain's line stays in the text box with the
three replies listed under it. They pick one; the reply itself is not
shown again. The reaction under that reply plays, then the scene rejoins
at `@endchoice`.

- A reply is `* <tone>: <text>` at the first column. The tone is a note
  for writers; the player never sees it. The text is at most **60
  characters**.
- A reaction is the lines indented by **two spaces** under the reply: at
  most **4** text boxes.
- Every reaction must leave the same people on the same sides. They may
  leave different expressions behind, so set the one the scene goes on
  with right after `@endchoice`, as above.
- 2 or 3 replies. No choice inside a choice.

### Who is still there: `@if`

In Classic mode a companion who falls in battle is dead for good. A scene
that plays afterwards must still work. Lines inside an `@if <character>`
block play only if that character is still there; an optional `@else`
gives what plays instead.

```dlg
@scene guide_if
@left lead neutral
> The fort is theirs. Somebody finds the pay-chest.
@if sergeant
@right sergeant happy
sergeant: Three years late. I'll take it.
@right clear
@else
> Nobody steps forward to open it.
@endif
lead: Count it. All of it.
@end
```

What the player sees, if the sergeant lived: she appears, speaks and
leaves. If she died: the narration line instead. Either way the scene ends
on the lead's line.

The checker knows who the army can lose and **requires** the block for
them: their portrait cues and their lines. It can't read your prose, so a
narration line that has a dead character do something is yours to put in
their block. Scenes that can't play without someone (their own death
scene, the opening before any battle) don't need it. The full rules are in
the reference, "Who is still there".

## 3. The lead

The player picks the lead's first name and gender, and decides who they
are. So (from [`docs/design/setting-and-tone.md`](../design/setting-and-tone.md),
"Rules for writing the lead"):

1. **Few lines.** Outside reply choices the lead says only short, neutral
   things: a `lead:` line is at most **40 characters** ("Let's move.").
   Other characters carry the scenes.
2. **Reply choices at key moments**: about one to three in a chapter.
   Choices never change the plot, who joins or the ending. They give the
   player a voice.
3. **Never fix the lead's personality.** No joke, cruelty or strong
   opinion the player didn't pick. Their *situation* (exiled, wronged) is
   fixed and is where the weight comes from.
4. **No line depends on the lead's gender.** Write the name and pronouns
   with tokens.
5. Because the lead is quiet, **the supporting cast must be vivid**.

### The lead's name and pronouns

| Write | Male lead | Female lead |
| ----- | --------- | ----------- |
| `{lead}` | the player's name (default Ellery) | the player's name |
| `{they}` | he | she |
| `{them}` | him | her |
| `{their}` | his | her |
| `{theirs}` | his | hers |
| `{themself}` | himself | herself |

A capital gives a capital: `{They}` becomes `He` or `She`. The tokens
become he or she, so the verb agrees with he and she: `{They} knows`, not
`{They} know`.

```dlg
@scene guide_lead
@left  lead neutral
@right messenger neutral
messenger: {lead}? That's the name on the letter. It's yours, then.
> {lead} turns it over in {their} hands. {They} knows the writing.
lead: Thank you.
@end
```

With the default male lead the narration reads: "Ellery turns it over in
his hands. He knows the writing."

## 4. Names are never typed out

Every proper noun in the story (a person, a place, a faction, a god) can
still be renamed, and will be if Nick finds a better name. So a script
never contains a name. It contains a **name token**, and the game fills in
today's name:

```dlg
@scene guide_names
@caption {N:place.harrowby}, morning
@left  lead neutral
@right messenger neutral
messenger: {n:king} wants you gone from {n:place.thornmarch}. Those are
  his words, not mine.
messenger: {N:red_captain.nickname} was less polite about it.
@end
```

Today this shows "Harrowby, morning", "Emeric wants you gone from the
Thornmarch." and "Red Harl was less polite about it."

- `{n:<id>}` is the name; `{N:<id>}` the same with a capital first
  letter, for the start of a sentence.
- The ids are in [`names.md`](names.md): one for every name, and separate
  ones for short forms, because people mostly say a first name
  (`{n:retainer.first}` is "Hollis", `{n:retainer}` is "Hollis Marr").
- A name keeps its article: `{n:place.thornmarch}` is "the Thornmarch",
  so don't write "the" in front of it.
- **If you type a name, the checker stops you** and names the token to
  use. A few names are also ordinary words (Mother, Hand, Pyre, Wren,
  Crane, Rook, Holt, Mast): with a capital letter they are always read as
  the name, so reword a sentence that would start with one.
- A new name (a new village, a new minor character) needs an id first:
  ask Claude to add it.

The lead is the exception: `{lead}`, never a name token.

## 5. Where scenes are played from

You don't edit these files, but it helps to know they exist:

- A **chapter** (`assets/chapters/<id>.ron`) lists the scenes played
  before its battle and after a victory, in order.
- A **battle** (`assets/battles/<id>.ron`) lists **triggers**: a scene
  played when something happens on the map. The moments a trigger can
  catch: a turn starts; a unit steps into an area; a unit is about to
  fight (anyone, or one particular opponent); a unit drops to half
  health; a unit falls; two units talk.
- A **support conversation** between two companions is a scene too.

The header comment of `assets/dialogue/ch01.dlg` shows Chapter 1's scenes
and what plays each. To add a scene, move one, or have one play at a new
moment, write the scene and tell Claude when it should play. A scene
nothing plays yet is fine: it is checked, and you can watch it (below).

## 6. What is fixed and what may change

- **Fixed: Nick's beats**, [`beats.md`](beats.md). They are the owner's
  own words and are canon. Only Nick changes them.
- **Approved by Nick, change only with him:** the cast and their arcs
  ([`characters/`](characters/)), the world ([`bible.md`](bible.md)) and
  the outline with its twists ([`outline.md`](outline.md)). If a better
  idea needs one of these to move, bring it to him as a proposal.
- **Fixed by the game:** the lead's rules above, and each chapter's
  shape (how many battles, who is in them: `docs/design/`).
- **Yours:** every line of dialogue and narration, the scene breakdown
  inside a chapter ([`chapters/`](chapters/)), expressions, captions,
  music moods, where reply choices fall and what they say. Names may be
  proposed; Nick decides them.
- **Keep up to date:** the continuity ledger ([`ledger.md`](ledger.md)),
  which records what each character knows and has done as of each
  chapter. After a rewrite that changes who knows or did what, update it,
  or the next chapter is written from a wrong picture.

## 7. What a rewrite costs

Every line (speech, narration, reply) has a hidden **line id**, worked out
from who says it and its exact words. Translations and voice clips are
attached to that id.

- **Changing a line's words, even a comma, gives it a new id.** Its
  translation in every language falls back to English, and its voice clip
  falls silent, until each is redone. Changing who says the line, or
  moving it to another scene, does the same.
- **Free:** moving a line within its scene, adding or removing lines
  around it, changing an expression, a caption, the music, a reply's
  tone, or where a long line is broken across script lines.
- Renaming a scene changes every line in it.

So polish freely before a chapter is translated and voiced, and after
that change lines for a reason.

To see the cost of an edit, list a scene's lines with their ids before
and after, and compare:

```
cargo xtask lines ch01_intro
```

Each row is an id, the speaker and the text. A row whose id is gone
after your edit is a line whose translation and voice are now out of
date. For a language, `cargo xtask lang-status <code>` lists the entries
that have gone out of date since they were translated. (Dialogue isn't
translated or voiced yet; when it is, that command and the voice tool
will list the lines to redo.)

## 8. Checking your work

```
cargo xtask check-script
```

checks every script and prints each problem as `file:line: what is
wrong`, with the fix where there is one, then a count:

```
assets/dialogue/ch01.dlg:48:14: non-ASCII punctuation '’'; use ' instead
assets/dialogue/ch01.dlg:52: "Emeric" is written out; write {n:king} ("Emeric") so a rename reaches this line
2 problems in 5 scripts
```

`cargo xtask check-script assets/dialogue/ch01.dlg` lists one file's
problems only. The file may also be a draft kept anywhere else. A script
is finished when this says `0 problems`.

If a limit gets in the way of good writing (the 200 characters of a text
box, the 40 of a lead line, typewriter punctuation), don't work around
it: tell Nick. The limits come from the screen and the font and can be
reconsidered; the checker can't be argued with.

## 9. Watching a scene

```
cargo run -p trpg-app -- --scene ch01_intro
```

opens the game straight on that scene, with the default lead (Ellery,
male). Add `--lead f` for the female lead:

```
cargo run -p trpg-app -- --scene ch01_intro --lead f
```

The scene plays with everyone alive (every `@if` block plays, no `@else`).
When it ends, the screen names the key that plays it again and the key
that quits. After editing the script, quit and run the command again: the
scripts are built into the game. The first run takes several minutes;
later ones are quick.

The portraits in this repository are placeholders. The real ones are
bought art kept in a private repository; Nick can give access.

## 10. Setting up

There are two ways to work.

**A. With the repository on your machine.** You can check and watch
scenes yourself.

Windows:

1. Install [Git](https://git-scm.com/download/win).
2. Install Rust from [rustup.rs](https://rustup.rs). When the installer
   offers to install the Visual Studio C++ build tools, accept.
3. In a terminal: `git clone https://github.com/Zafnok/visions-of-shuyi`
   then `cd visions-of-shuyi`.
4. `cargo xtask check-script`. The first run downloads and builds for a
   few minutes.

macOS:

1. In Terminal: `xcode-select --install` (Apple's build tools, with Git).
2. Install Rust from [rustup.rs](https://rustup.rs).
3. `git clone https://github.com/Zafnok/visions-of-shuyi` then
   `cd visions-of-shuyi`.
4. `cargo xtask check-script`.

The repository picks the right Rust version by itself. How your edits get
back (a branch and a pull request, or files sent to Nick) is agreed with
Nick.

**B. Without it.** Write `.dlg` files in a plain text editor and send
them to Nick. He runs the checker and the preview and sends back the
list of problems. This works, but each round costs a day; for more than a
few scenes, set up A.

A released copy of the game can't check scripts: the checker is part of
the repository's tools, not of the game.
