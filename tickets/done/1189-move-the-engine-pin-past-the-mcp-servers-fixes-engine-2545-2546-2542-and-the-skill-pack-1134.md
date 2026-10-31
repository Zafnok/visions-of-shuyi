---
id: "1189"
title: "Move the engine pin past the MCP server's fixes (engine 2545, 2546, 2542) and the skill pack (engine 1134)"
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: done
blocked_by: ["engine#2546", "engine#2542", "engine#1134"]
nick_input: none
completed: 2026-10-25
---

# 1189 — Move the engine pin past the MCP server's fixes and the skill pack

## Context

The game builds with the Shuyi Engine commit in `shuyi-engine.rev`
(`3f2256d4`, pinned by 1188). 1181 registered the engine's MCP server
here: `.mcp.json` runs `cargo shuyi mcp`, which is the game's own tool
program (`xtask`) on the pinned engine. On Nick's Windows machine
(2026-10-22) the server failed to connect in most new sessions. Four
engine tickets fix that and what follows from it, and Nick asked to
move the pin once all four are in (2026-10-22):

- **engine#2545** (done, engine PR Zafnok/shuyi-engine#76,
  `24b56b3`). On Windows, a running server locked its program, so the
  next session's `cargo run` couldn't relink it and the server closed
  before answering. Now the server and `mcp-host` move their running
  program aside. `mcp install` also writes `.claude/settings.json` with
  `env.MCP_TIMEOUT` at 5 minutes, so a cold build doesn't time out. Its
  notes leave one case unchecked, because the engine has no such
  project: **in a game, the server and the game's tool program are one
  binary.** That is this game: `.mcp.json`'s `cargo shuyi mcp` and the
  `start` tool's `cargo run -p xtask -- shuyi mcp-host` build the same
  `xtask`.
- **engine#2546.** Claude Code's desktop app starts a worktree
  session's server in the main checkout, so a relative `project` played
  the main checkout's game, not the worktree's edits. The server now
  reads the client's roots and hands each project to that checkout's own
  server, and its tool list follows. A server whose engine moved under it
  restarts by itself. So the `engine-change` skill's "reconnect it" after
  a pin move is no longer needed.
- **engine#2542**, the CLI's seams. The stale-server message names which
  side is newer (since 2546, it is written only when a restart fails),
  and each answer's output schema names its own op. Its PR says what API
  changed for the game, if anything.
- **engine#1134**, the engine's skill pack in the engine's `skills/`.
  The game's `keyboard-input` and `run-gates` overlap two of its skills
  (`shuyi-input`, `shuyi-gates`). Engine 1134 sets the rule for the
  engine's own skills only: one source, linked from the other.

Read each engine PR's body for "API change for the game's pin bump" before
starting. The engine writes one whenever an API changes (engine
`AGENTS.md`, *Working with the game repository*).

## Nick input

None. Nick asked for this pin move once engine 2546, 2542 and 1134 land
(2026-10-22).

## Scope

**In:**
- ~~`cargo xtask engine-pin <commit>` to the engine's `main`: its newest
  commit when the ticket is worked, at or past the commits that closed
  engine#2546, engine#2542 and engine#1134.~~ (done by the pin move of 2026-10-25, `3f2256d4` → `b4591a81`)
- ~~Whatever the game needs to build and pass the gates on the new pin,
  game side only, keeping every behaviour the same.~~ (done by the pin move of 2026-10-25, `3f2256d4` → `b4591a81`)
- `cargo shuyi mcp install --yes` in the root, then commit what it
  writes: `.claude/settings.json` (new, with `MCP_TIMEOUT`) and
  `.mcp.json` if it changed. If the game has a `.claude/settings.json`
  by then, install merges into it. Check that the merge kept every other
  key.
- The `engine-change` skill's step 6 (`.claude/skills/engine-change/SKILL.md`,
  the paragraph after `cargo xtask engine-pin`): drop "then reconnect it
  (`/mcp` in Claude Code, or a new session)". Keep `cargo shuyi mcp
  --ready`, which still builds the server and tool program before the
  next session connects. Say in one sentence that the server follows
  the pin by itself (engine#2546).
- **Check by hand on Windows** the case 2545 couldn't. With a Claude
  Code session's server running:
  1. `start` a battle;
  2. edit a line of game code that `xtask` builds (a comment in
     `crates/xtask/src/game_tools.rs` is enough);
  3. `start` again.

  Both starts answer. From a game worktree, `start` with a relative
  `project` builds the worktree's `xtask`, not the main checkout's:
  the stderr in Claude Code's MCP log for the session
  (`%LOCALAPPDATA%\claude-cli-nodejs\Cache\<project>\mcp-logs-shuyi\`)
  names the worktree's `target`. Write both results in the completion
  notes.
- **The skill pack:** read engine 1134's completion notes for how a
  game takes the shipped skills. If they name a step (a command, or files
  to copy), write a game ticket that takes them. Fold the game's
  `keyboard-input` and `run-gates` down to what holds in this game
  alone, linking the shipped `shuyi-input` and `shuyi-gates` for the
  rest, as engine 1134 did for the engine's own. If they name no step,
  say so in the completion notes.

**Out (do not do):**
- Taking the skill pack in this PR, or rewriting `keyboard-input` or
  `run-gates`: that is the follow-up ticket above, if one is needed.
- Engine changes. If the new pin breaks something the game can't fix
  on its side, stop and follow the `engine-change` skill.
- The engine watcher and its pin moves (1190). Either order: if 1190 has merged, this
  move also follows the `engine-pin` skill's review and writes the next
  section of `docs/engine/pins.md` (`engine-pin --check` fails
  otherwise), and it closes the open `engine-pin` issue if it pins the
  engine's `main`; if not, nothing changes here and 1190's baseline
  starts from this pin.

## Implementation steps

1. ~~`cargo xtask engine --off` if a local checkout is patched in.~~
2. ~~Read the bodies of the engine PRs since `3f2256d4`
   (`gh api repos/Zafnok/shuyi-engine/compare/3f2256d4...main`, then each
   PR) for API changes the game must follow.~~
3. ~~`cargo xtask engine-pin <engine main's commit>`.~~
4. ~~Build and run the gates (`run-gates` skill); fix what the new engine
   needs, game side only. If a snapshot changes, say why in the PR (the
   engine ticket that changed it).~~
   Steps 1–4 done by the pin move of 2026-10-25, `3f2256d4` → `b4591a81`: the pin is past engine#2546, engine#2542
   and engine#1134. Run the gates once more before the PR.
5. `cargo shuyi mcp install --yes`; look at the diff it prints; commit
   its files.
6. The `engine-change` skill's step 6, as in Scope.
7. The check by hand on Windows, as in Scope.
8. Engine 1134's notes, and the follow-up ticket if they name a step
   (`write-ticket` skill).

## Acceptance criteria

- [x] `shuyi-engine.rev` is at or past the commits that closed
      engine#2546, engine#2542 and engine#1134 (`gh api
      repos/Zafnok/shuyi-engine/compare/<commit>...<pin>` says `ahead` or
      `identical` for each).
- [x] `cargo xtask engine-pin --check` passes.
- [x] No behaviour changes: snapshots and play records unchanged, or
      each change explained by an engine ticket.
- [x] `.claude/settings.json` holds `env.MCP_TIMEOUT` and is committed.
- [x] `cargo shuyi mcp --ready` succeeds.
- [x] The `engine-change` skill no longer says to reconnect the server
      after a pin move.
- [x] The completion notes give both results of the check by hand, and
      the follow-up ticket's id or why there is none.
- [x] `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new; the existing suite on the new pin, and the check by hand
  above.

## Completion notes

- **Pin:** moved to `b4591a81` by the pin move of 2026-10-25 (PR #425), past
  engine#2546, engine#2542 and engine#1134; nothing more here.
- **`.claude/settings.json`** written by `cargo shuyi mcp install --yes`:
  `env.MCP_TIMEOUT` = `300000` (5 minutes). There was no settings file
  before, so nothing to merge. `.mcp.json` was "set up already" and is
  unchanged. Why it matters: on 2026-10-25 a new worktree's session
  failed with "connection timed out after 30000ms", because the first
  `cargo shuyi mcp` built `xtask` cold; once built it starts in about
  0.3 s. Install also wrote `.vscode/mcp.json` (git-ignored) and
  `.gemini/settings.json`; neither was committed, as in 1181.
- **`engine-change` step 6:** no longer says to reconnect; keeps
  `cargo shuyi mcp --ready` and says a running server follows the pin by
  itself (engine#2546).
- **Check by hand on Windows** (this session's Claude Code server, in a
  worktree):
  1. `start` answered (0.2 s), then `step` reached the Quick Battle map.
  2. A comment added to `crates/xtask/src/game_tools.rs`, then `start`
     again: answered ok in 4 s (the rebuild), stopping the old game on
     frame 134. No lock error.
  3. The worktree's `target/debug/xtask.exe` was relinked at the second
     start (13:30:47); the main checkout's `D:\tactical-rpg\target\debug\xtask.exe`
     kept its 13:10:42 time. So a relative `project` built the
     worktree's `xtask`. The MCP log (`mcp-logs-shuyi`) records the calls
     and times but not cargo's stderr, so the target path came from the
     files' times instead.
- **Skill pack:** engine 1134's notes name a step (the plugin
  `shuyi@shuyi-engine`, and `shuyi new` copying the pack into a game's
  `.claude/skills/`), so 1192 takes it and folds `keyboard-input` and
  `run-gates`.
- **Found while checking:** the server's `check`, `bots` and `replay`
  answer `SH3001` on this game: 1193. (`playtest` and `autobalance` are
  1180 and 1183.)
- No Rust code; no gameplay rules decided; nothing for Nick to check.
