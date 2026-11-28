---
id: "0195"
title: Private-content PRs merge in any order, each adds its own pin file and the merge lands the private PR
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: setup
completed: 2026-11-27
---

# 0195 — Private-content PRs merge in any order, each adds its own pin file and the merge lands the private PR

## Context

A ticket that changes the private repository
([`Zafnok/visions-of-shuyi-assets`](https://github.com/Zafnok/visions-of-shuyi-assets))
lands as two pull requests in a fixed order (the `content-change` skill,
ADR-0117, ticket 0194):

1. Nick merges the private PR (squash), so the private `main` moves.
2. The session moves `assets-private.rev` to the private `main` with
   `cargo shuyi private-assets --pin`. This is allowed only once every
   other content commit that move carries has its game PR merged ("What
   the pin carries": *never pin past it*).
3. Nick merges the game PR.

With several asset tickets open at once (on 2026-11-27: #875, #879, #881
and private #46, #47), they lock each other out. The first private merge
puts A's content on the private `main`, and from then on nobody can pin
`main` until A's game PR merges. Every pin PR also edits the same one line
of `assets-private.rev`, so each merge leaves the others conflicting.
Each one needs a session round trip and a new CI run before Nick can merge
it. Nick (2026-11-27) asked for "whichever approach leads to more
concurrency and less locks".

The fix has two parts:

- **The game's merge lands the private PR.** A game PR pins its own
  private branch commit and is merged that way. A workflow on the push to
  the game's `main` then merges the matching private PR with a merge
  commit, so the pinned commit stays in the private `main`'s history. The
  private `main` then only ever holds content whose game PR has merged, so
  "what the pin carries" has nothing to wait for.
- **One pin file per change.** A PR never edits `assets-private.rev`. It
  adds `assets-private.d/<private-branch>.rev`, a path no other PR
  touches, so pin PRs never conflict with each other. CI builds with the
  base pin merged with every pin file. After landing, the same workflow
  folds the landed files into `assets-private.rev` and deletes them, in
  one commit to `main`.

Nick merges only game PRs, in any order, with no session work in between.
What still needs a person is a real conflict between two private branches
(both edit the same lines of one file). CI's merge reports it, and the
workflow stops on it. The engine's `private-assets` tool isn't changed:
it keeps reading `assets-private.rev`, which the fold keeps current.

## Nick input

**Setup** (once, when this PR is ready; the PR body repeats these steps):

1. GitHub → Settings → Developer settings → Personal access tokens →
   Fine-grained tokens → Generate new token. Resource owner `Zafnok`.
   Repository access: *Only select repositories*:
   `visions-of-shuyi-assets` and `visions-of-shuyi`. Permissions:
   **Contents: Read and write** (both repositories), **Pull requests: Read
   and write**. Expiry: one year. Copy the token.
2. `visions-of-shuyi` → Settings → Environments → New environment
   `private-land` → Deployment branches and tags: *Selected branches* →
   add `main`. Under Environment secrets add `PRIVATE_LAND_TOKEN` = the
   token.

The token acts as Nick, who is the bypass actor on `main`'s ruleset, so
the fold commit can be pushed without waiting for checks. CI still runs
on it. Because the secret lives in an environment that only `main` may
use, no pull request's workflow can read it.

## Scope

**In:**
- Pin files `assets-private.d/<branch>.rev`, written by `cargo xtask
  assets-pin --add`.
- The effective pin: the base merged with every pin file, from `cargo
  xtask assets-pin --resolve`, used by CI's private-assets action and by
  every workflow that reads the pin.
- `assets-pin --check` for the new rules.
- `.github/workflows/private-land.yml`: lands the private PRs and folds
  the pin files.
- ADR-0135. The `content-change`, `story-import`, `work-ticket` and
  `run-gates` skills, `AGENTS.md`, and the tools' messages that say
  `private-assets --pin`.

**Out (do not do):**
- Changing the engine's `private-assets` or its `shuyi-private-files`
  skill (engine repository). A generic version of this for every Shuyi
  game is an engine ticket, filed only with Nick's OK.
- Auto-merging game PRs. Nick merges every game PR himself.
- Changing how Nick merges game PRs.
- 0172 (the bots play on a content change). Either order works: if 0172
  is done first, its `detect` job diffs the old effective pin against the
  new one (`--resolve` at each side) instead of the two
  `assets-private.rev` lines; if this lands first, 0172 does the same (a
  line is added there).

## Implementation steps

1. **ADR-0135** (`write-adr`): "A change's own pin file, landed by the
   game's merge". It amends ADR-0117 and the rule of ticket 0194:
   `assets-private.rev` is the base, moved only by `private-land.yml`;
   ticket PRs add `assets-private.d/*.rev`; the effective pin is the base
   merged with those, in file-name order. It also covers the merge commit
   in the private repository: a squash would orphan the pinned commit.
   Record the token's reach (write to both repositories, used only from
   the `private-land` environment on `main`).
2. **`crates/xtask/src/assets_pin.rs`** (extend; keep `judge` and the
   `History` trait for the testable core):
   - `--add`: writes `git -C assets-private rev-parse HEAD` to
     `assets-private.d/<current private branch>.rev`. It refuses a
     detached HEAD, uncommitted changes, a branch not pushed to `origin`
     at that commit, and `main`. It runs the same bought-art credit check
     `private-assets --pin` runs (`cargo shuyi docs credit`; call the
     engine's function if it is public, otherwise reproduce the rule in
     xtask and say so in the ADR).
   - `--resolve [--checkout]`: prints the effective commit. With no pin
     files it prints the base. With one pin file that is a descendant of
     the base, it prints that commit (no merge, so a shallow checkout is
     enough). Otherwise it merges the pin files into the base in
     file-name order in a temporary worktree of `assets-private/`, with a
     fixed author and date so the tree is reproducible, and prints the
     merge's commit. On a merge conflict it fails, naming both branches
     and the command that fixes it: merge `origin/main` into your branch,
     push, and `--add` again. `--checkout` also puts `assets-private/` at
     that commit (detached), for local gates.
   - `--check` (CI's `tickets` job): fail when a pull request changes
     `assets-private.rev` ("add a pin file with `cargo xtask assets-pin
     --add`"). On a push to `main`, keep today's moves-forward rule for
     the base. For each pin file: the name is a branch on the private
     `origin` (`git ls-remote`), its content is that branch's head (warn,
     don't fail, when the head is newer: the session must `--add` again),
     the commit is in the private history, and it isn't already an
     ancestor of the base (a leftover after a fold: fail, "delete it").
3. **`.github/actions/private-assets/action.yml`**: when
   `assets-private.d/` has files, fetch what `--resolve` needs. Use the
   shallow checkout for the single-descendant case, and full history
   (`fetch-depth: 0`, with blobs) only when a merge is needed. Then run
   `cargo xtask assets-pin --resolve --checkout`, or the same logic in
   bash if building xtask there is too slow (measure; the `tickets` job
   already builds it). Print the effective commit. Every caller keeps its
   inputs.
4. **Other readers of the pin**: `story-mirror.yml` (its `Assets-Commit`
   trailer and the copy), `release.yml`, `demo.yml` and
   `story-kit.yml` read the effective commit, never the raw
   `assets-private.rev` line. `grep -rn "assets-private.rev" .github
   crates` lists them. A merge push that still holds pin files must not
   close the writer's mirror PR before its files are in. Either resolve
   the effective pin, or skip and let the fold's push do it. Say which in
   the ADR.
5. **`.github/workflows/private-land.yml`**: `on: push: branches: [main],
   paths: ["assets-private.d/**"]` plus `workflow_dispatch`;
   `environment: private-land`; `concurrency: {group: private-land,
   cancel-in-progress: false}`; `permissions: contents: read`. Check out
   with `token: ${{ secrets.PRIVATE_LAND_TOKEN }}` and the private
   repository with the same token. For each pin file in name order:
   - find the private PR whose head branch is the file's stem (`gh pr
     list --repo Zafnok/visions-of-shuyi-assets --head <stem> --state
     all`);
   - already merged, or the pinned commit already an ancestor of the
     private `main`: landed;
   - open: `gh api -X PUT repos/Zafnok/visions-of-shuyi-assets/pulls/<n>/merge
     -f merge_method=merge -f sha=<pinned commit>`. The `sha` refuses a
     head that moved past the tested commit;
   - no PR, a moved head, or a conflict: stop. Leave that file and the
     ones after it, fail the run, and open or update one issue labelled
     `private-land` with the branch, the PR and the fix command. No `@`
     mentions.

   Then fold: fetch the private `main`, write its commit to
   `assets-private.rev`, `git rm` every pin file now in its history, and
   commit with a fixed bot identity and the message `[pin] Land <branches>`.
   Push to `main`, retrying (fetch, rebase, redo the fold) when `main`
   moved.
6. **Skills and docs** (edit the shared source under `.claude/skills/`,
   then `pwsh -File scripts/sync-agent-skills.ps1`):
   - `content-change`: replace "The game's CI reads the new content
     before either merges", "What the pin carries" and "The pin only
     moves forward" with the new flow. Push the private branch, open its
     PR (body: "merged by the game's merge, don't merge by hand"), `cargo
     xtask assets-pin --add`, commit the pin file, push. On a CI merge
     conflict, merge `origin/main` into the private branch, push, `--add`
     again. Never run `private-assets --pin` or edit `assets-private.rev`.
     Say how to override the engine skill's §4 and §5.
   - `story-import`, `work-ticket` (the credit step), `run-gates` (the
     `assets-pin` line: `--check`, and `--resolve --checkout` before local
     gates in a worktree whose branch has pin files from other merges),
     `AGENTS.md`'s bought-art bullet, and every message in
     `crates/tools/src/*_import.rs` and `title_art.rs` that says
     `private-assets --pin`.
7. **Open PRs at the switch** (#875, #879, #881, or whatever is open
   then): the PR body says they may finish the old way or convert: undo
   their `assets-private.rev` change and `--add` instead.
8. Tick 0172's note (add the line there if 0172 is still open).

Expected size: about 1,300 changed lines. Roughly 350 of production code
in `assets_pin.rs`, 300 of its tests, 150 of workflow and action, 200 of
ADR and skills, and the ticket. Judge it by 0194 (the forward check, about
400 lines).

## Acceptance criteria

- [x] Two PRs that each add a pin file never conflict, and either can
      merge first. Shown in the PR by a test of `--resolve` on a
      throwaway repository with two sibling branches.
- [x] `cargo xtask assets-pin --resolve` prints the base with no pin
      files, the pinned commit for one descendant, and a merge commit
      with the same tree on every run for two siblings. It fails with
      the fix command on a conflict.
- [x] `cargo xtask assets-pin --check` fails on a PR that edits
      `assets-private.rev` and on a leftover pin file. It warns on a pin
      file behind its branch's head.
- [ ] `private-land.yml` merged a real private PR (waits for the setup; see the notes) with a merge commit and
      folded its file (the first ticket to use it is enough). Its
      `workflow_dispatch` run on a `main` with no pin files does nothing.
- [x] No workflow reads the raw `assets-private.rev` line where it needs
      the content (`grep` in the PR).
- [ ] `PRIVATE_LAND_TOKEN` is readable only (waits for the setup) from the `private-land`
      environment (`gh api repos/Zafnok/visions-of-shuyi/environments/private-land`).
- [x] `content-change` no longer has a "wait for another ticket" step.
- [x] `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass: locally fmt, workspace clippy, the xtask and trpg-tools tests, check-leaks, typos, xtask docs, ticket-lint, check-mentions, zizmor and actionlint; the rest (the whole workspace's tests, the WASM and Playdate builds, deny, machete) in CI, since no code they build changed.

## Tests required

- Unit: `judge` and the new pin-file rules over a fake `History`.
- Integration: `--resolve` and `--add` on temporary git repositories
  (`tempfile`): no pin files, one descendant, two siblings, a conflict, a
  leftover, a moved head.
- CI: the workflow's dry run (`workflow_dispatch` with no pin files) in
  the PR.

## Completion notes

Done in the same PR that filed it (Nick, 2026-11-27: "knock it out now").

- **ADR-0135** records the flow. It amends ADR-0117 and 0194's rule.
- **`cargo xtask assets-pin`** (`crates/xtask/src/assets_pin.rs`):
  - `--add` writes `assets-private.d/<branch>.rev`, the pushed head of the
    private branch. It refuses a detached checkout, uncommitted changes,
    `main`, a branch name with `/`, and a head that isn't pushed.
  - `--resolve` puts `assets-private/` at the effective pin.
  - `--check` now fails on a base pin off the private `main`, and checks
    each pin file: its branch exists, it isn't a leftover the base
    already holds, and it warns when the branch moved past it.
- **`.github/actions/private-assets/resolve.sh`** merges the pins. CI's
  action runs it in a step of its own that holds the key, with history
  fetched without file contents. xtask runs the same script, through
  Git's own bash on Windows.
- **`.github/workflows/private-land.yml`** lands each pin file's private
  pull request with a merge commit at the pinned commit (the API's
  `sha`), then folds the pin into `assets-private.rev` in one commit on
  `main`. When it can't land one, it files or updates a `private-land`
  issue. zizmor and actionlint pass on it.
- **The readers.** `story-mirror.yml` runs on a pin file too and takes its
  trailer from the action's new `pin` output. `story-kit.yml` rebuilds on
  pin files and lists them in its trailer.
- **Docs.** The `content-change`, `story-import`, `story-writing`,
  `work-ticket` and `run-gates` skills, `AGENTS.md`, `docs/voice.md` and
  the importers' messages say `--add` in place of `private-assets --pin`.

Deviations:

- `--resolve` always checks the result out. It has no `--checkout` flag:
  the merge has to happen in the checkout anyway.
- `--add` doesn't run the bought-art credit check. xtask doesn't build the
  engine's credits code (ADR-0129), and the run-gates test
  `every_bought_file_has_a_credit` already refuses an uncredited file.
- A pull request may still move `assets-private.rev` **forward to the
  private `main`**, so the old-flow PRs open today can finish that way.
  What fails is a base off `main`, which is what a branch pin in the old
  flow is.
- `story-import`'s assets branch is now `story-<n>-<slug>`, not
  `story/<n>-<slug>`: a pin file is named after its branch, so the name
  can't hold a `/`.
- `private-land.yml` goes on to the other pin files after one fails,
  rather than stopping. They don't depend on each other, and the failed
  one stays merged into every build.
- The open tickets whose steps say `private-assets --pin` (0460, 0491,
  0735, 0862, 0866, 0878, 0881, 1069, 1222, 1280, 1380–1382, 1386–1388)
  weren't edited. `content-change` has a paragraph that maps the old
  words to `--add`, and `work-ticket` sends every content change there.

Not verified yet:

- `private-land.yml`'s first real landing and its `workflow_dispatch` dry
  run need `PRIVATE_LAND_TOKEN`, which is Nick's setup above. The first
  ticket that adds a pin file after the setup shows it working.
- The environment check (`gh api …/environments/private-land`) waits for
  the same setup.

Follow-up, not filed: a generic form for every Shuyi game, where the
engine's `private-assets` reads the pin files itself. It would be an
engine ticket, and needs Nick's OK.

Engine friction: estimate 350 production lines, measured 278 (0.8×, bash
and YAML aren't counted): under 2×. No engine workaround. PR size:
estimate 1,300, measured about 1,730.
