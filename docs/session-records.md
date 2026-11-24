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
feature (the playtest page's web build). A build that ships never does, since a record
holds the player's input and saves: `crates/app/src/main.rs` refuses to
compile a release build with the engine's debug tools on.

## Where the records are

- **On a desktop**, every run is written when the game quits, to
  `sessions/` in the game's data folder (`Documents\My Games\Visions of Shuyi\sessions\`
  on Windows), named `<when it started>.session.ron`. The newest ten are
  kept; older ones are deleted.
- **From the debug menu** (the debug key), **Save session** writes the run
  so far at once, and recording goes on. Use it right after a bug shows:
  the record then ends near it.
- **On the web** (the playtest page's build), nothing is written by itself: **Save
  session** downloads the file.

A crash doesn't write one yet (the engine's ticket 3060). After a crash,
start the game again, do the same, and use **Save session** before it
crashes; or send its crash report (below), which every build writes.

## Crash reports

Every build, a shipped one too, writes a **crash report** when the game
panics (ticket 1245, the engine's ADR-0079 and `docs/crash-reports.md`):
the panic's message and place, the frame and screen, the log's last lines,
the engine's and the game's versions and, in a battle, the battle's record
(its first state, the commands since and the one being applied; nothing the
player typed). The newest five are kept, beside the saves:

- **Windows:** `Documents\My Games\Visions of Shuyi\crash-<when>.ron` (`%APPDATA%\tactical-rpg` before ticket 0875, moved on the first launch);
- **macOS:** `~/Library/Application Support/tactical-rpg/crash-<when>.ron`;
- **Linux and the Steam Deck:** `~/.local/share/tactical-rpg/crash-<when>.ron`;
- **the web:** the browser's `localStorage`, key `tactical-rpg/crash-<when>`
  (the newest only, and no battle record).

The next start says once, under the title's menu, that the game closed
unexpectedly and a report was saved (`docs/design/crash-reports.md`).
Nothing is sent anywhere: a player reporting a crash sends the newest
`crash-*.ron` and `logs/latest.log` (below). Its battle record is the
report's `record` field; copied to a file of its own, it replays to the
crash:

```bash
cargo shuyi replay <file>
```

## The log file

Every desktop build, a shipped one too, writes its **log** to
`logs/latest.log`, in the folder the crash reports are in (ticket 1345;
the engine's *The log file* in `docs/targets/desktop.md` and *Logs* in
`docs/platform-backends.md`):

- **Windows:** `Documents\My Games\Visions of Shuyi\logs\`;
- **macOS:** `~/Library/Application Support/tactical-rpg/logs/`;
- **Linux and the Steam Deck:** `~/.local/share/tactical-rpg/logs/`
  (`$XDG_DATA_HOME/tactical-rpg/logs/` when that is set).

Each start moves the last three runs to `previous-1.log`, `previous-2.log`
and `previous-3.log` (oldest last), so the folder keeps four runs. The
first line names the engine's version, the system, debug or release, the
storage's name and the game's version. Each line after it reads

```text
    3.214 f182 WARN  storage: settings.ron couldn't be read (…); using the defaults
```

the seconds since the start, the frame (`f0` before the first), the level,
what wrote it (`storage`, `audio`, `game`, …) and the text. A panic is
logged with its message, `file:line` and thread, whether or not a crash
report was written. A run writes 2 MiB at most, then errors alone.

A build that ships writes warnings and errors alone. A debug build, or
one with the debug tools, writes `info` and up, and more
when the `SHUYI_LOG` variable says so, e.g. from a terminal:

```powershell
$env:SHUYI_LOG = "debug"; cargo run -p trpg-app
```

`SHUYI_LOG=warn,audio=debug` keeps warnings and errors and the audio's
lines down to `debug`; the categories and the rest of the filter are in
the engine's *Logs*. **On the web** there is no file: the same lines go to
the browser's console (its developer tools), which a tester copies or saves.

A log holds file paths, and a path may hold the player's account name:
like a crash report, it is sent only to the developers, and the game sends
nothing by itself.

## Sending one

A tester sends the `.session.ron` file of the run with the bug, the newest
in the folder or the one **Save session** made, with `logs/latest.log`
([The log file](#the-log-file); `previous-1.log` too if they started the
game again since), and says what went wrong and about when. The file holds their saves as the game read them at the start,
and everything they pressed: send it only to the developers.

A record plays back only on the build that made it, or one that plays the
same: a later change to the rules or the screens can make it play
differently from the frame that change matters. Say which build it was (the
commit it was built from, or the playtest page's upload date).

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
