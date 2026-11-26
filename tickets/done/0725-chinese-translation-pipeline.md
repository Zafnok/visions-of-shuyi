---
id: "0725"
title: "Chinese translation pipeline: glossary, names, speech styles, the back-translation check"
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: high
status: done
blocked_by: ["0045", "0235"]
nick_input: none
completed: 2026-11-25
---

# 0725 — Chinese translation pipeline: glossary, names, speech styles, the back-translation check

## Context

The Simplified Chinese option is machine-translated by the agent, chapter
by chapter (`docs/design/voices-languages-and-script.md`, *Chinese*).
It needs the same process as Japanese (0718;
[ADR-0045 §6](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)):
a glossary, a names table, a style sheet per speaker, and a separate
back-translation pass. This ticket sets it up; 0726 uses it on
Chapter 1.

The pack's code is `zhhans` (ADR-0045 allows 2 to 8 lowercase letters;
`zhhans` leaves `zhhant` free for Traditional later).

## Nick input

None. 0045 answered what a player sees
(`docs/design/voices-languages-and-script.md`, *Chinese (Simplified)*):
"Shuyi" is **姝怡**; the title is **姝怡的魔幻之境**, short
**姝怡幻境**; every name is written in characters chosen for their
sound; Nick's wife reads the short lists (step 7) and plays Chapter 1
later (0726). Nothing in this ticket waits on a person: 0726 waits for
her review of the lists.

## Scope

**In:** the documents a translating session follows; the Chinese names
table and game-term glossary; the pack's `lang.ron`.

**Out (do not do):** translating scripts or screen text (0726); code
(0233–0237 and 0239 built it). Don't copy another game's translated
text; other games are references for *conventions* only (ADR-0013).
Translate from the English, never from the Japanese pack.

Either order with 0718. If 0718 is done, give `docs/story/zh/` the same
file layout and move the steps both languages share into one
`docs/story/translating.md` that both READMEs point to. If it isn't,
write the README whole and add a line to 0718 saying to do that.

## Implementation steps

1. `docs/story/zh/README.md`: the pipeline per chapter: read the
   glossary and the speakers' style sheets → translate a scene at a time
   into `assets/lang/zhhans/dialogue/<file>.ron` → a **separate** session
   translates the Chinese back to English without seeing the original,
   then compares the two and lists lines whose meaning, tone or register
   drifted → revise → `cargo xtask lang-status zhhans` (no missing,
   stale or overflow).
2. `docs/story/zh/glossary.md`: every game term on screen (stats, weapon
   kinds, classes, menu verbs, phases, difficulty modes, magic terms
   from `docs/design/`) with its Chinese, matching what Chinese SRPG
   players already know (the official Simplified Chinese releases of
   Fire Emblem: Three Houses and Engage are the reference for
   conventions: 力量, 防御, 命中, 必杀…). One term, one translation,
   everywhere. Mainland wording, not Taiwan's, where they differ. Note
   which stat labels stay Latin (`HP`, `Lv`).
3. `docs/story/zh/names.md`: every id in `assets/data/names.ron` with
   its Chinese form per 0045's answer. If names are written in
   characters: choose by the English name's sound from the characters
   commonly used for foreign names (the Xinhua transliteration table's
   habits: 艾, 克, 莉, 斯…), note the pinyin and the plain meaning of
   each character, and avoid pairs that read as an ordinary word or an
   unlucky one. "Shuyi" is exactly 姝怡, and her surname is never
   used. The lead's default name gets a Chinese form like any other
   (0045's starting rule); a name the player typed is shown as typed.
   Entered in `assets/lang/zhhans/data.ron` as `names.<id>`. Names stay
   renameable: the rename rule in `docs/story/names.md` gains a line (a
   rename makes the Chinese entry stale).
4. `docs/story/zh/styles.md`: per character in
   `docs/story/characters/*.md`, from its *Voice notes*: how they say
   "I" and "you" (我 / 你 / 您, an older or rougher form where the
   character calls for it), how they address others (titles, 大人,
   plain names), how formal or clipped their sentences are, sentence-end
   particles they do and don't use (啊, 吧, 呢, 嘛), three sample lines
   translated. The setting is not modern: no internet slang. For the
   lead: Chinese third-person pronouns differ only in writing (他 / 她),
   so say when a line needs `text_m` / `text_f` (0235).
5. Token and punctuation rules, in the README: `{lead}` and `{n:…}` are
   kept; `{they}`-style tokens become 他 / 她 by the lead's gender;
   key-name placeholders are kept as they are. Full-width punctuation
   (`，。！？：；`), `“ ”` for speech, `……` and `——` doubled, no space
   between Chinese and Latin text.
   A `.dlg` line's stand-in for the lead's name (`{lead|the exile}`, 0754)
   isn't in the English text a pack translates: the line shows `{lead}`. A
   translated line that is voiced takes the stand-in in its own language
   the same way, which the pack's token check must then allow.
6. `assets/lang/zhhans/lang.ron`: name `简体中文`, `made_by: Machine`,
   and the fields 0239 added (`wide_font`, `wide_punctuation: true`) if
   0239 is done; if it isn't, add a line to 0239 to set them.
7. Nick's wife checks the short lists (0045): put the title (both
   forms), the names table and the glossary in one plain page
   (`docs/story/zh/for-review.md`: Chinese beside English, nothing
   else), ask Nick in the PR description to pass it to her, and record
   her changes as they come. The PR does not wait for them.
8. Add Chinese to the `story-writing` skill's "Translating" section
   (0718 adds the section; if it isn't there yet, add it).

## Acceptance criteria

- [x] The glossary covers every term in `assets/lang/en/ui.ron` and the data files that 0726 will translate (list any left for 0726 with a reason). (What is left is the glossary's last section: whole sentences, the text of tips, lessons and notes, later chapters' titles, the writer's preview.)
- [x] Every `names.ron` id has a Chinese entry; `cargo xtask lang-status zhhans` shows no missing `names.*` key. (142 of 142, in `zh/names.md`. `lang-status` was run on a trial pack that is not committed: see the notes.)
- [x] "Shuyi" is written with the characters recorded in the design doc, everywhere it appears.
- [x] Every Chapter 1 speaker has a style sheet. (The messenger, a name plate with one line, follows *A speaker with no sheet*.)
- [x] The pipeline README can be followed by a session that has read nothing else (try it on one scene of `test.dlg` and note the result).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The all-assets test validates `assets/lang/zhhans/`.

## Completion notes

Done 2026-11-25. No code and no file the game embeds: five documents in
the private repository, and in this one the skill's section, four
tickets' context and a new ticket.

The `story-writing` skill was at its 500-line limit (`check-skills`), so
its *Translating* section is two lines and a link to
`references/translating.md`, which holds the rules; five one-word last
lines elsewhere in the skill were joined to the line before them to make
the room. No word of the skill changed.

**What was made** (`assets-private/story/zh/`, private pull request
first, then the pin: the `content-change` skill):

- `README.md`: the whole process, for a session that has read nothing
  else: the rules, the pack's files and where they go, a chapter step by
  step, the blind back-translation by a separate session, the machine
  checks, tokens, punctuation and spacing, lengths in cells, what to do
  when the English changes.
- `glossary.md`: about 470 terms. Stats, weapons, armor, elements, sides
  and phases, the menus, what happens in a fight, objectives,
  progression, screens, options, the world map, shops, every class,
  item, spell, skill, Combat Art and terrain, and the story's forms of
  address. Its last section lists what is left for 0726 and why.
- `names.md`: all 142 ids of `names.ron`, each with pinyin and a note,
  the rules they were made by, and the plain meaning of every character
  used for its sound.
- `styles.md`: a sheet for each of the 21 characters with a sheet,
  narration and captions, the lead's lines and reply tones, who calls
  whom what in Chapter 1, when a line needs `text_m` and `text_f`, and a
  speaker with no sheet. Three sample lines each, translated from the
  voice cards' sample lines (illustrations, never script lines).
- `for-review.md`: the title, the names (each with what it is, and a
  second candidate where there is a real choice) and the glossary, Chinese
  beside English, with a table for her changes.
- `story/names.md` gained the rename line; `story/README.md` a row.

**Deviations**

1. **The documents are in `assets-private/story/zh/`, not
   `docs/story/zh/`.** The story's documents moved to the private
   repository after this ticket was written (0159), and the lists name
   every character of the story.
2. **No pack was added** (steps 3 and 6: `lang.ron`, and the names in
   `data.ron`). 0825 merged the day before: with a second language in
   the build, the first launch asks every player for a language and
   Options offers it. A `zhhans` pack holding only names would put
   简体中文 (机器翻译，母语者审校) in front of players with nothing
   translated and nothing reviewed, and 0045 says the names aren't used
   before Nick's wife has seen them. So the pack lands with 0726, whose
   scope already had "all of `names.ron`"; 0726 now says so, and the
   README holds `lang.ron`'s contents (0239's fields included). 0825's
   notes had already assumed this ("until 0719 or 0726 adds a pack").
3. **The names key check ran on a trial pack, not committed.** `lang.ron`
   and an `ui.ron` with the machine note in `assets/lang/zhhans/`, and a
   `data.ron` generated from `names.md` in
   `assets-private/content/lang/zhhans/`: `cargo shuyi lang-status
   zhhans` loaded it (`data: 320 missing, 0 stale`, no `names.*` key
   among them), which also showed that a pack split over the public and
   the private folder loads as one. Every Chinese character in the five
   documents is in the Fusion Pixel font (checked against the BDF's
   `ENCODING` lines).
4. **"The all-assets test validates `assets/lang/zhhans/`"**: there is
   no committed pack to validate. With the trial pack and the trial
   scene in, `all_assets_load::…` passed.
5. 0718 isn't done, so the README is whole and 0718 has the lines to
   share it.

**The README tried on `test.dlg`** (acceptance). A fresh agent given
only the README translated the scene `test` (17 lines and the caption)
into the trial pack; `lang-status --file test` then showed `0 missing,
0 stale, 0 orphaned`, and the content tests that load everything passed.
A second fresh agent, given only the keys, speakers and Chinese,
translated it back. Compared with the English: 16 of 18 came back with
their meaning and tone; two were flagged and would be revised ("Say
it." as 说出来 read unclear, because the no-sheet rule forbade the
natural 说啊; "stamps off" as 跺着脚走进 read awkward). The trial's
files were deleted. What the first agent had to guess became README and
style-sheet changes: the file's outer shape, that `cargo xtask lines`
gives the `source` and not captions, `lang-status`'s `--only` and
`--file`, a name the English writes with no token, a common noun that is
also a name plate, 你 or 您 for a speaker with no sheet, sounds like
"well", the 吧 of 走吧, and a better example of a line that needs two
texts.

**Four content tests fail with a pack in the build** (the trial; none
fails on this branch). Two expect `test` to be the only pack
(`lang::tests::the_embedded_languages_load`,
`test_files::the_test_files_add_to_the_story_and_change_none_of_it`):
0726 changes them. One refused the trial's `dialogue/test.ron` in the
private content (a test file's name): only a trial hits it. One is real:
**the Chapter 1 demo's bundle doesn't load** when a pack names a name
the demo cut. Follow-up ticket **0934**; 0726 and 0719 are blocked by
it.

**Other tickets changed:** 0726 (where the documents are, that it
creates the pack, the machine note, the tests, 0934), 0718 (the layout
to copy, the shared page, its `lang.ron` waits for 0719), 0719 (0934),
0756 (the pack's code; the voice's stand-in for the lead's name in a
pack needs the token check changed first).

### The agent's starting rules

What a Chinese player will read, decided here where the design doc was
silent. Nick, or his wife on the review page, can veto any of them.

- **Chinese isn't offered until Chapter 1 is translated** (deviation 2).
- **People are written by sound; a place or a thing named with plain
  English words is translated by its meaning**: the Thornmarch is
  荆棘边地 ("thorn borderland"), Greywater Abbey 灰水修道院. Ardeval and
  Harrowby are by sound. 0045 decided "by sound" for characters' names;
  places are this ticket's.
- **Four names were changed because their plain transliteration reads
  as something else in Chinese**: a Qing-court drama's word for
  "father", Ultraman, Batman's Wayne and the Red Army. Each row's note
  in `zh/names.md` says which and why.
- **Nine rows have a second candidate for her to pick from** (the "or"
  column of the review page): where the usual spelling of a name reads
  a little wrong for the person, and where a term can be said plainly
  or with more weight.
- **`EXP`, `CL` and `AS` stay in Latin letters as labels**, beside the
  decided `HP` and `Lv`.
- **The stat column's labels are one character** (力 魔 技 速 防 抗 移),
  because the column is three cells and a Chinese character takes two.
  The full words (力量, 魔力…) are used everywhere else. 0726 decides
  whether the column should get four cells instead.
- **A turn is 回合 and a phase is 阶段** (Player phase: 我军阶段), so the
  two never share a word.
- **What people call the lead** (five forms of address, in the
  glossary's last table): none says whether the lead is a man or a
  woman, as the English doesn't.
- **A royal "we" is 朕**, in public only.

### For Nick

- **Please pass `assets-private/story/zh/for-review.md` to your wife.**
  0726 doesn't start until she has read it; her changes are recorded on
  the page.
- The style sheets are made from today's voice cards. If the writer
  (0766) rewrites a card, its sheet is redone before that character is
  translated.

Nothing for the Act 1 playtest: nothing a player sees changed.

**Friction:** none: no code. **PR size:** no estimate was written
before starting (a slip); `pr-size` measures 632 in the game's
repository (the ticket's own file counted where it was and where it
is), and the private documents, which it doesn't count, are about 2,430
(780 of them the generated review page).
