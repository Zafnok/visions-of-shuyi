---
name: engine-change
description: Make a Shuyi Engine change from a game ticket and land both repositories in order (engine PR, then the game PR that moves the pin), or hand it to the engine as a ticket and ship the game with a marked workaround. Use when a game ticket needs the engine changed, when the engine-friction skill says "engine feature" or "engine ergonomics", when writing a game ticket blocked by an engine ticket (`engine#NNNN`), or when starting engine work in a session of its own. Not for judging whether game code is too big (engine-friction) or for moving the pin alone (engine-pin).
metadata:
  version: "1.0"
---

# Change the engine from the game

Since ticket 1131 the engine's code lives in its own repository,
[`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine). The game
builds on one engine commit, the **pin** (`shuyi-engine.rev`; how it
works: `docs/engine/README.md`). An engine change is never a reason to
wait: do it now, or ship the game with a marked workaround and let the
engine catch up. This skill says which, and how.

Words used here:

- **the checkout**: `shuyi-engine/` inside the game's working folder,
  made by `cargo xtask engine`. It is a full clone of the engine's
  repository, git-ignored by the game.
- **the patch**: `.cargo/engine-patch.toml`, written by the same command.
  While it is there, the game builds on the checkout instead of the pin.
- **engine#NNNN**: an engine ticket, in a game ticket's `blocked_by` or
  text. The engine names game tickets the other way round: `game#NNNN`.

## 1. Is it an engine change at all?

Both tests must pass. If either fails, it is a game change: stop here and
do it in the game (a game-side structure, `engine-friction` outcome 3).

1. **What goes where** (ADR-0063 §5): *would a platformer or a farming
   game want this unchanged?* → engine. *Would another grid-tactics
   game?* → the tactics kit, also in the engine's repository. Otherwise →
   game. When unsure, game.
2. **The leak test** (ADR-0092, the `engine-friction` skill step 5): the
   engine change names no class, unit, faction, Combat Art, chapter,
   character, story beat, or id from the game's data. If a generic shape
   sits under a game-shaped one, the engine gets the generic shape and
   the game wraps it ("dialogue box" is game; "a text panel with a
   portrait slot and paged text" is engine).

Write the change in one sentence in the generic shape before going on.
You need that sentence for the engine ticket.

## 2. Now, or a ticket and a follow-up?

**Do it now, in this session** when the engine change is one PR's worth
or less (the engine's `write-ticket` states the number) and is
no larger than the game ticket itself. Go to step 3.

**Do it in a session of its own** (step 6) when the engine change is
larger than the game ticket, or touches engine files that another open
engine PR touches:

```bash
gh pr list --repo Zafnok/shuyi-engine --state open
gh pr view <N> --repo Zafnok/shuyi-engine --json files --jq '.files[].path'
```

**Ticket and follow-up** when it is more than one PR's worth. Then, in
this order:

1. **Write the engine ticket** in the engine's repository: the checkout
   (step 3's first two bullets) or any clone of it. Follow its
   `write-ticket` skill (`shuyi-engine/.claude/skills/write-ticket/SKILL.md`;
   a harness may not discover skills in another checkout automatically, so
   read the file). Its section "A request from the game" says what the
   ticket holds; `game#NNNN` names this game ticket. Commit it on a
   branch of its own and open a PR for it alone (title `Ticket NNNN: <title>`)
   so it reaches the engine's `main`.
2. **Ship this game ticket with the workaround.** Mark each workaround
   line or block `// engine-friction: engine#NNNN` (the `engine-friction`
   skill step 6).
3. **Write the game follow-up ticket** (`write-ticket`) that deletes the
   workaround and moves the pin: `blocked_by: ["engine#NNNN"]`. Check it
   with `cargo xtask ticket-lint --check-engine`.
4. Add the row to `docs/engine/friction.md` if the `engine-friction`
   skill sent you here.

## 3. Doing it now

1. **The checkout.** In the game's working folder:

   ```bash
   cargo xtask engine
   ```

   It clones the engine at the pin into `shuyi-engine/` (or fetches a
   checkout that is already there) and turns the patch on. The clone
   starts at the pin, not at the engine's `main`.

2. **The engine branch.** The engine's rules are its own: read
   `shuyi-engine/AGENTS.md` and `shuyi-engine/.claude/skills/work-ticket/SKILL.md`
   (read them as files). Then:

   ```bash
   git -C shuyi-engine switch main
   git -C shuyi-engine pull --ff-only
   git -C shuyi-engine switch -c t<NNNN>-<slug>
   ```

   `<NNNN>` is the engine ticket. If the change has no engine ticket yet,
   write it first on this branch (its `write-ticket`, the section "A
   request from the game"), and the same PR moves it to `done/`.
   A change of a few lines with nothing to decide (a doc fix, a missing
   re-export) needs no engine ticket: it uses this game ticket, on a
   branch `game-<game NNNN>-<slug>` and a PR titled `Game <game NNNN>:
   <what it does>` (the same section of the engine's `write-ticket`).
   If the engine's `main` is past the pin, the game may need small fixes
   to build on it: they belong to this game PR.

3. **Develop both sides against the patch.** Edit the engine in
   `shuyi-engine/` and the game as usual; `cargo build` and `cargo test`
   in the game's folder use the checkout. Commit the engine side with
   `git -C shuyi-engine commit` in the engine's style
   (`[NNNN] imperative summary`).

4. **The engine's gates**, inside the checkout: the engine's `run-gates`
   skill (`shuyi-engine/.claude/skills/run-gates/SKILL.md`). Run each in
   a subshell so the session stays in the game's folder, and so Cargo
   reads the engine's `.cargo/` settings, not the game's:

   ```bash
   (cd shuyi-engine && cargo test --workspace --locked --no-fail-fast)
   ```

   They must pass before the push. A change to docs only needs the
   ones that read docs (`typos`, and the links check once the engine has
   it).

5. **The engine PR.**

   ```bash
   git -C shuyi-engine push -u origin t<NNNN>-<slug>
   gh pr create --repo Zafnok/shuyi-engine --base main --head t<NNNN>-<slug> --title "[NNNN] <title>" --body-file <file>
   ```

   Run both from the game's folder: don't `cd` into `shuyi-engine/`
   (the session's working folder would move with it). The body follows the engine's `work-ticket` §6 and step 5 below: it
   names the game ticket (`game#NNNN`) and says the engine PR merges
   first. It is **squash-merged**, the only merge the engine's repository
   allows: one commit on the engine's `main` per PR, and step 6 pins
   that commit, never the branch's.

6. **When the engine PR has merged**, move the pin to the engine's
   `main`, not to the branch commit:

   ```bash
   git -C shuyi-engine switch main
   git -C shuyi-engine pull --ff-only
   cargo xtask engine --off
   cargo xtask engine-pin
   ```

   `engine --off` removes the patch and writes `Cargo.lock` for the pin
   again; `engine-pin` with no commit pins the commit the checkout is at
   and moves `shuyi-engine.rev`, `Cargo.toml` and `Cargo.lock` together.
   Run the game's gates (`run-gates`) on the new pin, then commit those
   three files with the game change.

   The move passes every engine PR between the old pin and the new one,
   not only yours: follow the `engine-pin` skill's review and ledger
   steps (its steps 4 to 7) for them, and write the move's section in
   `docs/engine/pins.md` (ADR-0096). `cargo xtask engine-pin --report`
   before moving lists them; `cargo xtask engine-pin --check` fails
   until the section is there.

   Run `cargo shuyi mcp --ready`: it builds the server (`.mcp.json`)
   and the game's tool program on the new pin, so the next call doesn't
   wait for a compile. A running server follows the pin by itself: one
   whose engine moved under it hands its calls to a fresh one
   (engine#2546).

7. **The game PR** goes up after the pin moves. Before that it can't
   pass CI: CI never has the patch, so the game builds on the old pin,
   and `cargo xtask engine-pin --check` fails on a `Cargo.lock` that
   names the checkout. Never merge a game PR built only against the
   patch. If you open it early, open it as a draft.

## 4. Breaking changes

- **Before the engine's first crates.io release** (engine ticket 2005):
  a breaking change is fine when the same session moves the game's pin
  and fixes the game in the game PR. Say in the engine PR what breaks
  and that the game PR follows it.
- **After it**: follow the engine's `CHANGELOG.md` and version rule (its
  `work-ticket` §3, *Public API*). The game still moves its pin in its
  own PR.

## 5. Link the two PRs

- The engine PR's body names the game ticket and the game PR when it
  exists: `Zafnok/visions-of-shuyi#N`.
- The game PR's body names the engine PR: `Zafnok/shuyi-engine#N`, and
  says it moves the pin to that PR's merge.
- Edit the engine PR's body to add the game PR once that exists
  (`gh pr edit <N> --repo Zafnok/shuyi-engine --body-file <file>`).
- The engine ticket's Completion notes name the game PR that moves the
  pin (the engine's `work-ticket`).
- **No bare `@word`** in either PR, commit or comment (`AGENTS.md`): put
  it in backticks or drop the `@`.

## 6. Engine work in a session of its own

Start it when step 2 says so: the engine change is larger than the game
ticket, or it touches engine files another open engine PR touches (the
two would fight over the same lines). In the desktop app, propose it
with `start_session`:

- **Where:** the engine's own folder when the machine has one (check
  `list_start_targets`; on Nick's machine it is `D:\shuyi-engine`).
  Otherwise in a fresh game worktree, where it runs `cargo xtask engine`
  and works in `shuyi-engine/`.
- **The brief:** the engine ticket's id, or the one-sentence change from
  step 1 and the facts for "A request from the game" if there is no
  ticket yet; the game ticket it is for (`game#NNNN`); "don't move the
  game's pin: the game's follow-up ticket does".

This game session does not wait for it. It carries on with the marked
workaround and the follow-up ticket of step 2, exactly as for "ticket
and follow-up".

## 7. A game ticket blocked by an engine ticket

`work-ticket` §1 has the check. A game ticket with `engine#NNNN` in
`blocked_by` is ready only when the engine ticket is in the engine's
`tickets/done/` **and** the game's pin is at or past the commit that
closed it. When it is done but the pin is behind, moving the pin is the
first step of that game ticket (step 3.6, starting at
`cargo xtask engine-pin <commit of the engine's main>`, with the
`engine-pin` skill's review and ledger section for the engine PRs it
passes). When an `engine-pin` PR is already open, the ticket may wait for
it to merge instead (`work-ticket` §1). Don't write a ticket only to
move the pin: the engine watcher files that as an `engine-pin` issue
(ADR-0096).

## Don'ts

- Don't put a game concept in the engine to make step 1 come out
  "engine".
- Don't pin a branch commit or an unmerged commit; pin the engine's
  `main` after the merge.
- Don't commit the game with the patch on (`cargo xtask engine --off`
  first), and don't commit `shuyi-engine/` (it is git-ignored).
- Don't make two engine PRs that touch the same files at the same time.
- Don't let the engine block the game ticket: do it now, or ship the
  marked workaround and follow up.
