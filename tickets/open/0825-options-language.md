---
id: "0825"
title: "Options: Language, and however the first launch picks one"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0233", "0805"]
nick_input: none
completed:
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

1. `Settings` (0805) gains `language: LangCode`, default `en`; bump the
   settings version and migrate old settings to `en`. Loaded into
   `Ctx::lang` at start-up.
2. Options screen: a "Language" row listing English and every pack in
   `Content::lang` except `test` (which shows only with
   `Ctx::debug_tools`). Each language is shown by its own name from
   `lang.ron` (`日本語`), whatever the current language is. Changing it
   takes effect at once on every screen.
3. The label beside a pack's name: `made_by: Machine` adds the pack's
   own note, in its own language: `(机器翻译，母语者审校)` for the Chinese
   pack, which a native reader checks, `(機械翻訳)` for Japanese. Keep
   the note's text in the pack (its `lang.ron` or a key of its
   `ui.ron`), so "checked" is the pack's fact and not code's; if the
   engine's `lang.ron` can't carry it, follow the `engine-change` skill.
4. First launch: a language list, once, before the layout picker, every
   language by its own name with its label. `app` reads the OS / browser
   language once and passes it to `Ctx` (only `app` may ask the
   platform, ADR-0004), and the engine's `best_match` (engine #278,
   2253) picks the pack that fits the system's languages, by their BCP 47
   tags (engine #276, 2226): that row starts highlighted. A system
   language with no pack highlights English (`LangCode::default()`).
   Write no matching of the game's own. Once picked, the list is not
   shown again; the Options row changes it.
5. A pack that names a language whose glyphs the atlas lacks can't
   happen (0236's atlas test), but until 0236 lands the row hides any
   pack whose own name can't be drawn.

## Acceptance criteria

- [ ] Choosing a language in Options changes the title screen's text at once and survives a restart (Harness test with the `test` pack).
- [ ] Old saved settings load with English (test).
- [ ] The first-launch behaviour matches the design doc (Harness test).
- [ ] The machine-translation label appears as decided (snapshot).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Snapshot / integration: the Options row; switching; first launch.

## Completion notes

