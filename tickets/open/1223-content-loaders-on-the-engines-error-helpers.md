---
id: "1223"
title: "Content loaders on the engine's error helpers: one call to fetch a file, to report at an entry, to refuse a duplicate id"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2247"]
nick_input: none
completed:
---

# 1223 — Content loaders on the engine's error helpers

## Context

From the game/engine boundary audit of 2026-11-03. The content crate
hand-writes three error patterns the engine has, or now gets, in one
call, and drops the engine's problem codes on the way:

- **The fetch.** 11 loaders build "file not found in asset bundle" with
  `ContentError::new` (so no `SH2003`): `class.rs:78`, `spell.rs:77`,
  `character.rs:191`, `chapter.rs:188` (`load_new_game`), `support.rs:62`,
  `lesson.rs:278`, `terrain.rs:97`, `tip.rs:114`, `camp.rs:60`,
  `ink.rs:24`, `title.rs:54`, about 6 lines each; `formats::load`
  (`formats.rs:66`) builds it from `NOT_FOUND`.
- **The entry's line.** 12 closures or `Validator::err` bodies of
  `match line_of(source, &format!("id: \"{id}\"")) { Some(l) => e.at(l,
  None), None => e }`: `art.rs:60` and `:160`, `camp.rs:91`,
  `character.rs:222` and `:356`, `class.rs:140`, `skill.rs:74` and
  `:211`, `spell.rs:100`, `:222` and `:600`, `terrain.rs:174` (and
  `item.rs`'s `err_at_item`).
- **The duplicate id.** 6 `insert(..).is_some()` checks with no
  `SH2009`: `art.rs:86`, `character.rs:279`, `class.rs:105`,
  `item.rs:303`, `skill.rs:112`, `spell.rs:162`.
- **The light.** `battle::light_of` (`battle.rs:304-308`) re-implements
  `shuyi_assets::grid_map::header_position` inline, beside
  `map::light_error`, which calls it; both build "is not a light; the
  lights are …".

[engine#2247](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2247-loader-errors-in-one-call-each.md)
adds `Bundle::source`, `ContentError::near` and an `Entries` helper with
`insert_unique`. `ContentError::not_found` and `header_position` are
already at the pin.

## Nick input

None. Every message and line stays the same; errors gain their codes,
and a data file that isn't UTF-8 now says so (`SH2002`) where it said
"not found".

## Scope

**In:**
- Move the pin past engine#2247's merge (`engine-change` skill §7, the
  `engine-pin` skill), unless an open `engine-pin` PR already has.
- `crates/content/src/bundle.rs`: `pub fn source(path) ->
  Result<(String, &'static str), Vec<ContentError>>` over
  `install().source(path)`; the 11 loaders call it; `formats::load` uses
  `ContentError::not_found`.
- The 12 id-line sites and `item.rs`'s `err_at_item` on `Entries::err`
  (or `ContentError::near` where the source is another file's), and the 6
  duplicate checks on `Entries::insert_unique`, their messages unchanged.
- `battle::light_of` on `grid_map::header_position(&grid_map::header_lines(source), …)`,
  and one `not_a_light(field, light, lighting)` message builder shared
  with `map::light_error` (each keeps its own field label: `light:`,
  `look: light`).
- **Either order with 1186:** if 1186 is done, `battle::load_lights` and
  the chapter, battler and world map loaders are on the engine's folder
  loaders: make this ticket's changes where that code now is; if not,
  nothing to do (1186 keeps this ticket's calls as it moves the folder
  loops, and leaves the light check and the fetch here).
- **Either order with 1230:** if 1230 is done, `class.rs`,
  `character.rs` and `item.rs` read the core enums (`enums.rs` is gone):
  make this ticket's changes on that code; if not, nothing to do (1230
  changes the raw types after this ticket).
- **Either order with 1231:** if 1231 is done, some `lang/data_tests.rs`
  tests are gone: keep the messages of the rest unchanged; if not,
  nothing to do (1231 deletes tests as this ticket left them).
- **Either order with 1233:** if 1233 is done, the `lib.rs` row
  (`docs/engine/boundary.md:180`) says "moves with 1223 and 1236":
  write this ticket's part as moved, with step 4's row; if not, update
  the rows here, and 1233 leaves them.

**Out (do not do):**
- Empty-id checks (their messages differ; they stay one line each).
- `battle.rs`'s enemy `id` "is already used" check and `item.rs`'s weapon
  kinds "listed twice": not id tables.
- The folder loaders (1186).

## Implementation steps

1. **Pin** (as above).
2. Read engine#2247's API, `crates/content/src/bundle.rs`, the files
   above and `map.rs` (`check_lights`, `light_error`).
3. `bundle::source`, then the 11 loaders and `formats::load`; then the
   id-line sites; then the duplicate checks; then `light_of`. Content
   tests green after each.
4. Drop `terrain.rs`'s `line_of` re-export if nothing uses it any more,
   and update its row in `docs/engine/boundary.md`.
5. **Measure** with `cargo xtask code-lines`: estimate about 15
   production lines added, about 100 removed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2247.
- [ ] `grep -rn "file not found in asset bundle" crates/content/src`
      finds only tests, and `grep -rn 'line_of(.*"id: ' crates/content/src`
      finds nothing.
- [ ] A missing data file's error has code `SH2003`, and a duplicate
      class id's `SH2009` (tests).
- [ ] The content tests' expected messages are unchanged
      (`font.rs`, `formats/tests.rs`, `lang/data_tests.rs` included).
- [ ] `light_of` has no line or column search of its own.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the existing content tests; the two code tests above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
