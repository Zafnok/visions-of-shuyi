---
id: "1394"
title: "The MCP server's long calls as tasks in a worktree session: checked by hand and written down"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 1394 — The MCP server's long calls as tasks in a worktree session

## Context

The pin move of 2026-11-27 (`10cbb98b → 230080db`, `docs/engine/pins.md`)
brought two engine PRs that change how the MCP server (`cargo shuyi mcp`,
`.mcp.json`, ticket 1181) answers its long tools:

- engine PR #629 (engine ticket 3436, engine ADR-0159): on the 2026-07-28
  protocol, to a client that declares MCP's tasks extension, `start`
  (without `link`), `playtest`, `autobalance`, `bake` and `export` answer
  a task at once, which the client polls with `tasks/get` and can stop
  with `tasks/cancel`. A session tool called while `start` builds answers
  `SH0002` naming the task;
- engine PR #638 (engine ticket 3440, engine ADR-0162): the same for a
  call forwarded to another workspace's server.

Every session of this game runs in a worktree
(`.claude/worktrees/<name>/`), and the `shuyi` server Claude Code starts
is the main checkout's, which forwards a worktree's calls to a server built
in that worktree: exactly #638's case. Neither engine PR could check it in
Claude Code by hand (both say so: a worktree session's server is built from
`main`, and a second Claude Code couldn't be started). This game is where
it can be checked, once the main checkout's server is built on a pin that
has both.

Today a worktree session's first `start` holds the call for the whole build
(minutes on this machine). `MCP_TIMEOUT` in `.claude/settings.json` (ticket
1189) is the server's own start, not a tool call, and stays.

## Nick input

None.

## Scope

**In:**
- By hand, in a worktree session, after the main checkout is on a pin past
  `230080db` and its `trpg-tools` is rebuilt: call `start` on the game, then
  `step` while it builds, then `step` after. Record whether Claude Code
  declares the tasks extension (the call answers at once and `step` answers
  `SH0002` naming the task) or not (the call waits as before).
- `.claude/skills/work-ticket/SKILL.md` §4 (the MCP session tools) and
  `docs/engine/README.md` (*The MCP server*): one or two sentences on what
  a session sees while `start` builds, as found.

**Out (do not do):**
- Any change to the engine or to `.mcp.json`'s server line.
- `MCP_TIMEOUT` (the server's start, 1189).
- Turning the extension on in Claude Code's settings if it isn't: if it
  isn't declared, write that down and stop.

## Implementation steps

1. Check the main checkout's `shuyi-engine.rev` is past `230080db` and
   rebuild its tools (`cargo build -p trpg-tools` there, or the
   `reference-shuyi-mcp-connection-closed` steps if the exe is held).
2. Start a new worktree session and run the calls in *In*. Note the times.
3. Write the sentences in the two docs.

## Acceptance criteria

- [x] The completion notes say whether a worktree session's `start` answered
      a task, with the times of the `start`, the `step` during the build and
      the `step` after.
- [x] `work-ticket` §4 and `docs/engine/README.md` say what a session sees
      while `start` builds.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None: a by-hand check of the agents' tools and a doc change.

## Completion notes

**Found: Claude Code doesn't declare the tasks extension, so `start` still
holds the call.** Checked 2026-11-27 in this ticket's own worktree session
(`mcp-servers-long-calls-tasks-11c7d9`), with the main checkout at
`230080db` and its `trpg-tools.exe` rebuilt at 20:57:08 (the server's own
`cargo run` rebuilt it when this session connected, so step 1 needed no
manual build and no exe moved aside).

- Claude Code's `shuyi` log (`mcp-logs-shuyi`) for the connection:
  `protocolEra: legacy`, `negotiatedProtocolVersion: 2025-11-25`. The
  2026-07-28 protocol, which engine PRs #629 and #638 need before they
  answer a task, wasn't negotiated. The log also says Claude Code skipped
  its `server/discover` probe because it *"remembered"* this server as a
  legacy peer from an earlier connect, so even a server that answers the
  probe now isn't asked again. That memo is Claude Code's own state, out
  of scope here (no client setting changed).
- `start` (project `.`): sent 01:57:42.373Z, answered 01:59:26.436Z,
  **1m 44s**, a full answer (title screen), no task.
- `step` "during the build": sent in the same message as `start`, but
  Claude Code didn't send it until `start` had answered (01:59:26.450Z),
  so no `step` ran during the build and no `SH0002` came back. It
  answered in **2 ms**, on frame 1.
- `step` after: that same call, **2 ms**. Then `stop`.
- Docs: `docs/engine/README.md` (*The MCP server*) and `work-ticket` §4
  say what a session sees (the call held for the build, other `shuyi`
  calls queued behind it, no task).

No deviation from the steps but the one above: the two `step`s collapsed
into one, since the client queued the first. No follow-up ticket: the
server is ready, and the rest is Claude Code's to change. No engine or
`.mcp.json` change; `MCP_TIMEOUT` untouched.
