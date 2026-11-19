---
id: "0090"
title: "Decide: does the writers' script preview follow the settings changed in the demo?"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: low
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0090 — Decide: does the writers' script preview follow the settings changed in the demo?

## Context

The writer trial (0766) sends each applicant two web pages on itch.io:

- the Chapter 1 demo (0914, 0915);
- the script preview (0767, 0771), which plays the `.dlg` files a writer drops on it.

A test audit before the packets go out found that the two pages share one store of saved settings.

- Both programs save under the storage name `tactical-rpg`:
  - `crates/app/src/app_info.rs`: `.with_storage("tactical-rpg")`;
  - `crates/story-preview/src/main.rs` reuses that file, saying "so the preview has its name, icon and storage".
- `docs/releasing.md` says every web page (the playtest page, the demo, the preview) is served from `html-classic.itch.zone`. A browser keeps one store per site, so the pages read each other's settings.
- The preview applies what it finds:
  - the key layout (`crates/ui/src/story_preview.rs`, `game::use_saved_layout`);
  - Auto-advance (`crates/ui/src/screens/dialogue.rs`, `auto_advance()` reads `ctx.settings().game.auto_advance`);
  - the portrait look (`portrait_style`);
  - the voice log and the volumes (`crates/ui/src/settings.rs`, `docs/design/options.md`).
- The preview has no Options screen, so a writer cannot change any of these there.

A player situation:

> An applicant plays the demo, turns Auto-advance On, switches the
> portraits to the glyph look and turns the music down. Then they open
> the preview and drop in their trial scene. Today their lines advance
> on their own, in the glyph look, with quiet music, and nothing in the
> preview turns that back.

The preview also uses the demo's key layout. Ticket 0764 meant that on purpose for the hired writer's Windows kit ("the key layout chosen in the game is used here").

Not part of this question (the agent's call, done in the build ticket 0774): the web demo gets a saved-settings store of its own. Then Nick's playtest page (every chapter, debug tools) stops leaking its saves and settings into the demo. Exporting a demo save into the full game is unaffected: it goes by file (`docs/design/demo.md`, rule 5).

Until this is built, test as an applicant in a new browser profile, in a normal window (or after clearing the site data of `html-classic.itch.zone`). Not a private window: it may refuse the framed page its storage (0931).

This is not a blocker of the writer trial (0766): the packet already tells applicants to use a normal window, and the leak only reaches an applicant who changed settings in the demo before opening the preview.

## Nick input

**Decision** (`ask-nick`), asked as the situation above. Options:

- **A. The preview keeps its own settings.** Nothing changed in the demo reaches it. It starts on the game's default settings and asks for a key layout on the first key, as a first launch of the game does. *Like inkle's Inky editor, whose play pane runs a story with none of a game's settings.*
- **B. Only the keys follow.** The layout picked in the demo carries over. Auto-advance, the portrait look and the volumes start at the game's defaults in the preview. *Like the Nintendo Switch's system button remapping, which applies in every game while each game keeps its own options.* (Before asking, the session confirms this comparison holds, or swaps it for a clearer example from a real game.)
- **C. Everything follows, and the preview gets the game's Options.** The demo's settings carry over as today. A key in the preview opens the same Options screen the game has, so a writer can change them there too. *Like a Ren'Py visual novel, whose Preferences open from inside any scene.*
- **D. Leave it as it is.** Everything follows and the preview has no Options. A writer who wants different settings changes them in the demo.
- **E. Describe your own.**

The desktop kit for the hired writer (the `story-preview` edition) stays as it is unless Nick says otherwise.

## Scope

**In:**
- Ask the question.
- Record Nick's answer in his words in `docs/design/demo.md`, under a new section *The demo and the script preview's settings*, with a row in `docs/design/README.md`.
- Write the answer into the build ticket 0774 (which option, and any of his words that change its steps).

**Out (do not do):**
- Building it (0774).
- The web demo's own storage (the agent's call, 0774).
- The Windows demo and the desktop kit.
- Mockups: this is a behavior, not a look. If Nick asks to see one, use `cargo shuyi frame-png` pictures of the preview with Auto-advance and the glyph look on.

## Implementation steps

1. Read this ticket's context, `docs/design/demo.md`, `docs/design/options.md` (*Auto-advance*, *Volumes*) and 0764's and 0767's completion notes.
2. Ask with the `ask-nick` skill: the situation, then options A to E as written above.
3. Record his answer and words in `docs/design/demo.md` and `docs/design/README.md`.
4. Edit 0774: state the chosen option at the top of its Context and keep only that option's steps.
5. If Nick picks E, rewrite 0774's steps to match, keeping its tests and its estimate of size.

## Acceptance criteria

- [ ] `docs/design/demo.md` has *The demo and the script preview's settings* with Nick's answer and words, and `docs/design/README.md` has its row.
- [ ] 0774 names the chosen option, and its steps and acceptance criteria cover only that option.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
