---
name: write-ticket
description: Create a new ticket file in tickets/open/ using the repo template — for follow-ups found while working, bugs or feedback Nick reports after playing, or new features. Use whenever work is discovered that is outside the current ticket's scope, or when Nick reports a bug/concern. Not for working a ticket (work-ticket) or for an engine ticket (engine-change).
metadata:
  version: "1.0"
---

# Write a ticket

Tickets must be good enough that an implementation-capable model can complete them
without asking questions.

## Steps

1. **Pick the block** (see `tickets/README.md`): the hundreds digit is the
   milestone/system the work belongs to. Bugs go in the block of the system
   they're in (a combat bug → `03xx`). Post–Chapter 1 features → `10xx`+.
2. **Pick the number:** the next unused number in that block across *both*
   `tickets/open/` and `tickets/done/`:
   ```bash
   ls tickets/open tickets/done | grep -E '^03[0-9]{2}-' | sort | tail -1
   ```
3. Copy `tickets/TEMPLATE.md` to `tickets/open/NNNN-short-kebab-title.md`.
   Keep a bare `@word` out of the `title:` (it becomes the PR title, where
   GitHub reads it as a mention of a user; see `work-ticket` step 6).
4. Fill in **every** section:
   - **Context:** why this exists; link ADRs, design docs, the ticket/PR that
     discovered it.
   - **Nick input:** `None.` / *Answer first* (name the `00xx` ticket) /
     *Setup* (exact clicks) / *Decision* (a design question asked
     before building). **Never a check after merge** on a feature,
     bug, tuning or content ticket (Nick, 2026-10-21; `tickets/README.md`,
     *No checks after merge on gameplay tickets*): say how it is
     verified instead (tests, `cargo shuyi frame-png` pictures, the
     bots) and that what is left to judge by playing goes on the Act 1
     playtest's checklist (1064). `sign-off` is for playtests only.
   - **Scope — in / out:** be explicit about what NOT to do.
   - **Implementation steps:** numbered, concrete, naming files, types and
     functions. Assume the reader knows Rust but not this codebase.
     A ticket that adds a screen or a scene states its **expected size**
     there: production lines, and the comparable code it is judged by
     ("about 80 lines: one list view and its skin"). The session's
     engine-friction estimate starts from it (ADR-0092).
   - **Acceptance criteria:** checkboxes, each objectively verifiable (a test
     name, a command's output, a visible behaviour).
   - **Tests required:** which layers from ADR-0007.
   - A ticket that draws a picture or puts something on the battle map
     says how it follows ADR-0038 (sprite items; the map scene and every
     map skin), and its tests of behaviour read the scene or the state.
   - Capability/effort per `tickets/README.md` (ADR-0102); never a provider model id.
   - `blocked_by`: every ticket whose output this one needs.
   - **No ticket only to move the engine pin.** The engine watcher files
     that as an `engine-pin` issue (ADR-0096). A ticket that needs a
     newer engine says that its first step moves the pin
     (`engine-change` §7), or that it waits for the open `engine-pin` PR.
5. Keep it small: one PR's worth of work, **planned at about 2,000 changed
   lines or fewer**, so that it stays under the limit of about 3,000 when it
   runs over (ADR-0101). What counts is what `cargo xtask pr-size` counts:
   code, tests, docs, the ticket's own file (250–400 lines when it moves to
   `done/`) and hand-written data like `.dlg` scripts, `.ron` and maps. Not
   snapshots, generated or binary files. Judge it by a done ticket of the
   same kind (a chapter's script is about 1,300–1,900; a chapter's battle
   about 2,250). Split a bigger one now, by a part that works alone: the
   first piece keeps the number, so the chains that name it still hold, and
   the next pieces are `blocked_by` it. The other tickets that need a later
   piece name it too (step 6).
6. **Check the chain** (see below) before you finish.

## Check the dependency chain

A ticket is picked up when every id in `blocked_by` is in `tickets/done/`,
so a missing id means a session starts work it can't finish. For every
ticket you write or change:

- **Each open ticket the text names** is one of three things, and the ticket
  says which:
  - *needed first* (its files, types, data, decision or bought assets are
    used) → it is in `blocked_by`;
  - *either order works* → the text says what to do in each case ("if 0805
    isn't done, keep the value in `Ctx` and add a line to 0805");
  - *not this ticket's job* → it is under **Out**.
- **Look the other way too.** Search `tickets/open/` for tickets that need
  what the new one builds or decides, or that the new one makes wrong
  (`grep -rn "<id or feature name>" tickets/open docs/ROADMAP.md`), and
  update their `blocked_by` and text in the same PR.
- **Nothing waits on nothing.** If a ticket waits for a decision, a purchase
  or a setup step, a ticket for that exists and is in `blocked_by`. Use
  `status: blocked` only together with a `blocked_by` id or a written
  reason that names who unblocks it.
- **"Before X" and "after X" are dependencies.** "Do this before the Steam
  page" means X's ticket is blocked by this one; "after the playtest" means
  this one is blocked by 1064 (the Act 1 playtest; 0804 and 1050 were
  folded into it on 2026-10-21).
- If the ticket is needed for Act 1 (its chapters, its systems, the
  Playdate build Nick plays it on), add it to 1064's `blocked_by` and to
  *Act 1 critical path* in `docs/ROADMAP.md`. When unsure, add it: the
  playtest covers all of Act 1, and Nick no longer checks tickets one by
  one.
- Run `cargo xtask ticket-lint`.

## Turning Nick's playtest feedback into tickets

Nick reports bugs and concerns in plain language after playing. For each item:

- **Bug** (something is wrong): `type: bug`. Include exact repro steps (keys
  pressed, which map/turn), expected vs actual, and require a failing test that
  reproduces it before the fix. If the report is ambiguous, ask Nick *one*
  concrete question ("Did this happen after the unit was attacked, or before?").
- **Feel/balance** ("cursor feels slow", "enemies too strong"): `type: tuning`.
  Name the data values to change and propose new values.
- **Design change** ("I want magic to work differently"): write a `00xx`
  decision ticket first, then implementation tickets blocked by it.

Reply to Nick with the list of ticket numbers and one-line titles created.
