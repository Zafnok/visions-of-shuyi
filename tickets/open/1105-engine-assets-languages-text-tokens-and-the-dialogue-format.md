---
id: "1105"
title: "shuyi-assets: languages, text tokens, the dialogue format, voice and tip files"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1104"]
nick_input: none
completed:
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

- [ ] An engine test parses and checks a fixture `.dlg` with a
      non-gendered speaker and custom limits, without any game type.
- [ ] Every existing dialogue, language, voice and tip test passes
      unchanged in behavior.
- [ ] `cargo xtask lang-status` and `check-script` output is unchanged
      on the real files.
- [ ] `docs/engine/boundary.md` rows updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: moved tests plus the fixture test above.
- Integration: the game's dialogue and language tests.

## Completion notes
