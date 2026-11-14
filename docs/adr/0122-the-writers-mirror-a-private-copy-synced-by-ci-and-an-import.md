# ADR-0122: The writer's mirror: a private copy synced by CI, and an import

- **Status:** Accepted
- **Date:** 2026-11-13
- **Related tickets:** 0152; 0153 (the research, §3.4), 0159 and 0160
  (the story and the content move), 0764–0766 (the writer's programs, the
  guide, the hire)

## Context

Nick is hiring a human writer (0766). The writer needs the scripts and the
story documents and a way to propose changes, and nothing else: not the
code's history of the story, not the bought art, not the game's other
content.

Since 0159 and 0160 those files are in the private assets repository
(`Zafnok/visions-of-shuyi-assets`, ADR-0117): the scripts in
`content/dialogue/`, the story documents in `story/`. The game builds with
one of its commits, the pin (`assets-private.rev`).

GitHub can't give a collaborator one folder. A collaborator reads a whole
repository, and on a personal account's repository every collaborator can
write all of it (no Read or Triage role outside an organization). The
assets repository holds the bought art, whose licence forbids passing it
on (ADR-0040), so the writer can never join it.

## Decision

1. **A third private repository, `Zafnok/visions-of-shuyi-story`, is a
   mirror, not a source.** It holds copies of the files a writer works on
   and nothing else. The real files stay where they are; the writer is a
   collaborator there and nowhere else. It holds no secret, so write
   access reaches only those copies.
2. **One list says what it holds:** `tools/story-mirror/files.txt`, each
   mirror path with where it comes from in the game's checkout (the
   assets repository at the pin, or a public file) and whether a writer's
   change there is taken back. Today: the scripts and `story/` (taken
   back, but `story/design/`), and read-only copies of the format, three
   public design notes, a start page and the mirror's own check. A change
   to a read-only file is a proposal for Nick. The sync and the import
   both read the list.
3. **The game's CI writes the mirror's `main`, and nothing else does.**
   `story-mirror.yml` runs on a push to the game's `main` that moves the
   pin or changes a listed public file. It checks the assets repository
   out at the pin with `PRIVATE_ASSETS_KEY` (content only, as the gates
   do), replaces the mirror's files with the list's, and commits on top of
   the mirror's `main`, never a force push, with `Game-Commit:` and
   `Assets-Commit:` trailers. A ruleset on the mirror lets only Nick's
   account update `main`; the token is his, fine-grained, on the mirror
   alone (`STORY_MIRROR_TOKEN`), and only `story-mirror.yml` and
   `story-kit.yml` name it.
4. **The import is a three-way apply of the writer's diff onto the assets
   repository's `main`.** The mirror's files are byte copies, so the blobs
   of the commit a pull request branched from are the assets repository's
   at the pin that commit names. `cargo xtask story-import <n>` refuses a
   pull request Nick's latest review didn't approve or one that changes a
   file not taken back; fetches the mirror into the assets checkout; maps
   the paths; and runs `git apply --3way --cached` on a scratch index
   read from `origin/main`, so a file that moved on since still takes the
   writer's lines, a real conflict stops it with the files named, and the
   checkout itself is never touched. The commit's author is the writer
   (GitHub's noreply address), with a `Story-PR: <n>` trailer. It then
   pushes a branch and opens the assets pull request; the game's pull
   request that moves the pin follows (`content-change`), and carries the
   same trailer, by which the sync closes the mirror's pull request once
   it is in the game. The `story-import` skill holds the steps; a mirror
   pull request Nick approved is its own ticket (AGENTS.md hard rule 1).
5. **The writer's checker is a program built from the game's crates.**
   The check logic moves from xtask into `crates/writer` (`trpg-writer`):
   `cargo xtask check-script` and the writer's `story-check` run the same
   code and print the same problems. `story-check` validates the scripts
   against the chapters and battles that name their scenes, so it embeds
   the game's content: `story-kit.yml` builds it with the content checked
   out and uploads it straight to the mirror's release `kit` with the
   crates' licence notices, never as a workflow artifact (ADR-0117 §4).
   The mirror's own CI downloads it with its read-only token and marks
   each problem on its line; it runs no code of the game's and has no
   secret.

## Consequences

- A writer gets exactly the files in the list, a checker, and pull
  requests; Nick approves on GitHub, a session lands it. Nobody merges in
  the mirror, and a stale mirror pull request is closed by the sync.
- Every other ticket keeps editing the real files as before
  (`content-change`); the mirror follows the pin by itself.
- `story-check` carries the game's content inside it. The writer could
  dig the data tables and maps out of the program; they are a hired
  collaborator with the whole story already, so this is accepted. It never
  goes anywhere public.
- The token expires: the sync's error says how to renew it (0152's setup).
- A writer's change that conflicts with a later one goes back to the
  writer (merge the mirror's `main`, Nick approves again), never resolved
  by an agent rewording lines.
- 0764 adds the preview program to `trpg-writer` or `trpg-app` and to
  `story-kit.yml`; 0765 writes the mirror's start page.

## Alternatives considered

- **The writer in the assets repository** — the bought art's licence
  forbids it, and they could write all of it.
- **The scripts moved into the writer's repository as the source** (0153
  §3.4) — a second deploy key, the engine's `private-assets` reading two
  repositories, and Nick merging writer pull requests before any game
  check sees them.
- **The pin of ADR-0040 for a story repository** — every build, test and
  worktree would need a second checkout and key, for files only one person
  edits.
- **`gh pr diff` and `git apply` in the checkout** — GitHub's diff has
  short blob ids and the checkout's working tree would change; fetching
  the mirror's commits gives full ids, and a scratch index keeps the
  checkout as it is.
- **The mirror's CI building the checker from source** — it would need
  the game's code and content there, which is what the mirror exists not
  to hold.
