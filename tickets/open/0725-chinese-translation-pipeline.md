---
id: "0725"
title: "Chinese translation pipeline: glossary, names, speech styles, the back-translation check"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0045", "0235"]
nick_input: none
completed:
---

# 0725 — Chinese translation pipeline: glossary, names, speech styles, the back-translation check

## Context

The Simplified Chinese option is machine-translated by Claude, chapter
by chapter (`docs/design/voices-languages-and-script.md`, *Chinese*).
It needs the same process as Japanese (0718;
[ADR-0045 §6](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)):
a glossary, a names table, a style sheet per speaker, and a separate
back-translation pass. This ticket sets it up; 0726 uses it on
Chapter 1.

The pack's code is `zhhans` (ADR-0045 allows 2 to 8 lowercase letters;
`zhhans` leaves `zhhant` free for Traditional later).

## Nick input

None. 0045 answered what a player sees (the characters for "Shuyi", the
title, names in characters or Latin, who checks the Chinese). If 0045
recorded that Nick's wife reads the short lists, step 7 applies; nothing
else in this ticket waits on a person.

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
   unlucky one. "Shuyi" uses exactly the characters 0045 recorded.
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
6. `assets/lang/zhhans/lang.ron`: name `简体中文`, `made_by: Machine`,
   and the fields 0239 added (`wide_font`, `wide_punctuation: true`) if
   0239 is done; if it isn't, add a line to 0239 to set them.
7. Only if 0045 says Nick's wife checks the short lists: put the title,
   the names table and the glossary in one plain page
   (`docs/story/zh/for-review.md`: Chinese beside English, nothing
   else), ask Nick in the PR description to pass it to her, and record
   her changes as they come. The PR does not wait for them.
8. Add Chinese to the `story-writing` skill's "Translating" section
   (0718 adds the section; if it isn't there yet, add it).

## Acceptance criteria

- [ ] The glossary covers every term in `assets/lang/en/ui.ron` and the data files that 0726 will translate (list any left for 0726 with a reason).
- [ ] Every `names.ron` id has a Chinese entry; `cargo xtask lang-status zhhans` shows no missing `names.*` key.
- [ ] "Shuyi" is written with the characters recorded in the design doc, everywhere it appears.
- [ ] Every Chapter 1 speaker has a style sheet.
- [ ] The pipeline README can be followed by a session that has read nothing else (try it on one scene of `test.dlg` and note the result).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The all-assets test validates `assets/lang/zhhans/`.

## Completion notes

