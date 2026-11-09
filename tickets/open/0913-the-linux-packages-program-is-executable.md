---
id: "0913"
title: The Linux package's program is executable
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
