---
id: "1236"
title: "One check-if-loaded helper in the content load graph: the six check_* adapters and the two error tails go"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1236 — One check-if-loaded helper in the content load graph

## Context

The game/engine boundary audit of 2026-11-03 (findings content-formats#4
and content-data#3, one job) read `crates/content/src/lib.rs`. Six
adapters have the same 15–17 lines: `check_seals` (608),
`check_art_references` (707), `check_skill_references` (727),
`check_spell_references` (747), `check_map_features` (767) and
`check_map_lights` (788). Each is

```rust
match (result, deps…) {
    (Ok(v), Some(d)…) => { let errors = module::check(&v, d…);
        if errors.is_empty() { Ok(v) } else { Err(errors) } }
    (r, ..) => r,
}
```

`check_places` (537) and `check_presence` (559) end with the same
`if errors.is_empty()` tail. All of it is plumbing, not rules, and the
engine has nothing for it (engine `docs/formats.md`, *Checks after
loading*, leaves cross-checks to the game). `Loaded` and `assemble`
stay: `load_worldmaps` and `data_text` read earlier results through
`Loaded` before `Content` exists.

**Either order with 1013** (mods merge tables by id), which runs these
same checks on the merged tables (its step 4). If 1013 is done, the
helper wraps the merged loaders' results the same way; if not, 1013's
step 4 calls `checked` where it says `check_seals` and the rest.

## Nick input

None.

## Scope

**In:**
- Two private helpers in `crates/content/src/lib.rs` and the eight
  call sites on them. Every check, and which failures skip it, unchanged.
- Rewriting the adapters' tests onto the helper.

**Out (do not do):**
- `Loaded`, `assemble`, `take` and the `let … else` skip blocks of
  `load_story`, `load_tilesets` and the cinematic loaders (they carry
  their own skip rules and are short).
- The per-table loaders' own error idioms (`class.rs`, `spell.rs`, …):
  1223 moves them onto the engine's error helpers.
- An engine `shuyi_assets::error` helper (optional; not worth a ticket
  for one game).

**Either order with 1233:** if 1233 is done, the `lib.rs` row
(`docs/engine/boundary.md:180`) says "moves with 1223 and 1236": write
this ticket's part ("run every loader, collect every error") as done;
if not, add a line to that row here, and 1233 leaves it.

## Implementation steps

Expected size: about 100 production lines removed and 20 added (net
about −80), and about 40 test lines removed.

1. Add, near `load`:
   ```rust
   /// `result`, with `check`'s errors when it and `dep` loaded; skipped
   /// (the result unchanged) when either failed.
   fn checked<T, D>(result: Result<T, Vec<ContentError>>, dep: Option<D>,
       check: impl FnOnce(&T, D) -> Vec<ContentError>) -> Result<T, Vec<ContentError>>
   /// `value`, or `errors` when there are any.
   fn ok_unless<T>(value: T, errors: Vec<ContentError>) -> Result<T, Vec<ContentError>>
   ```
   `checked` is `match (result, dep) { (Ok(v), Some(d)) => { let e = check(&v, d); ok_unless(v, e) } (r, _) => r }`.
2. In `load` (lines 281–296) and `load_items` (600), replace each adapter
   call. Several dependencies zip:
   `checked(maps, items.as_ref().ok().zip(terrain.as_ref().ok()), |m, (i, t)| map::check_features(m, i, &t.rules))`,
   `checked(item::load(), classes, item::check_seals)`, and so on for
   arts, skills, spells (classes zipped with characters) and lights.
   Keep each adapter's doc line ("Skipped when … failed to load") as a
   one-line comment at its call.
3. Delete the six adapters. End `check_places` and `check_presence` with
   `ok_unless(scripts, errors)` / `ok_unless(scripts.table, errors)`.
4. Tests: `a_maps_light_is_checked_when_the_maps_and_the_lights_loaded`,
   `map_feature_checks_join_the_map_errors`,
   `spell_reference_checks_join_the_spell_errors`,
   `skill_reference_checks_join_the_skill_errors`,
   `seal_checks_join_the_item_errors` and
   `art_reference_checks_join_the_art_errors` each test the skip rule
   again. Move the skip rule into three tests of `checked` (both loaded,
   the dependency failed, the result failed) and keep, in each table's
   test, only the case where its check finds an error, called through
   `checked`.
5. Measure with `cargo xtask code-lines` and put the numbers in the PR
   (ADR-0092).

## Acceptance criteria

- [ ] `crates/content/src/lib.rs` has no `fn check_seals`,
      `check_art_references`, `check_skill_references`,
      `check_spell_references`, `check_map_features` or `check_map_lights`.
- [ ] `rg -c 'if errors.is_empty\(\)' crates/content/src/lib.rs` is 2 or
      fewer (`ok_unless` and `assemble`'s, which builds `ContentErrors`; 9 today).
- [ ] The three `checked` tests pass, and each table's error case still
      reports its check's errors.
- [ ] `embedded_content_loads` passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `checked` (three cases) and `ok_unless`; the six tables' error
  cases through `checked`.
- Integration: the existing `embedded_content_loads`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
