---
id: "1105"
title: "shuyi-assets: languages, text tokens, the dialogue format, voice and tip files"
type: infra
milestone: "Engine: carve"
model: reasoning
effort: high
status: done
blocked_by: ["1104"]
nick_input: none
completed: 2026-10-16
---

# 1105 — shuyi-assets: languages, text tokens, dialogue, voice, tips

## Context

ADR-0063 §6. Text by key and language packs (ADR-0045), the `.dlg`
dialogue format with choices and `@if` (ADR-0011, ADR-0055), line ids,
voice clips by line id (ADR-0046) and the tip schema are things any
story game wants. They are mixed with this game's lead (a
player-named, gendered main character) and its tables
(`docs/engine/boundary.md`, *content*: `lang.rs`, `lang/lines.rs`,
`lang/data.rs`, `names.rs`, `dialogue*`, `voice.rs`, `tip.rs`, and
`core/src/lead.rs`'s `split_tokens`).

## Nick input

None.

## Scope

**In:**
- Move to `shuyi-assets`:
  - text by key and packs, stale and missing detection (`lang.rs`), with
    `DataText` (the game's names of classes, items…) taken out of `Lang`
    and kept by the game beside it;
  - per-line translation (`lang/lines.rs`) without `ByGender`, which
    becomes a generic `LineText::Variants(map from a variant name to
    text)`; the game names its variants `m` and `f`;
  - `names.rs` and the `{token}` splitter (`split_tokens`, `Part`) from
    `core/src/lead.rs`;
  - the dialogue format: `Step`, `Scene`, `Side`, `ChoiceOption`,
    `MusicLine`, `DialogueTable`, the parser, line ids, the on-screen
    checks and `@if` resolution (`Present`, `Scene::resolved`), with the
    text box size and the longest line as parameters
    (`DialogueLimits`), and the speaker id as a newtype the engine owns;
  - the voice clip manifest by line id, with variants generic as above;
  - the tip schema and `{Action}` placeholders, with the trigger type a
    generic parameter (the game keeps `TipTrigger`).
- The game keeps: the lead's rules (`LEAD_ID`, gendered portraits, lead
  tokens), the cast and roster checks, `TipTrigger`, `DataText`, and the
  `load()` functions that name its folders.
- Re-export under old paths.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1049's one-language Playdate builds use whatever language layers this makes.

**Out (do not do):** the dialogue *player* and box in `ui` (1112);
cinematics (1113).

## Implementation steps

1. Move the token splitter first (both `core` and `content` use it).
2. Move `lang` and `names`, making the variants generic.
3. Move the dialogue format and checks with `DialogueLimits`.
4. Move voice and tips.
5. Run every test; no `.dlg` file, language pack or snapshot changes.

## Acceptance criteria

- [x] An engine test parses and checks a fixture `.dlg` with a
      non-gendered speaker and custom limits, without any game type.
- [x] Every existing dialogue, language, voice and tip test passes
      unchanged in behavior.
- [x] `cargo xtask lang-status` and `check-script` output is unchanged
      on the real files.
- [x] `docs/engine/boundary.md` rows updated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: moved tests plus the fixture test above.
- Integration: the game's dialogue and language tests.

## Completion notes

Done on Opus 5.5, as the ticket names.

- `shuyi-assets` gained `lang` (with `lang/lines` and `lang/source`),
  `names`, `dialogue` (parser, checks, line ids, `Present`), `voice` and
  `tip`, each with its own fixture tests (dialogue, packs, names, voice,
  tips: about 140 new engine tests, including the fixture `.dlg` of a game
  with plain speakers and its own `DialogueLimits`). `shuyi-assets` now
  depends on `shuyi-core`.
- What the game gives the engine:
  - **Dialogue:** `SceneRules` (its `DialogueLimits`, its `Tokens`, the
    names table, who can speak, expressions, a line's limit by speaker,
    music cues, and checks of its own run after each scene) and a
    `Directive` type for directives the engine doesn't know. The game's
    are `Stage::Setting` (`@place`) and `Stage::Opening` (`@opening`);
    `Step::Setting(p)` is now `Step::Directive(Stage::Setting(p))`. A
    directive that stays in force (`Directive::kind`, `place`) is checked
    across `@choice` and `@if` parts as `@place` was.
  - **Languages:** `PackRules` (the rules of its data text, its tokens,
    the limits, the line variants). `LineText::ByGender` is
    `LineText::Variants`, by variant name; the game's are `m` and `f`
    (`lang::RULES`, `lang::variant(gender)`), read from the same
    `text_m`/`text_f` fields.
  - **Voice:** a `ClipVariant` type (the game's `Variant`: None, M, F, with
    its messages) and the words a line says as a variant plays it.
  - **Tips:** the trigger type (`TipTrigger` stays the game's) and
    `TipRules` (box, placeholders, faces).
- Content keeps the lead's rules (`LeadTokens`, gendered portraits, the
  lead's line limit), the cast and roster checks (`Cast`,
  `check_presence`), `check_places` and where an `@opening` may go,
  `TipTrigger`, the data text's keys and lookups, every `load()`, and
  wrappers with the old arguments. Old paths are re-exported or aliased
  (`dialogue::Step = Step<Stage>`, `voice::VoiceManifest =
  VoiceManifest<Variant>`, `tip::Tip = Tip<TipTrigger>`…).
- **Deviations:**
  - The `{token}` splitter (`Part`, `split_tokens`) went to
    `shuyi_core::text`, not `shuyi-assets`: `trpg-core` uses it and may
    depend only on the engine crate of its layer and below (ADR-0063 §2).
    For the same reason the engine-owned speaker id is
    `shuyi_core::CharacterId`, which `trpg_core::CharacterId` now
    re-exports (same type, same serialization), so dialogue steps and the
    game's units share it.
  - `DataText` is the engine's `SourceText` under the game's name (the
    English the packs translate); its builder from the game's tables and
    the lookups by table stayed. Because the engine's types can't get the
    game's inherent methods: `DataText::from_tables` → `lang::from_tables`,
    `VoiceManifest::playable`/`is_stale` (with names) →
    `voice::playable`/`voice::is_stale`, `Tip::face()` → `tip::face`, and
    `Playable::variant_for` takes a `Variant` (`Variant::of(gender)`). The
    game's `Lang` is a wrapper that derefs to the engine's.
  - The checks of where an `@opening` goes moved out of the scene checker
    into the game's `SceneRules::scene_problems`; same errors, sorted the
    same way.
  - `cargo xtask check-engine-boundary` gained `brace` in its list of
    ordinary words (also a skill id): the moved message "a name can't hold
    a brace" uses it in its plain sense.
  - The engine's messages for a choice count read "it needs 3" when the
    fewest and most are equal (the game's "2 or 3" is unchanged). An
    unknown field in a pack's dialogue entry is now worded by the engine
    (`unknown field \`text_x\`, expected one of …`), since the fields
    depend on the game's variants; no test or file had one.
- No `.dlg` file, language pack or snapshot changed. Playdate: everything
  moved keeps its `std` feature and builds without it
  (`shuyi-assets --no-default-features`).
- No gameplay rules decided.
