# Releasing

Releases are cut by pushing a tag ([ADR-0009](adr/0009-distribution.md)):

1. Bump `version` in the root `Cargo.toml`'s `[workspace.package]` and merge
   that change to `main`.
2. `git tag vX.Y.Z && git push origin vX.Y.Z` (the tag's version, without the
   leading `v`, must exactly match `Cargo.toml`, or the workflow fails).
3. The [release workflow](../.github/workflows/release.yml) runs `cargo shuyi
   export <target> --release` for Windows, Linux, macOS (universal) and web,
   checks each final package (`scripts/check-release-package.sh`), and attaches
   them to a new
   [GitHub Release](https://github.com/Zafnok/visions-of-shuyi/releases) for the
   tag with auto-generated notes.

## What a release holds

`shuyi.ron`'s `export` part names what a package holds beside what the exporter
writes (the program, `NOTICES.txt`, `HOW-TO-OPEN.txt`). A file it names that
isn't there refuses the export; only `voice` is optional. On macOS the folders
are inside the `.app`, in `Contents/Resources`.

Its `disclosure` part is the list a package is held to (ticket 1208): a file it
doesn't name, a forbidden path or a canary from the test files refuses the
export with `SH3012`, names the file and its source, and removes the package. A
new track or voice clip needs its two lines there, as on macOS it has another
path. The check finds what is written plainly: it guards against mistakes, not
against a player digging.

Beside each package the release carries, never in it:

- `<package>.debug`, Windows and Linux: the program with its line tables, which
  reads a player's crash report.
- `<package>.export-report.json` and `<package>.exposure-report.json`: the
  version, the sizes, the signing, and every file of the package with its hash.

Why Windows builds with the GNU toolchain:
[ADR-0108](adr/0108-windows-release-builds-with-the-gnu-toolchain.md).

## One package by hand

Make `dist/licenses/` first; `cargo install --locked cargo-about` gives the
tool. MSYS2's `mingw64/bin` must be on `PATH`, for `objcopy` and `windres`.

```bash
cargo about generate -m crates/app/Cargo.toml about.hbs -o dist/licenses/THIRD_PARTY_LICENSES.html
cargo shuyi export windows --release
bash scripts/check-release-package.sh windows dist/export/windows private-assets
```

Without the bought art (`assets-private/`), export with `--without
private-assets` and leave the check's last argument off. The Linux package
builds in the sniper SDK: the engine's `docs/targets/desktop.md`.

## A dry run

To try the whole pipeline without cutting a real release, run it manually via
**Actions → Release → Run workflow** on any branch: this does a dry run
(builds and uploads the four packages and what is kept beside them as workflow
artifacts, but creates no release).

## The Chapter 1 demo

The demo (ticket 0914, [ADR-0119](adr/0119-the-chapter-1-demo-is-an-edition-built-from-a-cut-bundle.md))
is the `demo` edition: Chapter 1 and nothing later, ending on "Thanks for
playing" and "Export save?" (`docs/design/demo.md`). What it keeps of
`assets/` is `crates/content/demo/bundle.txt`.

```bash
cargo shuyi export web --release --edition demo
```

The package is in `dist/export/demo/web/`. **Actions → Demo → Run workflow**
builds it with the bought art, checks it, and pushes it to the demo's itch.io
page (below). It uploads nothing to GitHub: the demo embeds the game's content
from the private repository, and a public repository's artifacts are anyone's
(ADR-0117 §4). Every pull request that changes `assets/`, `crates/content/` or `shuyi.ron`
exports it without the art, so a later chapter that reaches a file the demo
keeps fails there.

### The demo's itch.io pages

The demo has two itch.io projects of its own (ticket 0915), kept apart from the
full game's (0901), both **Restricted**, so only the writers applying (0766)
can open them:

- **The web demo** (`ITCH_DEMO_PROJECT`): played in the browser, behind one
  **password**. itch.io's download keys can't open it: a key unlocks
  downloads, and a browser-only page has none.
- **The Windows demo** (`ITCH_DEMO_DESKTOP_PROJECT`): a download, with **no
  password**, opened only by a **download key** per applicant. It is its own
  project because anyone with the web demo's password would otherwise see the
  download, and a key would no longer say whose link leaked.

Both variables are repository variables naming `<user>/<project>`; the
upload key is the secret `BUTLER_API_KEY`.

- **Push them:** **Actions → Demo → Run workflow** on `main` (leave *Push the
  demo to its itch.io project* ticked). One job per target builds the demo,
  checks it against its disclosure list, installs butler
  (`.github/actions/butler`, a pinned version checked by its SHA-256) and runs
  `butler push <package> <user>/<project>:<web|windows>`, with the game's
  version and the commit as the upload's version. A job without the secret or
  its variable builds and checks its demo, says it didn't publish, and stays
  green.
- **Once, after the first push:** on the web project's **Edit game** page,
  tick **This file will be played in the browser** for the `web` upload, set
  the embed size to 1600×1024 (or **Click to launch in fullscreen**),
  **Save**. On the Windows project's, tick **Windows** for the `windows`
  upload, **Save**.
- **Change the web demo's password:** **Edit game → Visibility & access →
  Restricted → Password**, type the new one, **Save**. The old one stops
  working at once; send the new one to the applicants still in the running.
  The game itself is served from an `html-classic.itch.zone` address in the
  page's frame, which plays without the password: someone who got in can pass
  that address on, and a new password doesn't close it. Its path holds the
  upload's build number (`/html/<upload>-<build>/`), so a new push probably
  moves it; that hasn't been checked yet.
- **A key for an applicant:** on the Windows project, **Distribute → Download
  keys → Generate**, one key, the applicant's handle (the packet's mark) as
  its label, the options as they come (*can be claimed* on, *require key to
  be claimed* and *grant external key* off: requiring a claim would make
  every applicant make an itch.io account); copy the key's link into their
  packet. **Turn one off:** the same
  list, the key's **Revoke**. A leaked link names its key, and the key's
  note names the applicant.
