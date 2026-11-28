# ADR-0135: A change's own pin file, landed by the game's merge

- **Status:** Accepted
- **Date:** 2026-11-27
- **Related tickets:** 0195; amends ADR-0117 (the pin) and ticket 0194's rule

## Context

A change to the private repository
([`Zafnok/visions-of-shuyi-assets`](https://github.com/Zafnok/visions-of-shuyi-assets))
was two pull requests merged in a fixed order (the `content-change` skill,
ADR-0117, 0194):

1. Nick merged the private pull request, squashed, so the private `main`
   moved.
2. The session pinned the private `main` (`cargo shuyi private-assets
   --pin`), but only once every other content commit that move carried had
   its game pull request merged. Pinning past one would build the game's
   `main` with content its code didn't expect.
3. Nick merged the game's pull request.

With several asset tickets open at once (#875, #879 and #881 on 2026-11-27),
they locked each other out. One private merge put A's content on the
private `main`, and nobody could pin `main` until A's game PR merged. Every
pin PR also edited the one line of `assets-private.rev`, so each merge left
the others conflicting, and each needed a session round trip and a new CI
run before Nick could merge it. Nick asked for "whichever approach leads to
more concurrency and less locks".

Two things caused it. The private `main` ran ahead of the game's, and every
change wrote to the same line.

## Decision

1. **The base.** `assets-private.rev` names a commit on the private `main`.
   It only moves forward (0194), and only `private-land.yml` moves it. A
   pull request that moves it to a commit off the private `main` fails
   `cargo xtask assets-pin --check`.
2. **A change's own pin file.** A change in flight adds
   `assets-private.d/<branch>.rev`, which names the head of its private
   branch (`t<NNNN>-<slug>`, no `/`). `cargo xtask assets-pin --add`
   writes it once that head is pushed. Two changes never write the same
   file, so their pull requests never conflict on the pin and merge in any
   order. The folder holds nothing else.
3. **The effective pin**, what every build and check reads, is the base
   merged with each pin file in file-name order (byte order). The base is
   the start. A pin the result already holds is skipped. A pin that comes
   after the result is taken as it is (one change made from the base is
   just its own commit). Any other pin is merged, with a fixed author and
   date (`assets-pin`, 2000-01-01), so the same pins give the same commit
   on every machine. A conflict fails and names the fix: merge the private
   `origin/main` into that branch, push, `--add` again.
   `.github/actions/private-assets/resolve.sh` does this, used by CI's
   private-assets action and by `cargo xtask assets-pin --resolve`. The
   action's `pin` output reports it as `<base> + <pin> …`.
4. **The game's merge lands the private side.** `private-land.yml` runs
   on each push to `main` that touches `assets-private.d/`. For each pin
   file, it finds the private pull request from that branch and merges it
   **with a merge commit, at exactly the pinned commit** (the API's
   `sha`). A squash would leave the tested commit on no branch, so the
   pinned commit has to stay in the private `main`'s history. Then it
   folds: `assets-private.rev` becomes the private `main`, and every pin
   file now in its history is deleted, in one commit (`[pin] Land …`)
   pushed to `main`. The fold changes no content: the private `main` then
   holds exactly what the effective pin held. A pin file it can't land
   (no pull request, a branch moved past the pin, a conflict) stays, and
   an issue labelled `private-land` says what to do. Every build keeps
   merging that file meanwhile, so `main` still builds with its content.
   A private pull request Nick merged by hand counts as landed: the fold
   moves to the private `main`, which holds it.
5. **The private `main` only holds landed changes.** Nobody merges a
   private pull request by hand any more: its body says the game's merge
   lands it. So "what the pin carries" has nothing to wait for. The
   `content-change` skill's step that waited on other tickets is gone.
6. **The token.** `PRIVATE_LAND_TOKEN` is a fine-grained token of Nick's:
   contents read and write on both repositories, pull requests read and
   write on the assets repository. It is a secret of the environment
   `private-land`, which only `main` may deploy from, so no pull request's
   run can read it. It acts as Nick, the bypass of `main`'s ruleset, so
   the fold is pushed without waiting for checks; CI then runs on the fold
   as on any push. The job runs no code from a pull request, and its
   checkout keeps no credentials (zizmor's `artipacked`). The fold step
   sets the token for its own fetch and push.
7. **The bought-art credit check.** `cargo shuyi private-assets --pin`
   refused a bought file without a credit. `--add` doesn't repeat that
   check: xtask doesn't build the engine's credits code (ADR-0129). The
   run-gates test `every_bought_file_has_a_credit` (`crates/content/tests`,
   with the private files) is the gate, as it already was for every build.
8. **The readers.** The private-assets action checks out the effective
   pin. Pin files make it fetch the history without file contents, and
   give git the key for the merge step alone, with GitHub's host keys
   from its API. `story-mirror.yml` runs on a change to either file and
   takes its `Assets-Commit` trailer from the action's `pin` output, so
   the merge's push already copies the change into the mirror and closes
   the writer's pull request. The fold's push then finds nothing new.
   `story-kit.yml` rebuilds on either and lists both in its trailer.

The engine's `private-assets` command and its `shuyi-private-files` skill
are unchanged. The command still clones and fetches the checkout at
`assets-private.rev`, which the fold keeps current. The `content-change`
skill says which of that skill's steps this game does differently.

## Consequences

- Nick merges only game pull requests, in any order and back to back.
  Each ticket's private pull request merges itself seconds later. No
  session has to come back between merges.
- Open pin pull requests no longer conflict with each other. A real
  conflict, where two private branches edit the same lines, fails CI's
  merge with the fix, as a code conflict would.
- Two changes merged close together were each tested alone, against an
  older base. `main`'s CI tests them together on the push, the same as
  for two code PRs merged without being brought up to date (ADR-0042).
- `main` gets bot commits that change only the pin. They are small, and
  CI runs on each.
- The token can push to the game's `main` and write to the assets
  repository. Only a run on `main` can read it, and anyone with write
  access to the game could already read the private files (ADR-0117).
- A private pull request squashed by hand still lands, but its branch's
  commits aren't on the private `main`. Only the effective pin of an
  older game commit can stop being fetchable then, the same as before.

## Alternatives considered

- **Keep the one pin line, and only have the merge land the private
  side.** This removes the waiting. Every open pin PR would still
  conflict after each merge and need a session round trip and a new CI
  run: less concurrency than asked.
- **A merge queue.** Nick merges each pull request himself (he holds the
  merge on purpose), and a queue doesn't fix a textual conflict on a
  shared line.
- **Pin files read by the engine's `private-assets` itself.** This is the
  general form, and every Shuyi game with private files could use it. It
  is an engine change, and the game doesn't wait on the engine. The fold
  keeps the engine's command working as it is. An engine ticket can
  follow, with Nick's OK.
- **The game's `main` naming the private `main` rather than a commit.**
  It needs no fold, but an old game commit would build with new content,
  and a release or a bisect couldn't be repeated.
