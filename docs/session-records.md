# Session records

A debug build of the game records everything that reaches it while it runs:
each key and button, the clock, the music's position, what the system said
and the saved files as they were when the game started. A **session record**
is that, frame by frame, in a text file. Played back, it runs the same game
again to the same frame, so a bug from a tester's run (in a menu, a save, a
scene, a battle) can be seen again and kept as a test. Ticket 1210; the
engine's [ADR-0072](https://github.com/Zafnok/shuyi-engine/blob/main/docs/adr/0072-session-records-app-notes-what-the-platform-gives-a-hash-per-frame-of-what-happened-and-ron-never-migrated.md)
and its *Session records* in `docs/platform-backends.md` say how.

Only debug builds record: a `cargo run` build, or one with the `debug-tools`
feature (the Pages build). A build that ships never does, since a record
holds the player's input and saves: `crates/app/src/main.rs` refuses to
compile a release build with the engine's debug tools on.

## Where the records are

- **On a desktop**, every run is written when the game quits, to
  `sessions/` in the game's data folder (`%APPDATA%\tactical-rpg\sessions\`
  on Windows), named `<when it started>.session.ron`. The newest ten are
  kept; older ones are deleted.
- **From the debug menu** (the debug key), **Save session** writes the run
  so far at once, and recording goes on. Use it right after a bug shows:
  the record then ends near it.
- **On the web** (the Pages build), nothing is written by itself: **Save
  session** downloads the file.

A crash doesn't write one yet (the engine's ticket 3060). After a crash,
start the game again, do the same, and use **Save session** before it
crashes.

## Sending one

A tester sends the `.session.ron` file of the run with the bug, the newest
in the folder or the one **Save session** made, and says what went wrong and
about when. The file holds their saves as the game read them at the start,
and everything they pressed: send it only to the developers.

A record plays back only on the build that made it, or one that plays the
same: a later change to the rules or the screens can make it play
differently from the frame that change matters. Say which build it was (the
commit it was built from, or the Pages build's date).

## Playing one

```bash
cargo shuyi run --session <file>
```

plays it headless from the game's real start and says how far it got: every
frame, or the first frame that happened differently, with both hashes. A
frame's hash holds the open screens, the sounds and music asked for, whether
the game quit and the battle's rules state; never the picture, so a change of
look plays the same. The answer ends with the screen it stopped on, and
`cargo shuyi scene --session <file>` gives the battle's scene there. The MCP
server's `start` takes a `session` too, and `step` goes on from where it
ended. The file is read from the repository's folder.

## Keeping one as a test

1. Put the file in `crates/ui/tests/it/sessions/`, under a name that says
   what it shows.
2. In `crates/ui/tests/it/sessions.rs`, play it with
   `assert_session(title(), kept("<name>.session.ron"))`, then check what
   the bug got wrong at the end: `Core::from_session` and `play_all`, then
   the screens or the battle's state.
3. Fix the bug. The test then fails where the record plays differently, as
   it should: bless the record (below) and check that the end is now right.

When a change to the game is meant to play a kept record differently,
bless it:

```bash
SHUYI_BLESS=1 cargo test -p trpg-ui --test it sessions
```

writes each file again with the same input and this build's hashes. Bless
only after checking that the difference is the change you meant; it fails
when this build reads the platform differently from the record, and the
session must be recorded again (`write_the_quick_battle_session`, an ignored
test, does it for the Quick Battle's).
