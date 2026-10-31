---
id: "0135"
title: A game PR stays under about 3,000 changed lines
type: infra
milestone: M0 Foundation
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-30
---

# 0135 — A game PR stays under about 3,000 changed lines

## Context

PR #483 (ticket 1007, the world map) changed about 7,250 lines, and the
session working it ran out of context even with a 1M window. Nick,
2026-10-29: "limit our game PRs to approx 3000 LOC or less". The
engine's own rule is to split a ticket past 600 changed lines; Nick
thinks that is too aggressive for the game ("3000 LOC seems about the
limit without worrying about ctx window") and wants the engine's raised
to 1,500 (an engine ticket, not this one). On how to enforce it: "idk if
we need outright fail, the engine mostly handles by estimation and
periodic checks. warning is good tho", with a nudge while working
("hey we're a bit over can we split cleanly?").

The game's `write-ticket` skill says "one PR's worth (roughly ≤ 600
changed lines excluding snapshots/data)", which nothing follows: of the
last 80 merged PRs, 30 passed 600, and two passed 3,000 once generated
files are left out (0506, the persona bots, about 5,300; 1007, about
7,250). ADR-0092's `cargo xtask code-lines` measures production Rust
for the engine-friction rule, not what a session reads and writes.

## Nick input

None. Nick set the number and asked for warnings, not a failing check.

## Scope

**In:**
- `cargo xtask pr-size [--base <ref>] [--branch <name>]`: the PR's
  changed lines (added + removed, as GitHub counts them), leaving out
  binary files and files marked `linguist-generated` in
  `.gitattributes`. It warns from 2,000 lines and again over 3,000, and
  never fails. On an `engine-pin/` or `sonar/` branch it reports the
  size with no warning.
- `.gitattributes` marks the generated text files: insta snapshots,
  `Cargo.lock`, the font atlas, the generated test tilesets, the vendored
  font sources and the engine's skill pack (`.claude/skills/shuyi-*`).
- CI: a step in the `tickets` job that runs it on a pull request and
  shows a warning annotation over 3,000. The job stays green.
- A project Claude Code hook that runs it on `git commit`, `git push`
  and `gh pr create` and tells the session when the branch is past
  2,000 or 3,000 lines. It never blocks the command.
- The rule where sessions read it before starting: `CLAUDE.md`'s hard
  rules, `work-ticket` (estimate the PR's size before coding, beside
  the ADR-0092 line estimate; split with `write-ticket` when it will
  pass about 2,000), `write-ticket` (replace the 600 with the new
  numbers and say what counts), `run-gates` (the command).
- An ADR for the rule and the check.
- Open tickets plainly bigger than one such PR get split or a note,
  with `blocked_by` checked both ways.

**Out (do not do):**
- The engine's limit (Nick wants it raised to 1,500): an engine ticket
  in `Zafnok/shuyi-engine`, written after this PR is open.
- Changing ADR-0092's `code-lines` or its thresholds.
- A failing CI check or a required status.

## Implementation steps

1. `crates/xtask/src/pr_size.rs`: `git diff --numstat -z -M` from the
   base's merge base with `HEAD` to the working tree, plus untracked
   files; skip binary (`-`) entries and paths whose `linguist-generated`
   attribute is set (`git check-attr -z --stdin`). Report the total, the
   largest files, and the warning. Wire it in `main.rs` with usage text
   and dispatcher tests.
2. `.gitattributes`: the `linguist-generated` lines.
3. `.github/workflows/ci.yml`, `tickets` job: checkout with
   `fetch-depth: 2`; on `pull_request`, `cargo xtask pr-size --base
   HEAD^1 --branch "$HEAD_REF"`.
4. `.claude/settings.json`: a `PreToolUse` hook on `Bash` that runs the
   check only for commit, push and PR-create commands.
5. ADR (`write-adr`), `CLAUDE.md` rule 10, the three skills.
6. Read the open tickets, split the plainly-too-big ones.

Expected size: about 350 production lines of Rust in xtask (comparable:
`code_lines.rs`, about 300), plus about 600 lines of docs and tickets.

## Acceptance criteria

- [x] `cargo xtask pr-size` reports the changed lines with generated and
      binary files left out (unit tests over numstat and check-attr
      output), warns past 2,000 and 3,000, exits 0 either way, and only
      reports on `engine-pin/` and `sonar/` branches.
- [x] CI's `tickets` job runs it on pull requests and stays green over
      the limit, with a warning annotation.
- [x] The hook runs on commit, push and PR create and never blocks.
- [x] `CLAUDE.md`, `work-ticket`, `write-ticket` and `run-gates` state
      the rule and the numbers; no "600" is left in the game's skills.
- [x] Too-big open tickets are split or noted; `cargo xtask ticket-lint`
      passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: numstat parsing (plain, rename, binary), the generated filter,
  the thresholds and the branch exemption, the report.
- Integration: `run` against `HEAD` in this repository.

## Completion notes

Done as scoped; the hook, CI step, ADR-0101 and rule text are in. Size: about
1,250 changed lines, about 240 of them production Rust (estimate 350).

- **Splits.** A review of every open ticket found two plainly too big. 1072
  (five side quests) is now 1072 (Tamsin, Aske), 1083 (Hollis, Rue; the Candle
  question moved here) and 1084 (Maud). 1020 (map editor) is now 1020 (units,
  battle file, Play) and 1085 (reinforcements, objective, triggers, talk
  pairs). 0512, 1064 and the ROADMAP name the new ids.
- **Notes, no split.** 1008, 0510, 0512 and 1073 got a size note saying where
  to cut if they run over. 1070 fits: it is mostly data.
- **Deviation:** added `assets-src/fonts/*.bdf` to the exclusions. 0236 and
  0239 will each commit a font source far past the limit on its own.
- **Hook tested** in fresh headless sessions (alone and chained); it can't
  show in a session already running, since hooks load at start.
- **Local gates run:** fmt, clippy, xtask's tests, check-keys, check-text,
  skill-pack, check-skills, ticket-lint, doc (xtask), typos. Left to CI: the
  other crates' tests and the WASM, no-std and Playdate builds, which this
  change doesn't touch.
- **Follow-up:** the engine's 600-line rule becomes 1,500 (Nick): an engine
  ticket, written next.
- **Nick input:** none needed; no gameplay changed.
