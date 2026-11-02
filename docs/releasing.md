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
