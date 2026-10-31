---
name: engine-pin
description: Move the game's Shuyi Engine pin to the engine's main and review what the engine PRs it passes give the game, for a GitHub issue labelled engine-pin (filed by the engine watcher), or for the engine PRs a game ticket's own pin move passes. Use when pointed at an engine-pin issue, asked to move or bump the engine pin, or when a ticket moves the pin itself (engine-change §3.6 and §7). Not for making an engine change (engine-change).
metadata:
  version: "1.0"
---

# Move the engine pin

One `engine-pin` GitHub issue is one PR. The issue body **is** the ticket:
the second exception to "work one ticket at a time from `tickets/open/`"
(`AGENTS.md` hard rule 1,
ADR-0096),
like a `sonar` issue. There is no ticket file to write or move.
`.github/workflows/engine-watch.yml` files the issue and keeps it up to
date as the engine's `main` moves.

**Model:** `opus-5.5 / gpt-6.1-sol`, effort `medium` (`tickets/README.md`).

`work-ticket` still governs everything this skill doesn't change: gates,
never weakening a gate, one PR, then stop. The PR stays **mechanical**:
the pin, the break fixes, the ledger section, the tickets. Adopting an
engine piece is ordinary ticket work, never done in the pin PR, and a
review that needs a design call goes to a ticket (`write-ticket`; the
`ask-nick` skill when it is Nick's), never decided here.

A game ticket that moves the pin itself (`engine-change` §3.6 and §7)
does steps 4 to 7 for the engine PRs its move passes, in its own PR.

## Steps

1. **Read the issue** (`gh issue view <n>`): the four sections, why the
   move is due, and the adoption tickets still open.

2. **Re-run the report.** The engine's `main` may have moved since the
   issue was written, so pin the newest:

   ```bash
   git switch main && git pull --ff-only
   cargo xtask engine-pin --report
   ```

   Work from this report, not the issue's, and note `main`'s commit.

3. **Branch, pin, build.** With `<8 hex>` the first eight of that
   commit:

   ```bash
   git switch -c engine-pin/<8 hex>
   cargo xtask engine --off
   cargo xtask engine-pin <main's full commit>
   cargo xtask skill-pack
   pwsh -File scripts/sync-agent-skills.ps1
   ```

   `skill-pack` builds with the new pin and copies its skill pack into
   `.claude/skills/shuyi-*` (ADR-0097); commit what it changes. When a
   pack skill this game folds into its own changed (`shuyi-input` under
   `keyboard-input`, `shuyi-gates` under `run-gates`), take out of the
   game's skill any rule the pack's now states.

   Run the gates (`run-gates`). Fix every *Breaking* row **on the game
   side, with behaviour unchanged**: the row's `Breaking:` lines say what
   changed (a type moved onto `new`, a field added). A snapshot that
   changes needs its engine PR named in the PR body as the reason. Then
   `cargo shuyi mcp --ready` (the MCP server builds on the new engine).
   A break that can't be fixed on the game side needs an engine change:
   file the engine ticket (`engine-change`, "ticket and follow-up") and
   say so in the PR; if the game can't build without it, pin the commit
   before the breaking PR instead and say why.

4. **The adoption review**, one decision per *New for games* row. Ask:
   - Does the game already have code doing this job? Search `crates/`,
     the rows of `docs/engine/friction.md` for that area, and
     `docs/engine/boundary.md`.
   - Does an open ticket plan to write code for it?
     `grep -rln "<the piece's name or job>" tickets/open`.

   Then record exactly one of:
   - **`adopt → NNNN`**: a game ticket (`write-ticket`) that moves the
     game onto the engine's piece and deletes the game's copy, stating the
     lines it expects to remove; or an open ticket `NNNN` whose text you
     change to use the engine's piece instead of writing its own. Example:
     the engine ships threat areas and `crates/bots` computes its own
     danger map → a ticket that switches the bots to the engine's and
     deletes theirs, "about 120 lines removed".
   - **`not for the game: <why>`**: one line. Example: "templates for new
     games", "the game has no modules".

   When adopting needs an engine change first (the engine's piece almost
   fits), file the engine ticket in the engine's repository (its
   `write-ticket`, and the game's `engine-change`, "ticket and
   follow-up"), block the adoption ticket on it (`blocked_by:
   ["engine#MMMM"]`), and write `adopt → NNNN (after engine#MMMM)`.

5. **Unblocks.** Those game tickets are now ready; the pin PR does nothing
   else for them. In each one whose text says its first step moves the
   pin, strike that step ("done by the pin move of YYYY-MM-DD").

6. **The ledger section**, appended to `docs/engine/pins.md` (its header
   says the form):

   ```markdown
   ## 2026-10-26: 3f2256d4 → 6e7f71fd (27 engine PRs), #430

   | Engine PR | Ticket | What | Decision |
   | --- | --- | --- | --- |
   | #88 | 2013 | Public types moved onto constructors | fixed: `Window::new` and `AppConfig::new` in `crates/app` |
   | #102 | 2131 | Threat areas in the tactics kit | unblocks 1180 |
   | #78 | 2511 | Engine defaults, a prelude, `run_game` | adopt → 1191 |
   | #93 | 2358 | Modules join a running game | not for the game: the game has no modules |

   Nothing for the game: 12 PRs (#75, #79, …)
   ```

   A row for every *Breaking*, *Unblocks* and *New for games* PR; the
   game PR's number once it exists.

7. **Check:** `cargo xtask engine-pin --check` (the ledger's last
   section is the pin, every row has a decision, every `adopt → NNNN`
   names a ticket), `cargo xtask skill-pack --check` and `cargo xtask
   ticket-lint`.

8. **Commit and PR.** Commit as `Engine pin: <old 8> → <new 8>`. The PR
   is titled `Engine pin: <old 8> → <new 8> (N engine PRs)`, and its body
   has `Closes #<issue>`, the ledger section, the tickets written or
   changed, and an `Engine friction:` line ("none: no game code" when the
   fixes are only call sites). No bare `@word` (`AGENTS.md`): engine PR
   titles can have them, so put them in backticks. Then stop.

## Rules that trip agents up here

- **Pin the engine's `main`**, never a branch commit; `--report` names
  it.
- **Two pin moves open at once:** the second rebases on `main` once the
  first merges, pins the engine's `main` again and extends its section
  to cover the PRs since the first's pin.
- **Don't edit the issue body.** The watcher rewrites it; the PR closes
  it.
- **The ledger is the review.** A PR without its section fails CI
  (`engine-pin --check`), whether an issue or a ticket moved the pin.
