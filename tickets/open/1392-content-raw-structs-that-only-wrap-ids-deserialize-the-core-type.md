---
id: "1392"
title: "Content's Raw structs that only wrap strings into ids deserialize the core type directly"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1237"]
nick_input: none
completed:
---

# 1392 — Content's Raw structs that only wrap ids deserialize the core type

## Context

Step 6 of 1237 (shared test drivers and fixtures), split off by its size
rule: 1237 measured 1,747 changed lines before this step (`cargo xtask
pr-size`), past the 1,500 its *Scope* set for doing it in the same PR.

From the 2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), bucket B3): adding
one field to a definition edits the core type, its `Raw*` mirror in
`trpg-content` and the conversion between them. `trpg-content` has about
63 `Raw*` serde types (`grep -rn "struct Raw\|enum Raw" crates/content/src`:
`item.rs` 10, `battle.rs` 8, `worldmap.rs` 7, `spell.rs` 5, `lesson.rs`
4, `character.rs` 4, …). Some only turn a `String` into an id
(`ClassId`, `SkillId`, `SpellId`, `ItemId`, `ArtId`, `CharacterId`, …),
which the core id types can deserialize themselves: they derive
`Deserialize`.

## Nick input

None. Loading only; nothing a player sees changes.

## Scope

**In:**
- Each `Raw*` type of `crates/content/src` whose conversion only wraps
  strings into core ids (no lookup, no check, no default worked out from
  other fields): deserialize the core type directly, and delete the
  `Raw*` type and its conversion.
- A list, in the completion notes, of every `Raw*` type kept and why.

**Out (do not do):**
- A `Raw*` type whose conversion checks something, looks something up, or
  works a default out from other fields: it stays.
- The serde mirror enums of `content/src/enums.rs`: 1230 deletes them.
  Either order: skip any `Raw*` type 1230 already removed; if 1230 is
  not done, leave the enums to it.
- Changing a data file's format: the `.ron` files read the same before and
  after.
- Deriving `Default` or `Deserialize` on a core type whose fields don't
  all read from data as they are (ADR-0004: `core` stays pure; a serde
  derive is fine, I/O is not).

## Implementation steps

1. List the `Raw*` types (`grep -rn "struct Raw\|enum Raw"
   crates/content/src`) with each one's conversion (`fn into_*`, `From`,
   `TryFrom`, or the loader's `map`). Mark each "ids only" or "keeps"
   with its reason.
2. For each "ids only" type: give the core type `Deserialize` (with
   `#[serde(deny_unknown_fields)]` and the same `#[serde(default)]`s the
   `Raw*` type had), read it directly in the loader, and delete the
   `Raw*` type and its conversion.
3. Content errors keep their file and line: run `cargo test -p
   trpg-content`, whose error tests cover them, and add one for a type
   whose errors no test reads yet.
4. Measure with `cargo xtask code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [ ] Every `Raw*` type left in `crates/content/src` is named in the
      completion notes with the check or lookup that keeps it.
- [ ] No data file under `assets/`, `assets-test/` or
      `assets-private/content/` changes.
- [ ] `cargo test -p trpg-content` passes, its error tests unchanged in
      what they check (file and line included).
- [ ] The same tests pass, none removed (`cargo test --no-fail-fast`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing content loading and error tests, unchanged in what they
  check; a new error test only where a removed `Raw*` type's errors had
  none.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
