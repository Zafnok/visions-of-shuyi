# Releasing

Releases are cut by pushing a tag ([ADR-0009](adr/0009-distribution.md)):

1. Bump `version` in the root `Cargo.toml`'s `[workspace.package]` and merge
   that change to `main`.
2. `git tag vX.Y.Z && git push origin vX.Y.Z` (the tag's version, without the
   leading `v`, must exactly match `Cargo.toml`, or the workflow fails).
3. The [release workflow](../.github/workflows/release.yml) builds Windows,
   Linux, macOS (universal) and web packages — each with `LICENSE`,
   `THIRD_PARTY_LICENSES.html` and a `README.txt` — and attaches them to a new
   [GitHub Release](https://github.com/Zafnok/visions-of-shuyi/releases) for the
   tag with auto-generated notes.

To try the whole pipeline without cutting a real release, run it manually via
**Actions → Release → Run workflow** on any branch: this does a dry run
(builds and uploads the four packages as workflow artifacts, but creates no
release).
