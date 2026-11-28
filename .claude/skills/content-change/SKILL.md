---
name: content-change
description: This game's rules on top of the engine pack's shuyi-private-files for its content and story documents in the private assets repository — its names (`assets-private/content/`, `story/`, the base pin `assets-private.rev`), the ticket in each name, the change's own pin file `assets-private.d/<branch>.rev` (`cargo xtask assets-pin --add`) that the game's merge lands (ADR-0135), `check-leaks` and the dialogue pause. Read shuyi-private-files first for the steps (checkout, `--no-track` branch, the private pull request, never in public), then this for the game's names and for where its pin differs from the template's §4 and §5. Use when a ticket edits, adds, renames or removes a file in `assets-private/content/`, `assets-private/story/` or the bought art, when a pin file or `assets-private.rev` conflicts or fails CI, or when a `private-land` issue is open. Not for the engine (engine-change).
metadata:
  version: "3.0"
---

# Change the private content

**Read [`shuyi-private-files`](../shuyi-private-files/SKILL.md) first.** It
holds the steps a Shuyi game with private files follows: the checkout,
a branch there made with `--no-track`, never pushing the private `main`,
pushing by the branch's full name, the private pull request, and what
never goes in public. Its pin steps (§4's `--pin`, what the pin carries,
§5) are **not** this game's: here a change adds its own pin file, and
the game's merge lands the private pull request (below, ADR-0135). This
skill holds only what is this game's (ADR-0117).

## This game's names

`cargo shuyi private-assets` reads them from `shuyi.ron`'s
`private_assets` (`checkout`, `pin_file`, `folders`), so the commands work
as written; the skill's text uses the template's names. Read them as:

| `shuyi-private-files` says | This game has |
| -------------------------- | ------------- |
| `private/` | `assets-private/` |
| `private.rev` | `assets-private.rev` |
| `private/files/`, laid over `crates/game/files/` | `assets-private/content/`, laid over `assets/` (but `content/tests/`, never embedded: ADR-0118 §3) |
| `private/docs/` | `assets-private/story/`: beats, bible, characters, outline, ledger, and `story/design/` |
| the private repository | [`Zafnok/visions-of-shuyi-assets`](https://github.com/Zafnok/visions-of-shuyi-assets) |
| `the_private_files_are_the_checkouts`, `cargo clean -p <crate>` | `bundle.rs`'s test that every layer is its folder, `cargo clean -p trpg-content` |
| the game's gates (`shuyi-gates`) | the `run-gates` skill |

The checkout also holds `game/` and `test/`, the **bought art**. Its
files need a credit (ADR-0051) and the gates never read them (ADR-0040
§5), but a change to them lands the same way: a branch, its pull
request, a pin file. The run-gates test `every_bought_file_has_a_credit`
refuses a bought file without a credit (`--add` doesn't check it). Never put a content
file in `game/`, or bought art in `content/`. The tools that read
`assets/` from disk read the content over it too (`cargo xtask
check-script`, `voice`, `cargo shuyi check-keys`, `lint`).

## The ticket in every name

- The private branch: `t<NNNN>-<slug>`, after the game ticket.
- Its commits: `[NNNN] <what changed>` (`git -C assets-private add
  content story`).
- The private pull request's title starts with `[NNNN]`: it becomes the
  title of the merge commit `private-land.yml` makes on the private `main`.
- Its body names the game ticket, says the game's merge lands it (don't
  merge by hand), and links the game's pull request by full URL
  (`https://github.com/Zafnok/visions-of-shuyi/pull/<N>`). The game's
  names `Zafnok/visions-of-shuyi-assets#<N>` and its pin file.

## Its own pin file, and the merge lands it (ADR-0135)

This game differs from `shuyi-private-files` §4 and §5 here: **a change
never edits `assets-private.rev` and never runs `private-assets --pin`**.
Several asset tickets run at once, and that one line made each wait for
and conflict with the others (ticket 0195). Instead:

1. Commit on the private branch and push it by its full name (§3, §4 of
   `shuyi-private-files`).
2. Open its pull request in the private repository, as above. Its body
   also says: "Merged by the game's merge (`private-land.yml`): don't merge
   by hand."
3. Give the change its own pin file, then commit it with the game's change:

   ```bash
   cargo xtask assets-pin --add
   git add assets-private.d
   ```

   It writes `assets-private.d/<branch>.rev`, the pushed head of the
   private branch, and refuses one that isn't pushed. **Push the private
   branch again, then run `--add` again, after every new commit there**:
   CI builds the pinned commit, and the landing refuses a branch that
   moved past it. `--check` warns when they differ.
4. Push the game's branch. CI's private-assets action checks out the
   **effective pin**: `assets-private.rev` merged with every pin file
   (`.github/actions/private-assets/resolve.sh`). Locally, `cargo xtask
   assets-pin --resolve` puts `assets-private/` there (detached), when a
   worktree needs other changes' pins as well as its own. Switch back to
   your branch before you commit there again.
5. Nick merges the game's pull request whenever it is green, in any order
   with the others. `private-land.yml` then merges the private pull
   request with a merge commit at the pinned commit, and folds the pin
   into `assets-private.rev` in a commit of its own on `main`. Nothing
   waits for another ticket, and two pin files never conflict.

**When CI's merge of the pins conflicts** (two private branches edit the
same lines), the message names the fix: merge the private `origin/main`
into your branch (`git -C assets-private fetch origin`, `git -C
assets-private merge origin/main` on the branch), resolve, commit, push,
`--add` again.

**If a landing fails**, an issue labelled `private-land` says why (no
pull request from the branch, a head that moved, a conflict). `main` still
builds with the pin file. Fix what it says, in a game pull request when
the pin file must change, and run the workflow again (Actions → Private
land → Run workflow).

**`assets-private.rev` only moves forward** (0194), and only to the
private `main`: `cargo xtask assets-pin --check` (CI's `tickets` job)
fails on a pull request that points it at a branch commit or back. Only
the fold moves it. A conflict on it means a branch still uses the old
flow: drop the branch's change to it, and `--add` instead.

**An older ticket or doc that says "`cargo shuyi private-assets --pin`
and commit `assets-private.rev`"** means these steps now: `--add` and
commit the pin file. Don't wait for the private pull request to merge,
since nobody merges it by hand. "`--pin` accepts the file" in an
acceptance criterion means the run-gates test
`every_bought_file_has_a_credit` passes.

**A pull request from before 0195** that moved `assets-private.rev` to
its branch commit: undo that change (`git checkout origin/main --
assets-private.rev`) and `--add`. If one moved it to the private `main`
after its private pull request merged, it may finish that way.

## What this game adds to "never in public"

- `cargo xtask check-leaks` (ADR-0118 §4, in `run-gates`) finds a tracked
  file that quotes a script line or a map row. A tracked file may name a
  content id, never quote a line, draw a real map or tell the plot
  (ADR-0118 §2). A test that needs plot facts keeps them in
  `content/tests/`.
- No build that embeds the content is uploaded as a workflow artifact
  or kept in the Actions cache (ADR-0117 §4): a new job that checks out
  the content gives its `rust-cache` step `cache-targets: false`, as the
  xtask test `no_job_with_private_files_caches_or_uploads_a_build` checks.
- **No bare `@word`** in either repository's titles, bodies, commits or
  comments: GitHub reads it as a mention. A dialogue directive goes in
  backticks (`` `@music` ``) or loses its `@`.

## Dialogue is paused

No agent writes or rewords a script line (Nick, 2026-11-08, until the
writer of ticket 0766): a `content/dialogue/` change follows the
`story-writing` skill for what is still allowed, and a writer's change
comes in through the `story-import` skill.
