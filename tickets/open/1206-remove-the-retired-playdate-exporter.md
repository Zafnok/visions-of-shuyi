---
id: "1206"
title: Remove the Playdate build planner replaced by shuyi export
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1170", "1203"]
nick_input: none
completed:
---

# 1206 — Remove the retired Playdate exporter

## Context

The 2026-11-01 audit split oversized 1170 into functional adoption and
bounded deletions. 1203 replaces audio conversion; 1170 makes actual
device/Simulator builds use `shuyi export playdate`, disconnecting the
old `crates/xtask/src/playdate.rs` planner. This ticket removes its roughly
1,422 source/test lines and any now-unneeded converter wrapper. No second
SDK build plan, packaging algorithm or audio converter should survive.

## Nick input

None. Build-plan and package inventory checks verify deletion. Preserve
existing SDK/GNU setup instructions; no new setup or gameplay sign-off.

## Scope

**In:**
- Delete the disconnected old compiler/linker/pdc plan, SDK/tool discovery,
  C-shim embedding, package assembly and their old implementation tests.
- Remove 1203's converter wrapper only if the engine exporter now consumes
  its game audio adapter directly; keep required game selection policy.
- Keep a small compatibility `cargo xtask playdate` forwarding command
  only if 1170 retained it, with exact supported flag/error mapping.
- Current docs and downstream build invocations use the actual engine
  export route. Preserve private-art selection and device/Simulator/bench
  behavior already established by 1170.

**Out (do not do):**
- Backend migration (1170), source cleanup (1204/1205), and new exporter
  features. No local copy of engine planning fills a supposed gap.
- Desktop/web release jobs (0910): either order works because that ticket
  owns only those targets. Playdate sale/launcher policy is 1052's; if
  that ticket lands first, preserve its policy in the engine adapter,
  otherwise update its planned command to the engine route here.

## Implementation steps

1. Verify 1170's live public build command already selects the engine
   exporter, and capture its device/Simulator/bench plan and package list.
2. Delete the retired `playdate.rs` source, unused wrappers, old tests and
   dependencies; retain game-specific config/audio/asset adapters.
3. Search all current source, manifests, workflows and build instructions
   for the former planner/shim/encoder paths and compiler command
   assembly, not just function names. Re-run after merging main.
4. Exercise supported forwarding flags if a compatibility command
   remains, and require an unsupported option to fail clearly. Compare
   package inventory/build plans with step 1, including the debug ELF's
   unwind-section handling already owned by the engine exporter.
5. Update `crates/playdate/README.md`, existing build docs and the friction
   ownership row; preserve SDK license/setup and GNU-only host guidance.

Expected size: about 1,700–1,900 changed lines including deleted tests,
docs and this ticket's move; no replacement build planner is added.

## Acceptance criteria

- [ ] Old SDK discovery/compiler/linker/pdc/package-planning code and
      algorithm tests are gone, with no renamed or relocated copy.
- [ ] Actual public Playdate build commands delegate to engine export;
      device/Simulator/bench flag behavior and private-art policy match
      1170's baseline, or an intentionally retired alias fails clearly.
- [ ] Build/package inventories remain equal to the adopted engine path;
      selected music/one-bit data, debug information handling and current
      SDK/GNU instructions remain correct.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Existing active export/build checks and package inventory comparison;
  forwarding/error tests only where a compatibility alias remains.
- No new game tests duplicating engine build-plan/codec algorithms.

## Completion notes

*(Filled in by the session that completes the ticket: removed paths,
retained adapter/alias, plan/package comparison and dependency cleanup.)*
