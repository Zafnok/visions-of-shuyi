---
id: "0059"
title: "Decide: the engine's license, its name, and when it goes public"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0059 — Decide: the engine's license, its name, and when it goes public

## Context

Nick, 2026-10-14: split the project into an engine that "can become
free/open source" and the game, which stays paid. Ticket 1101 looked at
whether that is feasible (`docs/engine/feasibility.md`) and ADR-0063 sets
out how the split is done. Three parts of it are Nick's alone:

- **The license.** `LICENSE` is his call (`CLAUDE.md` hard rule 6,
  ADR-0013). Today every file in the repository is under the custom
  source-available license; the engine's files would move to an
  open-source one. Every commit so far is Nick's or Claude's (and
  Dependabot's version bumps), and `LICENSE` section 7 gives Nick the
  rights to contributions, so he can relicense the engine's files himself.
- **The name.** The engine needs a name for its repository, its crates on
  crates.io and its website. It is branding, like the game's title (0012).
- **When it goes public as its own project**, and whether outsiders may
  send code to it (pull requests) or only report problems (issues).

Nothing in the engine work waits on this ticket until the engine is moved
to its own repository (1131) and licensed (1130). The work inside this
repository (1102 onward) uses the working name `engine` and stays under
`LICENSE` until then.

## Nick input

**Decision.** Run with the `ask-nick` skill's format (options from real
projects, a recommendation, "describe your own"), but say plainly that
these are licensing and branding choices, not game design, and that a
lawyer's look before the engine is published is cheap insurance (as
ADR-0013 already recommends for `LICENSE`).

## Scope

**In:** the questions below, the answers recorded in
`docs/engine/decisions.md` (a new file: Nick's words verbatim, then the
rule), and any change to 1130–1132 that the answers call for.

**Out (do not do):** editing `LICENSE` or adding license files (1130
does that after the answer, and only in the words Nick chose); creating
the new repository (1131); any code.

## Implementation steps

1. Read `docs/engine/feasibility.md`, ADR-0013, ADR-0063 and `LICENSE`
   sections 2, 6 and 7.
2. Ask, in this order:
   1. **The engine's license.** Options, each with who uses it and what it
      means for someone who builds a game on it:
      - **A. MIT OR Apache-2.0 (either, the user picks)** — Bevy, most of
        the Rust ecosystem, macroquad. Anyone may use it in a paid game;
        Apache adds a patent grant. Fits the crates it depends on.
      - **B. MIT only** — Godot, Defold's older parts. The shortest
        license; no patent grant.
      - **C. Zlib** — SDL, raylib. Like MIT, and a game need not even show
        the notice.
      - **D. MPL-2.0 or another "share your changes to the engine"
        license** — changes to engine files must be published; a game's own
        files need not. Say plainly that ADR-0013 denies MPL for anything
        the *game* ships, so choosing it for the engine needs ADR-0013
        amended for our own engine, and some game makers avoid it.
      - **E. Describe your own.**
      Recommendation: **A**, because it is what Rust developers expect and
      every crate the engine uses is already under it.
   2. **What stays the game's.** Confirm: the rules of Visions of Shuyi
      (classes, stats, Combat Arts, magic), its story, characters, maps,
      dialogue, music choices, bought art and the private repository never
      move to the engine; the engine ships with only free, open art (the
      Terminus font, and CC0 art for its samples). Ask whether the generic
      grid-tactics parts (a grid, movement ranges, turns and phases, the
      battle cursor) may be open as an engine "tactics kit" (ADR-0063
      assumes yes), or must stay in the game.
   3. **The name.** Ask for a name or a short list; Claude then checks
      crates.io, GitHub and a web search for clashes and reports back
      before anything is created (like 0906 does for the game's title).
      The name must not contain "Shuyi" (the game's and Nick's wife's
      name stays the game's).
   4. **Outside code.** Options: **A. issues and discussions only at
      first, code by Nick and Claude** (SQLite, early Litestream); **B. pull
      requests welcome under a Developer Certificate of Origin** (sign-off
      line on each commit; Linux, GitLab); **C. pull requests welcome under
      a Contributor License Agreement** (contributors grant Nick the right
      to relicense; Godot does not, Qt does). Recommendation: **A** until
      1131, then B.
   5. **When it goes public.** Options: **A. after the game's Chapter 1
      ships** (itch.io, 0901); **B. as soon as an example game runs on the
      engine alone** (1118); **C. only after the game ships on Steam**.
      Recommendation: **B**, but never before 0804 (Nick's Chapter 1
      playtest) is done.
3. Record each answer verbatim and as a rule in `docs/engine/decisions.md`;
   add a line to `docs/engine/README.md` pointing at it.
4. Update 1130, 1131 and 1132 where an answer changes them (the license
   files to add, the repository name, the contribution files).

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in
      `docs/engine/decisions.md`.
- [ ] If Nick gave a name, the clash check (crates.io, GitHub, a web
      search) is written down with links.
- [ ] 1130–1132 match the answers; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

*(Filled in by the session that completes the ticket.)*
