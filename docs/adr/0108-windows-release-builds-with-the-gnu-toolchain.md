# ADR-0108: The Windows release builds with the GNU toolchain

- **Status:** Accepted
- **Date:** 2026-11-01
- **Related tickets:** 0910, 0902, 0903
- **Amends:** ADR-0009's table: the Windows row's `x86_64-pc-windows-msvc`. Its
  other rows stand.

## Context

ADR-0009 built the Windows package with MSVC on GitHub's runners. Ticket 0910
moves the packages to `cargo shuyi export`, whose Windows exporter builds
`x86_64-pc-windows-gnu` with MSYS2's mingw-w64 GCC. In exchange it gives the
stripped program, the kept debug file and the remapped paths, which the game
would otherwise do by hand beside the engine's. `crates/app/build.rs` embedded
the exe's icon and version info on MSVC only, because Nick's machine was thought
to have no `windres`; MSYS2's binutils give it.

## Decision

1. The Windows release builds with the GNU target, as the engine's exporter does.
2. `build.rs` embeds the icon and version info on both Windows toolchains.
3. `scripts/check-release-package.sh` refuses a Windows package whose program has
   no `.rsrc` section or imports a MinGW runtime DLL, since `build.rs` only warns
   when the resource compiler fails.

## Consequences

- Nick's local builds and the release now use one toolchain, so a local export
  shows what a player gets.
- The MSVC build is gone from the release; CI's other jobs are unchanged.
- Steam (0903) must check that the `steamworks` crate links on the GNU target
  before relying on this; if it doesn't, that ticket asks the engine for a choice
  of Windows toolchain.

## Alternatives considered

- **Keep MSVC by hand in `release.yml`:** the parallel packaging 0910 removes.
- **An engine option for the Windows toolchain now:** nothing needs it yet.
