---
id: "1011"
title: "ADR: how mods work — the mod folder, layering over the assets, merging data, ids, saves, scripting, the modder's tool"
type: research
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["0054"]
nick_input: none
completed:
---

# 1011 — ADR: how mods work

## Context

Nick said yes to modding and to dev tools for it, including "a scripting
DSL or something" ([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Modding and dev tools*). What mods are allowed to change is 0054's answer.
This ticket makes the technical decisions once, in an ADR, so the
implementation tickets (1012–1024) don't each invent them.

Why this needs an ADR rather than code straight away: it touches rules that
other ADRs set.

- **All content is embedded** (ADR-0005). `trpg_content::bundle` reads only
  from `include_dir!` directories, laid in **layers** (the private art over
  `assets/`, ADR-0040), and returns `&'static` data. `content` does no file
  I/O; only `app` touches files (ADR-0004). The layering itself is Shuyi
  Engine's since the carve (1103).
- **Skill effects are a closed set of data variants, "no scripting, no
  per-skill code"** (ADR-0021). Scripting changes that, so the new ADR
  amends ADR-0021.
- **The core is deterministic**; a battle is saved as its history and
  replayed (ADR-0019, ADR-0039). Mods and scripts must keep that, or
  rewind, suspend, replays and the bots break.
- **The bots are dev tooling, never in the game** (`CLAUDE.md`). A
  modder's balance report (1021) runs them in a separate tool, never in the
  game binary.
- **The engine's parts (ADR-0063, 1157).** Three pieces of modding are
  general engine work, each an engine ticket the game's tickets wait for
  (in [Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)):
  - **engine#2207**: bundle layers of files handed in at run time, and the
    platform shell reading a folder on disk into one (engine ADR-0004 §1:
    a mod is one more layer). 1012 and 1014 use them.
  - **engine#2101**: the script runtime, the engine module `shuyi-script`,
    with an engine ADR that picks the language. 1017 uses it.
  - **engine#1129**: the engine's tool contract (engine ADR-0005); the
    modder's tool (step 2.9) should be built on it if it exists, so a
    modder's checks are the same as a game developer's.

  This ADR decides the game's side of each and follows the engine's
  choices where those tickets are done; where they aren't, it takes the
  defaults below and says so.

## Nick input

None (technical). If 0054 is answered differently from what this ADR
assumes, follow 0054.

## Scope

**In:** one ADR (the `write-adr` skill; the next free number, 0054 at the
time of writing, but check the index and open PRs), answering every
question in step 2; the ADR index row; updates to 1012–1024 where the ADR
picks differently from the defaults they state.

**Out (do not do):** any code beyond a throwaway spike (not committed);
the Workshop's details (1024); the mod policy (0054).

## Implementation steps

1. Read ADR-0004, 0005, 0019, 0020, 0021, 0030, 0038, 0039, 0040, 0045,
   `crates/content/src/bundle.rs`, `crates/content/src/lib.rs`
   (`load_embedded`), `crates/core/src/save.rs` and 0054's answers.
2. Decide, and write in the ADR with the reasons and the options turned
   down:
   1. **The mod package.** A folder (and the same as a zip) holding a
      `mod.ron` manifest (id, name, version, author, the game version it
      was made for, mods it needs, mods it can't run with) and files laid
      out like `assets/`. Default: yes, exactly that.
   2. **Where mods live.** Default: a `mods/` folder beside the executable
      on native builds, plus the player's data folder; a `load_order.ron`
      in the player's settings names the enabled mods in order. The web
      build: per 0054 (default: not supported yet).
   3. **How `content` reads them without file I/O.** Default: `app` reads
      the enabled mods' folders once at startup (engine#2207's
      `read_folder`) and hands `content` a list of runtime layers (path →
      bytes, engine#2207), laid over the embedded layers in load order
      (last mod wins), then over the private art and `assets/`. The
      engine keeps the bytes for the program's life, so `bundle`'s
      `&'static` API stays. Changing the enabled mods takes effect at the
      next launch.
   4. **Replacing a file versus merging entries.** A mod's `.map`, `.dlg`,
      image or sound file replaces the file at the same path. A mod's
      `data/*.ron` table **adds and replaces entries by id** instead of
      replacing the whole table, and may list ids to remove. Default: yes;
      say which tables are keyed by what (1013 implements it).
   5. **Ids.** Default: plain ids; a clash between two mods is an error
      naming both mods, unless the later one marks the entry as an
      intentional override.
   6. **Validation.** The same `load_*` checks run on the merged result;
      every error names the mod and file it came from (`display_path` must
      show `mods/<id>/data/items.ron`).
   7. **Saves.** A `SaveFile` records the enabled mods' ids and versions.
      Loading a save whose mods are missing or changed warns first. Whether
      a modded save is kept apart: 0054.
   8. **Scripting.** The language and the runtime are the engine's
      (engine#2101: its ADR compares Rhai, Lua through `mlua`, WebAssembly
      modules through `wasmi` and a small language of our own, for
      licence, size on native and WASM, determinism and how a modder
      writes a rule; default Rhai, integer-only, with an operation limit).
      Take its choice, or its default if it isn't done yet, and check the
      licence against ADR-0013. Say **where** scripts may run in this
      game: inside `core` as pure functions over a read-only view of the
      battle that return data `core` already understands (effects,
      events, a win or a loss), never touching state directly. Say what
      this amends in ADR-0021.
   9. **The modder's tool.** Default: a separate binary, `vos-modkit`, in a
      new `crates/modkit` crate, shipped beside the game: `check`, `new`,
      later `playtest` (with `trpg-bots`; it's never linked into the game).
   10. **What tools never do**: export, copy or re-save bought art
       (ADR-0040, 0054).
3. Do a short spike to prove the riskiest default this game decides
   (where scripts run in `core`, and laying mod layers over the game's;
   engine#2101 and engine#2207 prove the engine's parts), and record
   what it showed. Don't commit the spike.
4. Write the ADR, add the index row, set ADR-0021's status line to
   "amended by ADR-NNNN" if scripting is accepted.
5. Re-read 1012–1024 and change any step the ADR contradicts.

## Acceptance criteria

- [ ] The ADR answers every point in step 2, with options turned down and
      why.
- [ ] The scripting choice is engine#2101's (or its default, said so);
      its licence passes ADR-0013, and its size cost on native and WASM
      is the measured one from engine#2101's ADR, or measured here if
      that isn't done.
- [ ] The ADR index lists it; ADR-0021's status line is updated if needed.
- [ ] 1012–1024 agree with the ADR; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
