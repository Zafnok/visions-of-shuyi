---
name: content-change
description: Change the game's content or story documents that live in the private assets repository (its `content/` and `story/` folders) and land both repositories in order (a pull request there, then the game's pull request that moves `assets-private.rev`). Use when a ticket edits, adds, renames or removes a file in `assets-private/content/` or `assets-private/story/`, when two pull requests both move that pin, or when a game pull request's CI needs content that isn't on the private repository's `main` yet. Not for the bought art in `game/` and `test/` (ADR-0040 and ADR-0051, the work-ticket skill's credits) or for the engine (engine-change).
metadata:
  version: "1.0"
---

# Change the private content

The game's own data and scripts kept out of the public repository live in
the private assets repository,
[`Zafnok/visions-of-shuyi-assets`](https://github.com/Zafnok/visions-of-shuyi-assets)
(ADR-0117). The game builds with one of its commits, the **pin**
(`assets-private.rev`). A change there is two pull requests: the private
repository's first, then the game's, which moves the pin.

Words used here:

- **the checkout**: `assets-private/` inside the game's working folder,
  made by `cargo shuyi private-assets`. A sparse clone of the private
  repository, git-ignored by the game.
- **`content/`**: laid out like `assets/`. The build lays
  `assets-private/content/` over `assets/` whenever it is there, with
  nothing to turn on: a file there replaces `assets/`'s at its path.
- **`story/`**: the story's documents. No build reads them.
- **the bought art** (`game/`, `test/`): not this skill. It needs a credit
  (ADR-0051) and the gates never read it (ADR-0040 §5).

## 1. The checkout

In the game's working folder, before building:

```bash
cargo shuyi private-assets
```

It clones the private repository at the pin into `assets-private/` (or
fetches a checkout that is already there) with `game/`, `test/`, `voice/`,
`content/` and `story/`. Without access the game still builds on the
public files, but this skill can't be done: say so and stop.

If a build made before the checkout still reads the public files,
`bundle.rs`'s test that every layer is its folder fails and says to run
`cargo clean -p trpg-content`.

## 2. The private branch

**Never push to the private repository's `main`.** A renamed scene or a
removed file there would ride into the next session's pin move and break
its build (0153 §3.3). A ruleset on `main` asks for a pull request; Nick's
account may bypass it, and so may a session using his token: don't.

```bash
git -C assets-private fetch origin
git -C assets-private switch --no-track -c t<NNNN>-<slug> origin/main
```

`<NNNN>` is the game ticket. `--no-track` matters: a branch made from
`origin/main` that tracks it pushes to `main` on a plain `git push`.

## 3. Develop both sides

Edit `assets-private/content/` (or `story/`) and the game as usual. The
build watches the content folder, so `cargo build` and `cargo test` in the
game's folder read the edits. The tools that read `assets/` from disk read
the same view (`cargo xtask check-script`, `voice`, `cargo shuyi
check-keys`, `lint`).

Commit in the private repository with the game ticket in front:

```bash
git -C assets-private add content story
git -C assets-private commit -m "[NNNN] <what changed>"
```

Run the game's gates (the `run-gates` skill): they read the content.

## 4. The private pull request

Push the branch by its full name, never `HEAD` alone:

```bash
git -C assets-private push origin HEAD:refs/heads/t<NNNN>-<slug>
gh pr create --repo Zafnok/visions-of-shuyi-assets --base main --head t<NNNN>-<slug> --title "[NNNN] <title>" --body-file <file>
```

Run both from the game's folder. The body names the game ticket, says it
merges first, and links the game's pull request once it exists (step 5).
Its title is the commit the squash merge makes on `main`, so it starts
with the game ticket.

## 5. The game's pull request while the private one is open

So the game's CI checks the new content before either merges, point the
pin at the branch's commit by hand:

```bash
git -C assets-private rev-parse HEAD > assets-private.rev
```

Commit it with the game's change, push, open the game's pull request, and
link the two (step 8). CI checks out exactly that commit.

**A game pull request never merges with the pin on a branch commit:**
after the squash merge that commit is on no branch, and the next fetch of
it may fail. Step 7 moves the pin to `main` first.

## 6. Before moving the pin: what does it carry?

Moving the pin brings in **every** commit on the private `main` between
the old pin and the new one, not only this ticket's. Each must already
have its game pull request merged, or the game's `main` would build with
content its code doesn't expect:

```bash
git -C assets-private fetch origin
git -C assets-private log --oneline "$(cat assets-private.rev)..origin/main" -- content story
```

For each commit that isn't this ticket's, take the ticket in front of its
message (`[NNNN]`) and check its game pull request merged (the ticket is in
`tickets/done/` on the game's `main`, or `gh pr list --repo
Zafnok/visions-of-shuyi --search "[NNNN]" --state merged`). If one hasn't:

- wait for it, or ask its session (`SendMessage` if it is linked here);
- never pin past it, and never pin a commit before it that leaves this
  ticket's out.

Art commits (`game/`, `test/`, `voice/`) aren't listed: they don't break a
build (ADR-0040, the gates never read them).

## 7. When the private pull request has merged

```bash
git -C assets-private fetch origin
git -C assets-private checkout --detach origin/main
cargo shuyi private-assets --pin
```

`--pin` writes the checkout's commit into `assets-private.rev` after
checking it is on the private `main` and that every bought file has a
credit. Run step 6 again if `main` moved, commit `assets-private.rev`, push,
and merge the game's pull request once CI is green.

A change to `story/` alone moves the pin too, so every worktree that
checks out the pin has the documents.

## 8. Link the two pull requests

- The private one names the game's: its full URL
  (`https://github.com/Zafnok/visions-of-shuyi/pull/<N>`).
- The game's names the private one: `Zafnok/visions-of-shuyi-assets#<N>`,
  and says the pin moves to its merge.
- **No bare `@word`** in either title, body, commit or comment: GitHub
  reads it as a mention. A dialogue directive goes in backticks
  (`` `@music` ``) or loses its `@`.

## 9. Two pull requests move the pin

The second to merge has a conflict in `assets-private.rev`'s one line
(ADR-0040 §3). Resolve it on that branch: merge the game's `main`, then

```bash
git -C assets-private fetch origin
git -C assets-private checkout --detach origin/main
cargo shuyi private-assets --pin
```

Step 6's check again, commit, push. The new pin holds both changes.

## Don'ts

- Don't push to the private `main`, or force-push or delete a branch
  there that another pull request uses.
- Don't put a content file in `game/` (it would need a bought-art credit),
  or bought art in `content/`.
- Don't copy a file of `content/` or `story/` into the public repository,
  or quote one in a public ticket, pull request or commit beyond what the
  change needs to be understood.
- Don't upload a build that embeds the content as a workflow artifact:
  anyone can download a public repository's artifacts (ADR-0117 §4).
