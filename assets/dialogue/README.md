# Dialogue scripts (`.dlg`)

Each `*.dlg` file here holds one or more **scenes**: cutscenes and
conversations with two portraits (left and right), speech and narration.
They are loaded by `trpg_content::dialogue` and validated by the all-assets
test (ADR-0005), so a broken script fails CI with `file:line: message`.
The dialogue screen (ticket 0704) plays them one text box at a time.

`cargo xtask check-script [file]` runs the same checks on the scripts
alone, and `cargo run -p trpg-app -- --scene <id>` opens the game on one
scene. A gentler introduction for a writer new to the repository:
`docs/story/writers-guide.md`.

## A full example

```
# Chapter 1, opening. Comments are whole lines starting with #.

@scene ch01_opening
@caption Village of Heth, dusk
@music talk_calm
> The rain had not stopped for three days.
@left  ana neutral
@right bors angry
bors: You're late.
ana[happy]: Better late than... well.
bors: Than never. Say it. I've heard it from you
  often enough to know how it ends.
@right clear
@music stop
@right mira surprised
mira: Was that Bors? He looked furious.
ana[sad]: He always does.
@end

@scene ch01_after_battle
@left ana neutral
ana: That's the last of them.
@end
```

## Lines

Every line is one of these. Directives start at the very first column.

| Line | Meaning |
| ---- | ------- |
| *(blank)* | Ignored. |
| `# text` | A comment, ignored. Comments take the whole line: `@left ana neutral # hi` is an error. |
| `@scene <id>` | Starts a scene. Every line below belongs to it until `@end`. |
| `@end` | Ends the scene. Every `@scene` needs one. |
| `@caption <text>` | A location/time caption, shown at the top from here until the next `@caption`. |
| `@left <character> <expression>` | Puts a character on the left side with that expression. Whoever stood there leaves. |
| `@right <character> <expression>` | The same, on the right side. |
| `@left clear`, `@right clear` | That side's portrait leaves. |
| `@music <cue>` | Switches the music to that music cue (the old track fades out first). See "Music". |
| `@music stop` | Fades the music out to silence. |
| `<character>: <text>` | The character speaks. They must be on screen (`@left`/`@right` first). |
| `<character>[<expression>]: <text>` | The character changes expression, then speaks. The new expression stays. |
| `> <text>` | Narration: a text box with no speaker. |
| `  <text>` (indented) | Continues the speech or narration line above; the two are joined with one space. |
| `@choice` … `@endchoice` | The lead's reply choice: see below. |
| `@if <character>` … `@else` … `@endif` | Lines that play only if the character is still there, and the lines that play instead: see "Who is still there". |

Each speech or narration line is **one text box** on screen (the screen
splits a long one into pages). Directives (`@caption`, `@left`, `@right`,
`@music`) take effect before the next text box.

## Rules

The validator reports every broken rule, with the file and line:

- **Ids** (scenes, characters, expressions) are lowercase letters, digits
  and `_`: `ch01_opening`, `bors`, `surprised`.
- **Scene ids** are unique across *all* `.dlg` files; battles and chapters
  refer to scenes by id. Several scenes may share a file.
- **Characters** must exist in `assets/data/characters.ron`, as a character
  or in its `speakers` list (someone who talks but isn't a unit).
- **Two portraits at most**: one left, one right. A character can't stand on
  both sides at once.
- **Speakers must be on screen.** Narration needs nobody.
- **Expressions**: a character with a portrait (`assets/portraits/`) may use
  exactly its portrait's expressions; one without may use `neutral`, `happy`,
  `angry`, `sad`, `surprised` (the five every portrait has).
- **Length**: a speech or narration line, continuation lines included, is
  at most **200 characters** (the text box is 3 lines of about 70).
- **Plain ASCII text.** Use `'` and `"`, not curly quotes; `...` not `…`;
  `--` not `—`; `-` not `–`. The error message names the ASCII form.
- Every scene has at least one speech or narration line.
- **Nobody who may be gone is shown** outside an `@if` block for them: see
  "Who is still there".
- **Music**: `@music` names exactly one music cue from
  `assets/audio/audio.ron` (not a sound, not a music pool), or `stop`.

## Music

Conversations use mood tracks (`docs/design/audio.md`, rule 7): a scene
switches to a mood cue such as `talk_calm`, `talk_antagonist`, `scene_sad`,
`scene_tragic` or `dungeon_tense`. The cue ids are the `music` entries of
`assets/audio/audio.ron`; what each one is for is in `audio.md`.

```
@scene ch01_bad_news
@music talk_calm
@left  ana neutral
@right bors neutral
ana: Quiet night.
@music scene_sad
bors[sad]: Not for long. The bridge is gone.
@end
```

- `@music` may stand anywhere in a scene, a reply's reaction included. The
  music changes when the scene **reaches** that line: above, `scene_sad`
  starts as Bors's line appears, not before.
- The music **keeps playing after the scene ends**, until something else
  asks for different music: a later scene, a battle or the title screen.
  End a scene with `@music stop` if silence should follow it.
- A scene with no `@music` leaves the music as it was.
- Asking for the track already playing does nothing (it doesn't restart).
- **Skipping** a scene applies the last `@music` it passes, so the music
  after a skipped scene is the same as after a watched one. (Skipping
  stops at each reply choice, like reading does.)

## The lead: reply choices and tokens

The lead is shaped by the player (`docs/design/setting-and-tone.md`,
"Rules for writing the lead"): they speak mostly through **reply choices**,
and the player picks their gender and first name at New Game. The
character id `lead` is the lead; their name plate shows the player's name
and their portrait is `lead_m` or `lead_f` by gender.

### A full example

```
@scene ch01_gate
@left  lead neutral
@right bors angry
bors: {lead}. You took your time.
@choice
* earnest: We do this properly, or not at all.
  bors[surprised]: ...Huh. Fine.
* wry: I've had worse mornings. Not many.
  bors[happy]: Ha! There's the spirit.
  > Even the guards smile.
* blunt: Stop talking. Move.
  bors[angry]: Charming as ever.
@endchoice
@right bors neutral
> {lead} checks {their} sword. {They} knows the way from here.
lead: Let's move.
@end
```

### Reply choices

| Line | Meaning |
| ---- | ------- |
| `@choice` | Starts a reply choice. |
| `* <tone>: <text>` | One reply, at the very first column. The text is what the lead says; it is shown in the menu (the reply isn't shown again as a text box). The tone (`earnest`, `wry`, `blunt`…) is an id for writers; the player doesn't see it. |
| `  <line>` (two spaces) | A line of that reply's **reaction**: any speech, narration or directive (`@left`, `@caption`, `@music`…), indented by exactly two spaces. |
| `    <text>` (more spaces) | Continues the reaction's speech or narration line above. |
| `@endchoice` | Ends the choice. Every reply **rejoins** the scene here. |

While the choice is open, the line before it stays in the text box with the
replies listed under it (the box grows upward if they don't fit). When the reaction ends, the portraits stay as
the reaction left them: **the script sets the transition**. If the replies
leave a character with different expressions, set the one the scene goes on
with right after `@endchoice` (`@right bors neutral`) or on the next line
(`bors[neutral]: ...`). Skipping a scene stops at each choice.

### Lead tokens

In speech, narration, captions and reply text:

| Token | Male lead | Female lead |
| ----- | --------- | ----------- |
| `{lead}` | the player's name (default Ellery) | the player's name |
| `{they}` | he | she |
| `{them}` | him | her |
| `{their}` | his | her |
| `{theirs}` | his | hers |
| `{themself}` | himself | herself |

`{They}`, `{Them}`, `{Their}`, `{Theirs}` and `{Themself}` give the
capitalised word, for the start of a sentence. The tokens become he/she, so
write the verb to agree with he/she: `{They} knows`, not `{They} know`.

### Rules for choices and the lead

- A choice has **2 or 3** replies. Choices can't be nested.
- Reply text is at most **60 characters** (it must fit the menu).
- A reaction has at most **4** speech or narration lines, so the scene
  rejoins quickly.
- Every reaction must leave the **same characters on the same sides** (and
  the same caption) as the first reply's does.
- Outside choices the lead speaks only in short, neutral lines: a `lead:`
  line is at most **40 characters**.
- Only the tokens above and name tokens (`{n:<id>}`, see "Names") exist;
  any other `{...}`, or a `{` without a `}`,
  is an error.
- Lengths count each token at its longest: `{lead}` as 12 characters (the
  longest name), `{themself}` as 7 (`himself`), and so on.

## Names

Every proper noun of the story (characters, places, factions, gods, terms)
has a stable id in `docs/story/names.md`, and its display name in the
game's names table, `assets/data/names.ron` (ticket 0709). Nick may rename
anything, so **scripts never write a name out**: they use a name token, and
a rename is one line in `names.ron`.

```
@scene ch01_road
@caption {N:place.thornmarch}, dusk
@left  lead neutral
@right retainer neutral
retainer: {n:king} wants you out of {n:place.thornmarch}.
@end
```

With the current names this shows the caption "The Thornmarch, dusk" and
the line "Emeric wants you out of the Thornmarch."

| Token | Becomes |
| ----- | ------- |
| `{n:<id>}` | The display name with that id, as written in `names.ron` (`{n:king}` → `Emeric`, `{n:place.thornmarch}` → `the Thornmarch`) |
| `{N:<id>}` | The same with a capital first letter, for the start of a sentence (`{N:place.thornmarch}` → `The Thornmarch`) |

Name tokens work in speech, narration, captions and reply text. Speaker
ids (`retainer:`) and `@left`/`@right` use character ids, not tokens; a
character's name plate comes from `names.ron` too (its character id is its
name id). Names keep their article (`the Thornmarch`), so write
`{n:place.thornmarch}`, not `the {n:place.thornmarch}`.

### Short forms

People mostly say a first name, so the short forms of a name are ids of
their own (`docs/story/names.md`, "Short forms"), used like any other:

| Id | For | Example |
| -- | --- | ------- |
| `<character>.first` | The first name of a character with a two-word name | `{n:retainer.first}` → `Hollis` |
| `family.<name>` | A family name several people share | `{n:family.marr}` → `Marr` |
| `<god>.name`, `<god>.title` | A god's name and title on their own | `{n:god.mother.name}` → `Ama`, `{n:god.mother.title}` → `the Mother` |
| `<character>.last` | A surname only that character has | `{n:sergeant.last}` → `Rook` |
| `red_captain.nickname` | A nickname | `Red Harl` |
| `vowmaster.title` | An office, for people who don't say the name | `the Master of Vows` |
| `faction.brennmark.adj` | A people's name without its article: the adjective, or a nickname | `{n:faction.brennmark.adj}` → `Brennish` |

### Rules for names

- A name token's id must be in `names.ron`. Ids are lowercase letters,
  digits, `_` and `.`.
- `{n:lead}` is an error: the player names the lead, so write `{lead}`.
- **No names written out.** A line that contains a display name from
  `names.ron` is an error that names the token to use. The check matches
  whole words, case-sensitively, without the name's leading `the`/`a`/`an`
  (so `Thornmarch` alone is caught too). Names with no capital letter
  (`breath`, `a vow`, `mor`) are ordinary words and aren't checked.
  Comments aren't checked. Short forms are names too, so `Hollis` alone is
  an error; a line that holds several names reports the longest
  (`Hollis Marr` is reported as `{n:retainer}`). A few short forms are also
  ordinary words (`Mother`, `Hand`, `Pyre`, `Wren`, `Crane`, `Rook`, `Holt`,
  `Mast`): with a capital they are always taken as the name, so reword a
  line that starts with one.
- **Lengths** count every name token as the **longest** name in
  `names.ron`, whichever name it is, so renaming anything can't push a line
  over its limit.

## Who is still there: `@if` blocks

In Classic a companion who falls is dead and leaves the army
(`docs/design/death-and-difficulty.md`). A scene played after that must
not have them speak. Lines in an `@if` block play only if the named
character is **there** when the scene plays (ADR-0055):

```
@scene ch01_after
@left lead neutral
@if sergeant
@right sergeant happy
sergeant: Pay-day at last.
@right clear
@else
> Nobody speaks for a while.
@endif
> The fort is quiet.
@end
```

| Line | Meaning |
| ---- | ------- |
| `@if <character>` | Starts a block. The lines below play only if the character is there. |
| `@else` | Optional. The lines below it play only if the character is **not** there. |
| `@endif` | Ends the block. The scene goes on for everyone. |

A block's lines are written as far in as its `@if`: not indented in a
scene, two spaces in a reply's reaction. (An indented line still continues
the line above it.)

**Who is there:**

- In a chapter's scenes (before and after its battle): everyone **in the
  army**. A companion who died in Classic is not; one who retreated in
  Casual is; someone recruited in the battle is, from its victory scenes
  on.
- In a battle's scenes (its triggers): everyone whose **unit is on the
  map** at that moment, friend or enemy. A companion left out of the
  battle is not there. A unit that is about to fall is still there for the
  scene before that fight and for its own last words.
- In the debug scene viewer: everyone.

Blocks nest, so a line between two companions sits in a block for each. A
block may stand in a reply's reaction, and a `@choice` may stand in a
block:

```
@scene ch01_road
@left lead neutral
@if keeper
@right keeper neutral
@if heretic
keeper: {n:heretic}, stay close to me.
@else
keeper: She would have hated this road.
@endif
@right clear
@endif
> The road goes on.
@choice
* earnest: We keep going.
  @if sergeant
  @right sergeant happy
  sergeant: That's the spirit.
  @right clear
  @else
  > Nobody answers.
  @endif
* blunt: Move.
  > They move.
@endchoice
lead: Onward.
@end
```

### Rules for `@if` blocks

- **Anyone the army can lose appears only inside an `@if` block for
  them**: `@left`/`@right` with them, and their lines. The army can lose
  everyone in the starting roster (`assets/data/new_game.ron`) but the
  lead, and everyone a battle recruits. The error says who and where.
- **Scenes that can't play without someone need no block for them.** The
  validator knows from where the scene is played:
  - a battle trigger's scene: the characters the trigger names (a death
    quote is said by the one dying; a boss's lines by the boss; a talk by
    its two);
  - a support conversation: its two;
  - before anyone can have fallen: the first chapter's scenes before its
    battle, and that battle's scenes at the start of turn 1, can count on
    everyone the game starts with.
- **Nobody appears in the `@else` of their own block.**
- **An `@if` that can only go one way is an error**: for the lead, for
  someone the scene can't play without, or inside a block (or its
  `@else`) for the same character.
- **Both ways leave the same screen**: the same characters on the same
  sides, and the same caption, as replies must. Someone who enters in a
  block leaves in it (`@right clear` before `@endif`), or the `@else` puts
  the same character there.
- A block opened in a reaction closes in that reaction. A `@choice` can't
  stand in a block that is itself in a reaction (that is a choice in a
  choice).
- A reaction's limit of 4 lines counts a block as its longer part.
- `@if` takes one character id; any character or speaker of
  `characters.ron`.
- **A scene with nothing left to say isn't played.** If every line of a
  scene is in blocks for people who are gone, the game skips the scene
  (its `@music` too). Give it an `@else` if something should still be
  said.
- **The validator doesn't read the words.** A line that mentions someone
  who may be dead, or narration that has them do something, is yours to
  put in their block.
- Both parts of every block get line ids, translations and voice clips.
  `cargo xtask lines` lists them all.

## Line ids

Every speech line, narration line and reply has a **line id**
(ADR-0045 §3). Translations and voice clips attach to it. Nobody writes
it: the loader computes it, so scripts stay free of ids.

```
cargo xtask lines ch01_intro
```

prints one row per line: the id, who says it (`>` for narration, `*` for
a reply) and the text as written.

```
ch01_intro_e4a70fe9	lead	Again.
```

An id is `<scene id>_<8 hex digits>`: a hash of the speaker and the text
as written (tokens not filled in, continuation lines joined with one
space). So:

- **Rewording a line changes its id**, even by one letter or comma. That
  is what marks its translation and its voice clip as out of date: they
  were made for the old words. Changing who says it, or moving it to
  another scene, does the same.
- **Moving a line within its scene, or adding and removing lines around
  it, changes nothing.** Nor does changing the speaker's expression, a
  reply's tone, or where a long line is broken over continuation lines.
- **The same line twice in a scene** (the same speaker saying the same
  words) gets `_2` on the second, `_3` on the third, in script order:
  `ch01_intro_e4a70fe9_2`. Removing the first makes the second the first.
- Renaming a scene changes the id of every line in it.
- Very rarely, two different lines of one scene hash to the same id; the
  validator reports both, and the fix is to reword one.
