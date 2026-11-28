---
id: "0825"
title: "Options: Language, and however the first launch picks one"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0233", "0805"]
nick_input: none
completed: 2026-11-25
---

# 0825 — Options: Language, and however the first launch picks one

## Context

0233 put a language in `Ctx`; nothing lets a player change it. Nick
decided in 0045 (2026-11-24, for every language;
`docs/design/voices-languages-and-script.md`, *Picking a language and
saying it is machine-made*) how a language is picked and how a machine
translation is labelled:

- First launch asks for the language once, before anything else is
  asked (the keyboard-layout picker after it), with the computer's
  language highlighted when the game has a pack for it, English
  otherwise. Options has a Language row too.
- A machine-translated language says so beside its name wherever it is
  picked: **简体中文 (机器翻译，母语者审校)** for Chinese, which a native
  reader checks, **日本語 (機械翻訳)** for Japanese. A human translation
  has no note. 0805 builds
the Options screen and `Settings`.

**Where it is kept** (0058, `docs/design/options.md`, *Where options are
kept*): the language is a field of `Settings`, so it belongs to the
current player profile (0836) like every option; nothing here needs the
profiles to exist first.

## Nick input

None: 0045 answered. Build exactly what the design doc says; if it
doesn't cover something a player would see, stop and ask through
`ask-nick`.

## Scope

**In:** `Settings::language`; an Options row; the first-launch
language list; the machine-translation labels.

**Out (do not do):** the translations (0719, 0726); a kana name-entry screen (its
own ticket if 0042 asks for one); store page text (0907).

The row lists whatever packs exist and labels them, so a Chinese or
Japanese pack needs nothing more here once it says how it was made.

## Implementation steps

The engine has the setting, the row and the system's languages since the
pin move of 2026-11-25 (engine #583, 2349): use them and write none of
the game's own.

1. The engine's `Settings::language: Option<LangCode>` holds the pick
   (`None` until the player picks; Restore defaults keeps it). The game's
   saved form (`crates/ui/src/settings.rs`) writes and reads it with the
   engine's `language_to_text` and `language_from_text`; bump the
   settings version so old settings read as `None` (English). `Ctx` sets
   `Ctx::lang` from it when the settings load and when it changes, so the
   game sets `Ctx::lang` nowhere itself.
2. Options screen: the engine's `OptionRow::language`, which lists
   `Ctx::languages_offered` (the source and every pack, each by its own
   name from `lang.ron`, `日本語`; `test` only with debug tools) and
   changes every screen at once, Discard putting it back.
3. The label beside a pack's name: `made_by: Machine` adds the pack's
   own note, in its own language: `(机器翻译，母语者审校)` for the Chinese
   pack, which a native reader checks, `(機械翻訳)` for Japanese. Keep
   the note's text in the pack (its `lang.ron` or a key of its
   `ui.ron`), so "checked" is the pack's fact and not code's. If
   `OptionRow::language` can't show it, build the row's names with
   `RowValue::names`; if the engine's `lang.ron` can't carry the note,
   or neither row can show it, follow the `engine-change` skill.
4. First launch: a language list, once, before the layout picker, every
   language by its own name with its label. The engine reads the
   system's languages (`PlatformFacts::languages`: the OS, the
   browser's `navigator.languages`, the Playdate's `getLanguage`) and,
   with `GameData::first_language` left at `FirstLanguage::System`, sets
   `Ctx::lang` to the pack that fits them by `best_match` while the
   player hasn't picked (English when none fits): that row starts
   highlighted. Write no matching or platform reading of the game's own.
   Once picked, the list is not shown again; the Options row changes it.
5. A pack that names a language whose glyphs the atlas lacks can't
   happen (0236's atlas test), but until 0236 lands the row hides any
   pack whose own name can't be drawn.

## Acceptance criteria

- [x] Choosing a language in Options changes the title screen's text at once and survives a restart (Harness test with the `test` pack).
- [x] Old saved settings load with English (test).
- [x] The first-launch behaviour matches the design doc (Harness test).
- [x] The machine-translation label appears as decided (snapshot).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Snapshot / integration: the Options row; switching; first launch.

## Completion notes

Engine friction: estimate 180 production lines (like the voices
question and a settings field), measured 131: under 2×.

Built on the engine's language API at the pin (`Settings::language`,
`language_to_text`/`language_from_text`, `Ctx::languages_offered`,
`RowValue::names`, the system-language pick); the game matches no
language and reads no platform itself.

- **Saved form**: `language` (its tag) in the settings record, version 4;
  a version 1–3 save reads as none picked, so English.
- **Options**: a Language row built with `RowValue::names` (not
  `OptionRow::language`, which can't add the note): each language by its
  own name, a `made_by: Machine` pack followed by its own
  `language.machine_note` from its `ui.ron`. The test pack says
  `(MACHINE TRANSLATION)`; the Chinese pack (0726) must say
  `(机器翻译，母语者审校)` and the Japanese (0719) `(機械翻訳)`, as the
  design doc decided. English's own text of the key is never shown.
  Preview, Keep, Discard and Restore defaults behave as every row.
- **First launch**: `screens/language_list.rs`. With no language and no
  layout picked, `game::start` puts the layout picker (on a keyboard)
  and the language list over the title, the list on top, starting on the
  engine's pick from the system's languages. The engine's first-key hook
  is unchanged.
- **Step 5** (hide a pack whose name can't be drawn) is not built: 0236
  landed, and its atlas test makes that case impossible.

### For the Act 1 playtest (1064)

- The list is skipped while English is the only language the game ships
  (the `test` pack doesn't count), so today's players see nothing new
  until 0719 or 0726 adds a pack.
- Its title is "Language", in the language already in use; its keys are
  the layout picker's.
- A player whose settings are from before the list (a layout saved) isn't
  asked: English stays; Options changes it.
- The Options row sits just above Layout, starting that group.
- On a computer with a keyboard, a controller player picks the language,
  then sees the layout picker, which any button closes (as before).

- Only visible once a Chinese or Japanese pack exists: the list's look
  with real wide names.
