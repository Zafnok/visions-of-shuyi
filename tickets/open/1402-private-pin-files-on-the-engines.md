---
id: "1402"
title: "Each change's private pin file through the engine's private-assets: assets-pin and resolve.sh go"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3445"]
nick_input: none
completed:
---

# 1402 — Each change's private pin file through the engine's `private-assets`

## Context

The third engine-friction sweep (1397, row C3) found that ADR-0135's
scheme names nothing of the game. Under it, a base pin moves only
forward, each change has its own pin file
`assets-private.d/<branch>.rev`, and the effective pin is the base
merged with each pin file. ADR-0135 itself names "pin files read by the
engine's `private-assets` itself" as the general form, to follow as an
engine ticket.

The game has it in three places:

- `crates/xtask/src/assets_pin.rs`, about 400 production lines (tickets
  0194 and 0195);
- `.github/actions/private-assets/resolve.sh`, 77 lines;
- the fold in `private-land.yml`.

Engine ticket 3445 adds `pin_dir`, `--add`, `--resolve`, `--check` and
`--fold` to `shuyi private-assets`. The files are marked
`engine-friction: engine#3445, moves (game 1402 deletes it)`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3445 if it is behind (`engine-change` §7).
- Set `pin_dir: Some("assets-private.d")` in `shuyi.ron`'s
  `private_assets`. `pin_file` stays `assets-private.rev`.
- The private-assets action calls `cargo shuyi private-assets
  --resolve`, or the engine's action if the template's fits. The fold
  in `private-land.yml` calls `--fold`.
- `cargo xtask assets-pin` goes. The `content-change` skill, ADR-0135's
  "How" section and `AGENTS.md` name `cargo shuyi private-assets --add`
  instead. ADR-0135 gets a line saying the engine now holds the scheme.
- Delete `assets_pin.rs`, `resolve.sh` and their lines in `main.rs`.

**Out (do not do):**
- Changing the scheme: the same files, order and checks.
- Touching the private repository.

## Implementation steps

1. Write the engine-friction estimate: about 30 lines added, about 480
   removed.
2. Move the pin, if needed. Set the config.
3. On a branch with a pin file, the effective pin from the engine is
   the one `resolve.sh` gives. Compare both on the branch before
   deleting.
4. Switch CI and the fold. Delete the old code and update the docs.

## Acceptance criteria

- [ ] `grep -rn "assets-pin\|resolve.sh" .github crates .claude/skills AGENTS.md`
      finds nothing (ADR text aside).
- [ ] CI's private-assets checkout on the PR resolves to the same
      commit as before (the job's log line, quoted in the completion
      notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the engine's tests cover the scheme. The PR's CI is the
  proof.

## Completion notes

*(Filled in by the session that completes the ticket.)*
