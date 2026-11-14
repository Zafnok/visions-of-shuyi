# Releasing

Releases are cut by pushing a tag ([ADR-0009](adr/0009-distribution.md)):

1. Bump `version` in the root `Cargo.toml`'s `[workspace.package]` and merge
   that change to `main`.
2. `git tag vX.Y.Z && git push origin vX.Y.Z` (the tag's version, without the
   leading `v`, must exactly match `Cargo.toml`, or the workflow fails).
3. The [release workflow](../.github/workflows/release.yml) runs `cargo shuyi
   export <target> --release` for Windows, Linux, macOS (universal) and web,
   with the bought art and the game's content, checks each final package
   (`scripts/check-release-package.sh`), and makes a
   [GitHub Release](https://github.com/Zafnok/visions-of-shuyi/releases) for the
   tag with auto-generated notes and each package's reports. The packages
   themselves are not attached (below).

## Where each build goes

The game's content and story are private (ADR-0117, ticket 0158), and so is
the bought art's source (ADR-0040). Anything public on GitHub, a Pages site,
a release or a workflow's artifacts, can be downloaded by anyone, so a build
with the content goes straight from its job to a restricted itch.io page:

| Build | Where | Who can open it | Workflow |
| ----- | ----- | --------------- | -------- |
| Public web build, debug tools, **no content** (the public files and the test game) | [GitHub Pages](https://zafnok.github.io/visions-of-shuyi/) | anyone | `pages.yml`, each push to `main` |
| Full web build: content, bought art, debug tools | the playtest page, itch.io `ITCH_PLAYTEST_PROJECT`, channel `web` | Nick, with the page's password | `playtest-itch.yml`, each push to `main` and by hand |
| The Chapter 1 demo (web and Windows) | the demo's two itch.io pages (below) | the writers applying | `demo.yml`, by hand |
| A release's packages | nowhere yet: built and checked, then left in their job; the store's itch.io page (ticket 0901) will take them | — | `release.yml`, on a tag |
| A release's debug files (Windows, Linux) | a release `debug-v<version>` of the private assets repository (below) | whoever can read that repository | `release.yml`, on a tag |

The demo never goes to Pages.

### The playtest page

`ITCH_PLAYTEST_PROJECT` is a repository variable naming `<user>/<project>`
of an itch.io HTML project, **Restricted** with **Also allow a password to
view page** (a password of its own, not the demo's). Each push to `main`
builds the full game for the web (`cargo shuyi web --release --debug-tools
--private-assets`, the content checked out) and runs `butler push dist/web
<user>/<project>:web`; pushes in a row upload only the last. Without the
secret `BUTLER_API_KEY` or the variable, the job builds, says it didn't
publish, and stays green. Once, after the first push: on the project's
**Edit game** page, tick **This file will be played in the browser** for the
`web` upload, **Save**. The password and the `html-classic.itch.zone`
address behave as the web demo's (below).

## What a release holds

`shuyi.ron`'s `export` part names what a package holds beside what the exporter
writes (the program, `NOTICES.txt`, `HOW-TO-OPEN.txt`). A file it names that
isn't there refuses the export; only `voice` is optional. On macOS the folders
are inside the `.app`, in `Contents/Resources`.

The desktops' and the web's programs read `assets/` (with the content over it)
from `trpg-app.pack` beside them, which the export bakes with the game's tools
and checks against the files (ADR-0120): a package without it shows "bake it
again" in place of the game. `cargo shuyi bake --target desktop --out <file>`
bakes one by hand, and `--check` proves it. The bought art stays in the
program. A new file of `assets/` or of the content needs its line in the
`disclosure` parts' `allowed_pack_entries` (the demo's too if it keeps it); an
xtask test names it.

Its `disclosure` part is the list a package is held to (ticket 1208): a file it
doesn't name, a forbidden path or a canary from the test files refuses the
export with `SH3012`, names the file and its source, and removes the package. A
new track or voice clip needs its two lines there, as on macOS it has another
path. The check finds what is written plainly: it guards against mistakes, not
against a player digging.

The GitHub Release carries, never in a package:

- `<package>.export-report.json` and `<package>.exposure-report.json`: the
  version, the sizes, the signing, and every file of the package with its hash.
  The job checks that only these leave it and that neither holds a line of
  dialogue.

The export also writes `<package>.debug` (Windows and Linux), which reads a
player's crash report. It is the whole program before stripping, content
included, so it goes to the private assets repository instead (below).

## The debug files

A real release puts each `.debug` straight from its package job on the
release `debug-v<version>` of the private assets repository,
[`Zafnok/visions-of-shuyi-assets`](https://github.com/Zafnok/visions-of-shuyi-assets/releases),
named `visions-of-shuyi-<version>-<windows-x64|linux-x64>.debug`
(`scripts/upload-debug-file.sh`; ticket 0917). The first job there makes the
release; a rerun replaces the file. A dry run builds and checks the debug
file and uploads nothing.

**Fetch one for a crash report** (anyone who can read the private
repository, with `gh` signed in):

```bash
gh release download debug-v0.2.0 -R Zafnok/visions-of-shuyi-assets -p '*windows-x64.debug' -D dist/debug
```

It holds the program's symbols and line tables at the same places as the
shipped program (`scripts/check-release-package.sh` checks it), so a
code address in the shipped program reads as a function, file and line:
`addr2line -f -C -e <file>.debug <address>` (MSYS2's binutils on Windows).
**Not yet usable on a player's report:** the engine writes a crash
report's `backtrace` in the standard library's short form, which in a
stripped build is `<unknown>` frames with no addresses, and the report
doesn't say where the program was loaded. Until it does (follow-up of
ticket 0918), a report's `location` (file and line of the panic, which
the release keeps) is what reads it.

**Why there.** The file is as private as the content it embeds (ADR-0117
§4), so it can't be on this repository's release or an artifact. Compared:

| Place | Cost | Who can read | Secret |
| ----- | ---- | ------------ | ------ |
| A release of the private assets repository (**chosen**) | free (a release asset up to 2 GiB) | the private repository's collaborators; `gh release download` fetches one by version | a new one, `DEBUG_FILES_TOKEN` |
| A restricted itch.io project of its own | free | Nick, from the project's page | none new (`BUTLER_API_KEY`) |

itch.io keeps one build per channel and butler has no documented command
to download one, so each version would need a channel of its own and a
crash report a visit to the page in a browser. The private repository
already holds what the file embeds, keeps one release per version, and an
agent reading a crash report fetches the file with `gh`.

`DEBUG_FILES_TOKEN` is a secret of this repository: a fine-grained token
of Zafnok's (made 2026-11-12, no expiry) with **Contents: Read and write**
on `Zafnok/visions-of-shuyi-assets` only. Workflows of this repository can
use it, never a fork's pull request; only `release.yml` does. A real release
without it fails at **Keep the debug file private**, before the GitHub
Release. **Make a new one** (if it's revoked or leaks): github.com →
Settings → Developer settings → Personal access tokens → Fine-grained
tokens → Generate new token, Repository access *Only select repositories*,
`visions-of-shuyi-assets`, Permissions *Contents: Read and write*; then this
repository's Settings → Secrets and variables → Actions → `DEBUG_FILES_TOKEN`
→ Update.

Why Windows builds with the GNU toolchain:
[ADR-0108](adr/0108-windows-release-builds-with-the-gnu-toolchain.md).

## One package by hand

Make `dist/licenses/` first; `cargo install --locked cargo-about` gives the
tool. MSYS2's `mingw64/bin` must be on `PATH`, for `objcopy` and `windres`.
A release `web` package (the game's or the demo's) needs Binaryen's
`wasm-opt` on `PATH` too, and fails with `SH1003` without it (engine
ADR-0094): `pacman -S mingw-w64-x86_64-binaryen` in MSYS2, or a release
from [github.com/WebAssembly/binaryen](https://github.com/WebAssembly/binaryen/releases).
CI installs the version pinned in `.github/actions/binaryen/action.yml`.

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
(builds and checks the four packages and uploads only their reports as
workflow artifacts, `kept-<target>`; it creates no release).

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
  the embed size to 1280×720 and tick **Fullscreen button** (Nick's
  setting, 2026-11-12), **Save**. On the Windows project's, tick **Windows** for the `windows`
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
