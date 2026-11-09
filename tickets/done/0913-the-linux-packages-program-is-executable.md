---
id: "0913"
title: The Linux package's program is executable
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-08
---

# 0913 — The Linux package's program is executable

## Context

0911's Release dry run ([run 42503479776](https://github.com/Zafnok/visions-of-shuyi/actions/runs/42503479776),
2026-11-08) failed `package (linux)` at "Check the package"
(`scripts/check-release-package.sh`, from 0910):

```
check: linux visions-of-shuyi-0.3.0-linux.tar.gz
check: 178 files, as expected
check: the program isn't executable
```

The check is `tar tvzf … | grep -q "^-rwx.* $folder/$bin$"`. The export
passed, disclosure gate included. The last green dry run was 2026-11-02
(#525); the engine pin moved three times since (#557, #572, #574), so the
engine's `export linux` may now write the tarball's entries without the
program's mode. A player who unpacks it would then have to `chmod +x`
the game. It may also be the check that is wrong (the `tar tvzf` line's
form): look at the tarball before deciding.

## Nick input

None.

## Scope

**In:**
- Download the dry run's `package-linux` artifact and read `tar tvzf` on
  it: the program's mode and the line's exact form.
- If the mode is wrong: the fix is in the engine's Linux export (the
  `engine-change` skill: an engine PR in `Zafnok/shuyi-engine` with a test
  that the program's entry is `0755`, then this ticket moves the pin).
- If the check is wrong: fix the `grep` in the script.

**Out (do not do):**
- The web package's file list (0912).

## Implementation steps

1. `gh run download 42503479776 -n package-linux`; `tar tvzf` it.
2. Fix where step 1 points (engine export or the script).
3. Run the Release workflow by hand on the branch; `package (linux)`
   passes. Record the run id.

## Acceptance criteria

- [ ] A Release dry run's `package (linux)` job passes (run id in the notes).
- [ ] The program's entry in the tarball has mode `0755` (the notes show
      the `tar tvzf` line).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine, if the fix is there): the Linux tarball's program entry
  is executable.
- The Release dry run.

## Completion notes

The check was wrong, not the export. The engine writes the program at
`0755` (`crates/cli/src/export/archive.rs`, `tar_gz`). The check piped
`tar tvzf` into `grep -q`, which exits at its first match. The entries are
sorted by name. Since 0911 shipped `voice/`, those entries come after
`visions-of-shuyi`, so tar was still writing when grep exited. Tar died of
SIGPIPE, and `pipefail` turned that into "isn't executable". Reproduced
locally with a 0755 program followed by 3,000 entries: the pipe fails, and
a check on the captured listing passes. The script now captures the listing
first, greps it and prints the program's line.

- Deviation: the failed run uploaded no `package-linux` artifact (the upload
  comes after the check), so step 1 read the engine's tar writer instead of
  the tarball.
- Release dry run on the branch:
  [run 42551186766](https://github.com/Zafnok/visions-of-shuyi/actions/runs/42551186766),
  `package (linux)` green; its line:
  `-rwxr-xr-x root/root  15057712 1970-01-01 00:00 visions-of-shuyi-0.3.0-linux/visions-of-shuyi`.
  `package (web)` still fails, which is 0912's job.
- No engine change, so no engine test.
- The other `| grep -q` checks in the script read `objdump -h` and
  `codesign` output, which is short enough to be written before grep exits.
  They were left as they are.

Acceptance: [x] dry run's `package (linux)` passes · [x] mode `0755` shown
above · [x] gates: no Rust changed; ticket-lint, check-mentions,
sync-agent-skills and pr-size passed locally, and CI runs the rest.
