---
name: story-import
description: Take a pull request of the writer's mirror (Zafnok/visions-of-shuyi-story) that Nick approved into the game, with `cargo xtask story-import <n>`, a pull request in the private assets repository and the game's pull request that moves the pin. Use when pointed at a mirror pull request, asked to import, take in or land the writer's change, or when Nick says he approved one. Not for changing the content or story yourself (content-change) or for writing script lines (story-writing, and dialogue writing is paused).
metadata:
  version: "1.0"
---

# Take in a writer's pull request

The writer works in the **mirror**, `Zafnok/visions-of-shuyi-story`
(ADR-0122): copies of the scripts and story documents, synced from the
private assets repository at the game's pin by `story-mirror.yml`. Nobody
merges there. When Nick approves a pull request in the mirror, this skill
lands it: one pull request in the assets repository, then the game's that
moves `assets-private.rev` (the `content-change` skill), then the sync
copies it back and closes the mirror's pull request.

A mirror pull request Nick approved is its **own ticket** (`AGENTS.md`
hard rule 1): no ticket file, like a `sonar` or `engine-pin` issue.
`work-ticket` governs the rest: scope, gates, one game pull request.

The words are the writer's. Never reword a line, fix their style, or
"improve" a scene, even to pass a test: the writing pause (0766) is about
agents writing, and the writer's lines are what Nick approved.

## 1. Read it

```bash
gh pr view <n> -R Zafnok/visions-of-shuyi-story
gh pr diff <n> -R Zafnok/visions-of-shuyi-story
```

Note what it renames or removes (a scene id, a speaker) and whether it
touches the ledger, the names registry or a beat sheet.

## 2. The assets pull request

In the game's folder, on `main`, with the private checkout there
(`cargo shuyi private-assets`):

```bash
git switch -c story/<n>-<slug> origin/main
cargo xtask story-import <n> --dry-run
```

The dry run prints the change as it lands on the assets repository's
`main`. It refuses, and says why, when:

- Nick hasn't approved it (his latest review isn't an approval): stop and
  tell him;
- it changes a decision (`design/`, `story/design/`, the format in
  `dialogue/README.md`) or a file the mirror doesn't hold
  (`tools/story-mirror/files.txt`): ask the writer, on the mirror's pull
  request, to take those files out and say what they propose in a
  comment; a design proposal goes to Nick with the `ask-nick` skill;
- the writer's lines and a later change touch the same place: ask the
  writer to merge the mirror's `main` into their branch, and Nick to
  approve again.

Then run it for real:

```bash
cargo xtask story-import <n>
```

It commits with the writer as the author, pushes `story/<n>-<slug>` to the
assets repository and opens its pull request, titled
`[story#<n>] <their title>`. The checkout's own branch and files are
untouched.

## 3. The game's side

Point the pin at the new commit (`content-change` §5), so CI checks the
game with the writer's change before anything merges:

```bash
git -C assets-private fetch origin story/<n>-<slug>
git -C assets-private checkout --detach FETCH_HEAD
git -C assets-private rev-parse HEAD > assets-private.rev
```

Run `cargo xtask check-script`, then every gate (`run-gates`). What may
need the game's code or tests to follow:

- **A test that quotes a line the writer reworded.** When the test's
  subject is a mechanism (a portrait's emotion, a flag, a choice), update
  the quoted words in the same pull request. When the words were never
  its subject, point it at a test script in `assets-test/dialogue/`
  (ADR-0057), so it stops breaking on rewrites.
- **A scene the writer renamed or removed.** Follow it into the content's
  chapters and battles (`assets-private/content/chapters/`, `battles/`)
  and any code that names it, in the same assets pull request (push to
  its branch) and this pull request.
- **The ledger** (`story/ledger.md`) is the writer's: take their version.
- **Line ids and voice clips** that follow the words (`cargo xtask
  lines`, `docs/voice.md`): a clip whose line changed is stale; list them
  in the pull request for a follow-up, don't record new ones here.

Commit the game's change with the trailer the sync closes the mirror's
pull request by:

```bash
git commit -m "[story#<n>] <their title>" -m "Story-PR: <n>"
```

Open the game's pull request, titled `[story#<n>] <their title>`; its
body links the assets one (`Zafnok/visions-of-shuyi-assets#<N>`) and the
mirror's (`Zafnok/visions-of-shuyi-story#<n>`), says the assets one merges
first, and has the usual `PR size:` and `Engine friction:` lines (`none:
no code` when there is none). The assets pull request's body links the
game's back.

## 4. Merge in order

`content-change` §6–§7: when the assets pull request has merged, check
what the pin carries, `cargo shuyi private-assets --pin`, commit
`assets-private.rev`, and merge the game's pull request once CI is green.
Nick merges; don't merge it for him unless he said to.

When the game's pull request merges, `story-mirror.yml` copies the files
into the mirror and closes the writer's pull request with a note (it
reads the `Story-PR:` trailer). Check it did: `gh pr view <n> -R
Zafnok/visions-of-shuyi-story --json state`.

## Don'ts

- Don't merge, push or commit in the mirror: its `main` is the sync's.
- Don't reword the writer's lines, and don't take a change Nick hasn't
  approved.
- Don't copy the mirror's files into the public repository, or quote
  their lines in a public pull request or commit beyond what the change
  needs to be understood (ADR-0117).
- No bare `@word` in a title, body or commit: a dialogue directive goes
  in backticks.
