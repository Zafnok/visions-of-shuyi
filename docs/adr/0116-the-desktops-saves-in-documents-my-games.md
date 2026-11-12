# ADR-0116: The desktop's saves live in Documents\My Games\Visions of Shuyi

- **Status:** Accepted (0875); amends ADR-0039 and ADR-0069 (where the storage is kept on Windows)
- **Date:** 2026-11-11
- **Related tickets:** 0875, 0872, 0903; engine 3310

## Context

The desktop kept every storage key in `%APPDATA%\tactical-rpg\` (ADR-0039,
ADR-0069), the engine's data folder for the storage name `tactical-rpg`.
Nick, answering 0872: the saves should never default to AppData; Documents
or My Games is more accessible. The folder is decided by the engine
(`shuyi_app::AppConfig`), so the engine got the choice first (engine
ticket 3310, engine ADR-0108).

## Decision

1. **Windows:** `<Documents>\My Games\Visions of Shuyi`
   (`config.desktop_folder = DesktopFolder::MyGames("Visions of Shuyi")`
   in `crates/app/src/main.rs`). Documents is the system's known folder,
   so a Documents moved to OneDrive is followed. `My Games` over plain
   Documents: it is where PC games keep saves, and it keeps Documents tidy.
   The folder is named after the game, not the storage name: players see
   it. It is a literal, never the window's title, so a rename of the game
   never moves the saves again.
2. **macOS and Linux** keep the system's data folder (`Application
   Support/tactical-rpg`, `$XDG_DATA_HOME/tactical-rpg`): neither has a
   My Games convention. The web keeps `localStorage`.
3. **The move** is the engine's (engine ADR-0108): on the first launch
   whose new folder holds no key, every file of `%APPDATA%\tactical-rpg\`
   (saves, backups, settings, crash reports, `exports\`, `sessions\`)
   is copied there, never over a file, and the old files removed once all
   are copied. A failure keeps the old folder for that launch and logs
   why. A new folder that holds keys is used as it is.
4. The storage name `tactical-rpg` doesn't change: it is the web's
   `localStorage` prefix and the folder the saves move out of.

## Consequences

- The Profile and Load Game place lines (0872) show the new folder: the
  engine's `storage_place` fact comes from the same folder.
- Steam Cloud (0903) syncs `Documents\My Games\Visions of Shuyi`.
- A player with saves in both folders (a copy made by hand) keeps the new
  folder's; the old one is left as it was.

## Alternatives considered

- **`Documents\Visions of Shuyi`** — Nick named both; My Games is the
  common place for PC games' saves.
- **Keep AppData** — what Nick asked to change.
