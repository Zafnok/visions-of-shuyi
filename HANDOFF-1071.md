# Handoff: ticket 1071, paused 2026-11-09

**Delete this file before opening the PR.** It is in the WIP commit only so
the next session can resume.

- Worktree: `D:\tactical-rpg\.claude\worktrees\act-1s-eight-fixed-skirmishes-766580`
- Branch: `claude/act-1s-eight-fixed-skirmishes-766580` (local only, **nothing
  pushed**). The last commit is a WIP commit: keep it (or squash it into the
  final commit). Commit messages: `[1071] ...`.
- Workflow: the `work-ticket` skill, from its step 4 (Verify).
- Local cargo: put MSYS2 on PATH (`export PATH="/c/msys64/mingw64/bin:$PATH"`),
  and run one cargo command at a time in this worktree.

## The split (done)

The eight skirmishes plus the knock-on work were well over 2,000 lines
(ADR-0101), so the ticket was split before coding:

- **1071 (this PR):** Chapters 4 and 5's four (Aldwater Toll Bridge, Millhaven
  Road with Gil, the Salt Road, Smugglers' Cove), and guards becoming part of
  the army a chapter arrives with.
- **1090 (new, `tickets/open/1090-...`):** Chapters 6 and 7's four.
- Already updated for the split: 1064's and 0512's `blocked_by` (+1090),
  `docs/ROADMAP.md`, notes added to 0852 and 0853, the "Split" paragraph in
  1071's Context.
- **Still to do:** 1071's `title:` still says "eight". Rename it to
  something like "Act 1's fixed skirmishes of Chapters 4 and 5, with Gil's
  recruitment" and use that as the PR title.

## What is built

- **Maps** (`assets/maps/`): `aldwater`, `millhaven_road`, `salt_road`,
  `smugglers_cove` (18 x 12).
- **Battles** (`assets/battles/`), same ids:
  - The guards (`aldwater`, `salt_road`) are Easy, Rout, and have **no
    `bots_army`**: the bots field the real arrival army.
  - The optional ones are Normal, with `bots_army`: ch04 lv 6 for Millhaven
    Road, ch05 lv 8 for the cove.
  - All four are `kind: FixedSkirmish` with `music: Pool("skirmish")`.
- **Millhaven Road:** `DefeatUnit(patrol_leader, leftovers: Flee)`. Gil (the
  `prizefighter` character) is an enemy, with `UnitFell(recruit: true)` →
  `ch04_prizefighter_fell` and `CombatStart` → `ch04_prizefighter_engage`, and
  two battle notes.
- **New character:** `patrol_leader`, display name **"Corporal"** (a
  one-word title, like "Officer"; a two-word name fails the names test). It
  is in `characters.ron`, `names.ron` and `docs/story/names.md`.
- **The cove's loot:** 800 gold, an Elixir and two Potions. The smugglers
  reuse the `privateer_*` templates.
- **`assets/worldmaps/act1.ron`:**
  - The four places are `FixedSkirmish` nodes with their scenes.
  - Saltmere and the Envoy's House open on `Cleared("salt_road")` again.
  - The Chapter 6 and 7 comments now point at 1090.
- **The presence check:** `Cast::with_places` (`crates/content/src/dialogue/presence.rs`)
  plus `worldmap::place_scenes()`. A place's intro scene can count on its
  battle's named enemies; that is what let Gil speak in
  `ch04_millhaven_road_intro`. It is wired in `crates/content/src/lib.rs`
  `check_presence`. Its test is
  `a_places_intro_counts_on_its_battles_named_enemies`.
- **The bots** (`crates/bots/src/arrival.rs`):
  - `guards_by_chapter` is a pure route plan: the guards each chapter must win
    on the way to its story battle.
  - `arrival` plays those guards before each story battle.
  - A guard battle gets the arrival army of its own chapter, so
    `place_in_story` and `chapters_before` cover guards.
  - `PLAYTHROUGHS` includes the `aldwater` and `salt_road` fixtures.
  - Test: `the_guards_on_the_way_are_won_first`.
- **UI tests:**
  - `ch04_winnable.rs`: an Aldwater step in `chapter_4()`,
    `win_on_the_way()`, `record_aldwater`, and the in-game travel stops at
    the bridge.
  - `ch05_winnable.rs`: the same for the Salt Road.
  - New `skirmishes.rs`: the place kinds, the guard blocking, Gil's replay
    and joining, the cove's replay and loot, and Gil from the map back to the
    map in game.
  - `playthrough.rs`: `record_where`, a "wanted win" condition, so Gil is
    beaten before the Corporal.
  - The `side_quests.rs` helpers are now `pub`.
  - `world_map.rs`: the open list at the Act 1 start, plus the
    `act1_start` snapshot, updated.
- **Fixtures** (`crates/ui/tests/it/fixtures/`): `aldwater_`,
  `millhaven_road_`, `salt_road_` and `smugglers_cove_winning.ron` are new;
  `ch04_winning.ron` and `ch05_winning.ron` were re-recorded.
- **Docs:** `docs/playtesting.md` (*The army*) and `assets/battles/README.md`.

## Bot results (100 tries, seeds 1–100; Casual / Normal / Hardcore success)

| Battle | Army | Result | Band |
| --- | --- | --- | --- |
| aldwater (Easy skirmish) | after ch03 | 100 / 89 / 90 | ✓ (autobalance: Swordsman at 6,4 +2; all Hounds lv 9, he 11) |
| millhaven_road (Normal) | ref lv 6 ch04 | 93 / 56 / 66 | ✓ (ring Hounds −4 to lv 1, one more swordsman) |
| salt_road (Easy) | after ch04 | 99 / 82 / 89 | ✓ (earlier autobalance: Hounds lv 11–12, guard at the stone removed) |
| smugglers_cove (Normal) | ref lv 8 ch05 | 98 / 67 / 77 | ✓ (privateers lv 12–13) |
| **ch04** (story) | after ch03 + Aldwater | **59** / 58 / 51 | Casual ⚠ too easy (band 30–55); was 52/61/63 |
| **ch05** (story) | after ch04 + Salt Road | **77 / 85 / 13** | all out (40–70, 50–75, 50–75); was 58/58/69 |

Full reports for the PR are in this session's scratchpad, which may be gone
after a reboot. Re-run them with `target/release/xtask playtest <id> --bot all`
after `cargo build --release -p xtask`.

## Next steps

1. **Decide about Chapters 4 and 5's bands.**
   - Winning the guards' fights first made both easier, and Chapter 5's
     Hardcore fell to 13%. Look at why: its success needs at most one item
     used, and Normal's median is 3 items.
   - The ticket lists the story battles as Out, but this is caused by this
     change. Either retune here (`cargo shuyi autobalance --stage ch05`, or
     by hand as `docs/playtesting.md` describes for 0851/0526), or file a
     tuning ticket (write-ticket) and say so in the PR.
   - Any change to ch04 or ch05 means: re-record `ch04_winnable::record`, then
     `ch05_winnable::record_salt_road` and `ch05_winnable::record` (the Salt
     Road's army comes from Chapter 4's playthrough). Then re-run the bots'
     arrival tests.
   - CI's playtest fails only below 20% Casual, so this blocks nothing in CI.
2. **Run the full-game tests on their own** (not yet confirmed to pass):
   `cargo test -p trpg-ui --test it -- ch04_winnable ch05_winnable skirmishes`
   (debug build). Use debug: in a `--release` build,
   `world_map::a_key_rebound_to_find_opens_it_on_the_map` fails because the
   title menu has other rows. That isn't from this change.
3. **`frame-png` pictures** of the four maps into `docs/img/1071-*.png`, for the PR.
4. **The `run-gates` skill**, every gate.
5. **Size:**
   - `cargo xtask code-lines`, against the estimate of 50 production lines.
     It will be over: the presence `with_places`, `place_scenes` and the
     arrival rewrite are about 150. At 3× or more, run the `engine-friction`
     skill. It is likely game-side structure, not engine.
   - `cargo xtask pr-size`, against the estimate of 1,500.
6. **Completion notes:** what was done, the split, and the deviations:
   - Gil's art is 0862's, not imported here.
   - Gil fights the party only; there are no three-sided fights.
   - Smugglers use the privateer templates.
   - The presence-check addition, and the guards using the arrival army.
   - The bot tables above.
   - *The agent's starting values*:
     - Tiers: guards Easy, optional Normal.
     - Rewards: Aldwater 400, Millhaven 300, Salt Road 500, the cove 800 +
       Elixir + 2 Potions.
     - The "Corporal" name, and the levels.
   - What's left to judge in the 1064 playtest: whether Millhaven Road's level
     marker (Lv 2, because the ring is lv 1) misleads, and how the guard
     fights feel.

   Then set `status: done` and `completed:`, `git mv` the ticket to `done/`,
   delete this file, commit, push, and open the PR (no bare `@` words). Watch
   CI once.
