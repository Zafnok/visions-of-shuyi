---
id: "1181"
title: "The engine's MCP server on the game: its session, `.mcp.json`, and a reconnect when the pin moves"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["1177", "engine#1133"]
nick_input: none
completed: 2026-10-22
---

# 1181 — The engine's MCP server on the game

## Context

Engine ticket 1133 (`github.com/Zafnok/shuyi-engine/blob/main/tickets/done/1133-mcp-server-over-the-tool-contract.md`,
engine ADR-0031) made `shuyi mcp`, the engine's tool contract as an MCP
server. An agent gets each `cargo shuyi` operation as a tool, and a
session keeps one headless game running between calls: `start`, `step`,
`scene`, `frame_png` (a picture), `events`, `reset`, `stop`. The engine's
`docs/contract.md`, *The MCP server*, says how it works.

`cargo shuyi mcp install` registers it for Claude Code in `.mcp.json` as
`cargo shuyi mcp`, the project's `command`. That runs the game's own
tool program (xtask), built on the pinned engine, so the server moves
with the pin and needs no install of its own. What the game lacks today
(checked 2026-10-22 on the engine's `main` after 1133):

- **The session's host can't start.** The server runs the game's tool
  program (`shuyi.ron`'s `game: (package: "xtask")`) as `cargo run -p
  xtask -- mcp-host`. `crates/xtask/src/main.rs` passes only a fixed list
  of commands to the engine's tools (`check-keys`, `check-text`,
  `lang-status`, `font-atlas`, `frame-png`, `web`, `private-assets`);
  `mcp-host` falls through to "unknown command". The same list leaves
  out the contract's operations (`check`, `run`, `scene`, `bots`, …), and
  xtask's own `playtest` shadows the contract's `playtest`. `cargo shuyi`
  itself works, because its alias is `xtask shuyi`, which hands
  everything to `shuyi_cli::main_with`.
- **The game offers no session.** `crates/xtask/src/game_tools.rs`'s
  `Trpg` gives `frame` and `lang` only, so `start` answers `SH3001`
  (the game's tools don't offer it).
- **Nothing reminds a session to reconnect after a pin move.** The
  server is built when Claude Code connects and runs until the session
  ends: after `cargo xtask engine-pin` it is the old engine's until
  reconnected. Engine ticket 2535 makes the server say so, and adds
  `shuyi mcp --ready`, which builds it so the first connection doesn't
  time out (engine ADR-0031; 2535 links this ticket).

1177 moves the pin to the engine's `main`, which holds 1133: this ticket
waits for it and doesn't move the pin itself.

## Nick input

None.

## Scope

**In:**
- **Route every command to the engine's tools:** `shuyi.ron`'s game
  program becomes `game: (package: "xtask", args: ["shuyi"])`, so the
  engine runs `cargo run -p xtask -- shuyi <command>` for every command
  that runs the game (`GameProgram::args`, engine `crates/cli/src/project.rs`).
  New engine commands then need no game change. `cargo xtask`'s own
  commands, its own `playtest` among them, stay as they are; the list in
  `main.rs` stays for the old names `cargo xtask frame-png …`.
- **`GameTools::session`** for `Trpg`: the game started as `frame`
  starts it (`Start::of`, the same setup, the shipped story, idle
  motion), as an engine `Session`:
  - its **scene**: on a battle, the battle map's `MapScene` as text
    (`MapScene::to_text`, as `Harness::map_text` writes it) and `data`
    the battle's state as JSON where it serializes (else `null`); no
    scene on other screens, saying the top screen;
  - its **picture**, for `frame_png`: the painter `frame` already uses;
  - its **events**: none of the game's own yet (screens, music, voices
    and the quit come from the engine).
  The engine's `Headless` (`shuyi_cli::contract::session`) is built over
  the engine's `Harness<G>`, and this game's `trpg_ui::harness::Harness`
  wraps one. Prefer `Headless::new` over the inner Harness, with
  `with_scene` and `with_picture`; move the map lookups the scene needs
  (`map_scene`, `map_text`) to functions the wrapper and the scene both
  call. If that doesn't fit, implement `Session` on a wrapper of the
  game's Harness instead, and run the `engine-friction` skill on what
  it cost.
- **Register the server:** `cargo shuyi mcp install` in the root,
  answering yes; commit the `.mcp.json` it writes (Claude Code's; the
  files of other agents it finds on the machine are not committed
  unless the repository already keeps them). `docs/engine/` or wherever
  the game's docs list `cargo shuyi` (`AGENTS.md`'s tools) gets one line:
  the server is in `.mcp.json`, its tools are `cargo shuyi`'s operations
  and the session.
- **The `engine-change` skill's step 6** (moving the pin): after `cargo
  xtask engine-pin`, reconnect the MCP server (`/mcp` in Claude Code,
  or a new session). **Either order with engine#2535:** if it is done,
  the step first runs `cargo shuyi mcp --ready`; if not, 2535's game
  follow-up adds that line (say so in 2535's ticket in the same PR if it
  doesn't).
- **The `work-ticket` skill's *Verify*** gains one line: a change a
  player sees can be checked through the session tools (`start`,
  `step`, `scene`, `frame_png`) besides `cargo shuyi frame-png`.

**Out (do not do):**
- moving the pin (1177);
- engine changes: `--ready` and the out-of-date warning are engine#2535;
- events of the game's own (a unit moved, a battle won): a later ticket
  when an agent needs them;
- a debug `command` tool (engine#2356).

## Implementation steps

1. Read engine `docs/contract.md`'s *The MCP server* and *What a game
   gives*, `crates/xtask/src/main.rs`, `crates/xtask/src/game_tools.rs`
   and `crates/ui/src/harness.rs`.
2. `shuyi.ron`'s `args`; check `cargo shuyi frame-png`, `cargo shuyi
   run --script "Enter" --json` and `cargo run -p xtask -- shuyi
   mcp-host` (which waits for a request on stdin) run.
3. `GameTools::session`, as **In** says. Expected size: about 60 to 100
   lines in `game_tools.rs`, judged against Hello, Grid's
   `examples/hello-grid/src/tools.rs` `session` and `scene` in the
   engine (about 50).
4. A test in `crates/xtask` that starts the session, plays the keys
   that reach the first battle (the playthrough tests' script), and
   reads its scene: kind `grid map`, its text the battle's
   `map_text`. And that `picture` is the size `frame` paints.
5. `cargo shuyi mcp install`, the docs line, the two skill lines.
6. By hand, in a Claude Code session in the repository: approve the
   `shuyi` server, then `start`, `step` into the battle, `scene`,
   `frame_png`. Note in the PR what each answered.

## Acceptance criteria

- [x] `shuyi.ron` routes the game's commands through `xtask shuyi`;
      `cargo shuyi frame-png` and `cargo shuyi run` still work.
- [x] `Trpg::session` gives the battle's map scene and the frame's
      picture; the test in step 4 passes.
- [x] `.mcp.json` registers `cargo shuyi mcp`, and a Claude Code
      session in the repository lists its tools and plays a battle with
      them (noted in the PR). Checked with an MCP client over stdio, not
      Claude Code's `/mcp` (Completion notes).
- [x] `engine-change` step 6 says to reconnect the server after the pin
      moves.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit or integration (in `crates/xtask`): the session reaches the first
  battle and reads its scene; the picture's size.
- The MCP protocol itself is the engine's (its stdio test,
  `crates/cli/tests/it/mcp.rs`): nothing to add here.

## Completion notes

- **Every command goes to the engine's tools:** `shuyi.ron`'s game program is
  `(package: "xtask", args: ["shuyi"])`, so the engine runs `cargo run -p
  xtask -- shuyi <command>` (the session's `mcp-host` among them). xtask's
  own commands and its list of old names are unchanged.
- **`Trpg::session`** is an engine `Headless` over the engine's Harness
  (`trpg_ui::harness::Harness::into_core`), started as `frame` starts the
  game (`frame` is now `harness` + `script.play` + `picture`, and the
  session uses the same `harness` and `picture`). `harness.rs`'s
  `map_scene`, `map_scene_in` and `map_text` became free functions over the
  engine's Harness, which the wrapper's methods and the session both call.
  The scene: on a battle, kind `grid map`, its text `map_text`; other
  screens answer "no battle on screen (the top screen is `…`)". No events
  of the game's own (out of scope).
- **Deviation, the scene's `data`:** the whole `BattleState` doesn't
  serialize to JSON (its map's `features` are keyed by `Pos`, and JSON keys
  must be strings), so `data` is the state without its map, which the
  scene's text already gives: `turn`, `phase`, `units`, `fallen`, `down`,
  `escaped`, `outcome` (`battle_data`). Not `null`, as the ticket allowed.
- **Test** (`game_tools/tests.rs`): the session's scene on the title is an
  error naming it; after the Quick Battle keys (`Down f Left f`, which
  `frame-png`'s tests use) it is a `grid map` whose text is the Harness's
  `map_text`, with units in `data`; its 800×512 picture is the same as
  `frame`'s.
- **`.mcp.json`** written by `cargo shuyi mcp install --yes`. It also wrote
  `.vscode/mcp.json` and `.gemini/settings.json`; the repository keeps
  neither folder, so they weren't committed.
- **By hand:** this session started before `.mcp.json` existed, so instead
  of Claude Code's `/mcp` I drove `cargo shuyi mcp` over stdio as an MCP
  client does: `initialize`; `tools/list` gave the 18 tools (`new`, `check`,
  `lint`, `run`, `replay`, `bots`, `playtest`, `import`, `bake`, `doctor`,
  `explain`, `start`, `step`, `scene`, `frame_png`, `events`, `reset`,
  `stop`); `start` answered ok on `title`, with the game's six start
  options; `step` with `Down f Left f` reached `battle` (through
  `preparations` and `game_flow`, with the music events); `scene` gave the
  grid map and turn 1, the player phase, the units; `frame_png` gave an
  800×512 PNG; `stop` answered ok.
- **Skills:** `engine-change` step 6 runs `cargo shuyi mcp --ready` and
  reconnects the server after `engine-pin` (engine#2535 is in the pin, so
  `--ready` exists); `work-ticket`'s *Verify* names the session tools.
  `docs/engine/README.md` says the server is in `.mcp.json`.
- Engine friction: estimate 70, measured 76 (1.1×), under 2×.
- No gameplay rules decided; nothing for Nick to check.
