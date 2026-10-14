# An engine and a game: feasibility report

Ticket 1101, 2026-10-14, for Nick. The technical plan is in ADR-0063 to
ADR-0067. The module-by-module audit is in [`boundary.md`](boundary.md).

## The short answer

**Yes, it's feasible.** This codebase is already shaped like an engine
with a game inside it. The rules have no idea how anything looks. The
screens describe *what* to show and skins decide *how*. A test can play
the game without a window. Any frame can be rendered to a picture. A
battle replays exactly from its seed and its moves. Those are the hard
parts of an engine, and they exist.

**Splitting it costs about 17 tickets.** That is the carve plus the
boundary gate, roughly 2–4 weeks of one session working through them one
at a time.

- About a third of the code (around 60,000 of 180,000 lines, counting
  tests) moves to the engine.
- The work is wide, not deep. Two changes touch almost every screen:
  input actions become something the game defines, and the screens'
  shared context becomes generic.
- **The risk to the game is timing.** Most of your round-3 playtest
  tickets touch the same files. So nothing moves until your Chapter 1
  playtest (0804) is done, and nothing engine-related ever goes on the
  Chapter 1 critical path.
- The carve pays back even if we never publish the engine. Input,
  screens and the test harness become smaller and cleaner, and they're
  tested on their own.

**A "serious general 2D engine" costs much more.** The other game styles,
importers for standard art tools, the AI tooling, a public repository
with docs, and networking add up to about 28 more tickets. After that
come real ongoing costs: answering users, keeping docs true, and running
servers once anything online ships.

- None of that speeds up *our* game, unless the game uses it. Free
  movement is decision 0060; multiplayer is 0053.
- After the carve, almost all of it adds new files under `engine/`, so it
  can run alongside game work without merge conflicts. It competes for
  session time, not for files.

**My recommendation:** do the cheap boundary now, do the carve right
after 0804, then publish the engine with its AI tooling. Build each new
game style or networking phase only when our game or a clear audience
needs it. Details are under *What I recommend*.

## What you asked for, and where it is

| You asked for | Where it is in the plan | Our game uses it? |
| ------------- | ----------------------- | ----------------- |
| Engine and game as two repositories, engine as an external dependency | ADR-0063: an `engine/` folder now (1102–1118), its own repository once the license and name are set and the API has settled (1130, 1131) | Yes |
| Engine free/open source, game paid | 0059 (your call: the license and name), 1130 | — |
| Top-down grid (our game) | Exists; becomes the engine's "tactics kit" (1114–1116) | Yes |
| Top-down free movement | ADR-0064; 1119–1125, with a walkable town example | Only if 0060 says so |
| Isometric, grid or free | 1126, with a small isometric tactics example | No |
| Side-scroller | 1127, with a platformer example | No |
| Moving between styles ("1 and 2 for an open world section") | ADR-0064 §1: a style is a screen, so moving between them is a screen change; 1125 shows a town that starts a grid battle and comes back | If 0060 says so |
| Asset loading: raw files, tilesets, sprite sheets | ADR-0065; Tiled and LDtk maps (1124), Aseprite and TexturePacker sheets (1123), plus today's formats | Partly (already) |
| Animations | 1123 (clips from Aseprite tags) | Already has walk cycles |
| Cutscenes | Exist (the cinematic files); the timeline moves in 1113 | Yes |
| Audio cues | Exist; move in 1104 and 1110 | Yes |
| Collision | 1122 (exact, replayable) | Only with free movement |
| Physics | 1128 (optional rigid bodies, rapier2d) | No |
| Graphical coding (like Blueprints) | ADR-0066 §5; research 1135, after the scripting runtime (1017) | No |
| Good AI "vibe coding" support | ADR-0066; the tool contract (1129), an MCP server (1133) | Yes (it speeds up our own work) |
| Shippable skills, like REA | ADR-0066 §3; 15 skills (1134) | Indirectly |
| Networking: P2P on a LAN or the internet | ADR-0067; 1140–1143 | If 0053 picks online play |
| Sessions | 1145 | " |
| Hosted multiplayer (a player's machine, or a company's) | 1146 (a "referee" server, listen or dedicated) | " |
| Matchmaking and ELO | 1148 (Glicko-2) | " |
| MMO | 1150, research only | No |
| Global chat | 1149 (chat channels and moderation tools); 1150 (what global chat really takes) | Your call, later |
| High-level and low-level access | ADR-0067 §1: every layer is public | — |
| No 3D | Out of scope everywhere | — |

## Where the code stands

The audit (`boundary.md`) sorted every module into four kinds: engine
(any 2D game would want it), tactics kit (another grid game would want
it), game, or mixed.

| Crate | Engine | Tactics kit | Game |
| ----- | ------ | ----------- | ---- |
| `core` (the rules) | 3% | 11% | 86% |
| `content` (data files and their checks) | 47% | 4% | 49% |
| `ui` (screens, frame, input, test harness) | 28% | 9% | 63% |
| `app` (window, sound, saving) | almost all | — | a little |
| `xtask` (our tools) | about half | some | the art imports |

**What's already engine-quality:**

- the deterministic rules with commands and events;
- the frame of glyphs and sprites;
- the screen stack;
- the headless Harness;
- `frame-png`;
- data formats with checks that name the file and line;
- the bundle layers that put private art over public placeholders;
- key rebinding with controller support;
- languages, dialogue with choices, voice by line id;
- audio cues with a music clock;
- timed cinematics.

**What stops a second game today** (`boundary.md`, *couplings*):

1. The list of input actions is our game's own fixed list.
2. Every screen's shared context holds our game's data.
3. The start-up builds our title screen.
4. The console is fixed at 100×32 characters.
5. The battle map's scene knows about our units.
6. The rules have no general "turn-based game" interface.
7. The asset folders are written into the code.

Each one is a ticket in the carve.

**What other styles need that doesn't exist:**

- time that steps (not only turns);
- positions finer than a tile;
- cameras that follow;
- depth sorting;
- mouse, touch and analog sticks;
- collision;
- animation clips;
- importers for map and sprite editors.

ADR-0064 and ADR-0065 plan these without giving up exact replays.

## The plan

```
engine-app    (window, drawing, sound, saving, controllers)  ◄── our game's small binary
engine-ui     (frame, screens, input, widgets, cinematics, Harness, grid map kit)  ◄── our screens
engine-assets (asset layers, images, fonts, sounds, languages, dialogue, importers)  ◄── our data
engine-core   (dice, turn and tick simulations, grid and paths, collision)  ◄── our rules
engine-cli    (frame-png, checks, new, run, replay, MCP)  ◄── our repo-only tools
```

Each of our crates sits on the engine crate of the same layer, so the
rules we already follow (ADR-0004) carry over unchanged. "engine" is a
working name until 0059.

| Phase | Tickets | When | Conflicts with game work? |
| ----- | ------- | ---- | ------------------------- |
| Boundary | 1102 | Now | No: adds a folder and a check |
| Carve | 1103–1118 | After 0804, one at a time | Yes: it touches most `ui` files. That's why it waits |
| Real time and the first new style | 1119–1125 | When you say, or when 0060 says the game walks around | No: new files in `engine/` |
| More styles | 1126–1128 | After 1125 | No |
| Going public, AI tooling | 1129–1135 | After 1118 (license after 0059) | No |
| Networking | 1140–1150 | When you say, or when 0053 picks an online mode | No |

## AI support: the part that makes this engine different

You pointed at REA. Its README and skill files show what made it work:

- It **wraps tools people already trust** and gives one set of
  operations three ways: a command line with JSON output, an MCP server
  that one command installs into many agents, and Agent Skills
  (`SKILL.md` folders saying when to use them, when not to, and the
  steps).
- **Every answer states its own limits.**

Its star count grew so fast (about 235,000 in two months) that the
number may be inflated. The pattern is sound either way.

We already have the hard part. Agents need to start a game without a
window, send input, read what's on screen as text, take screenshots,
and replay a bug exactly. That's the Harness, the scene dumps,
`frame-png` and the replays. It's a big part of why about 250 PRs landed
in this repository's first 20 days.

The research found no game engine that ships official skills. The agent
tools for Godot, Unity, Unreal and Bevy are all MCP servers, mostly
third-party. Unreal's Blueprints are binary files that agents and git
can't merge.

So the engine's pitch would be: **a 2D engine an AI agent can make,
run, see, test and replay games in, with no window and no human.**

- ADR-0066 makes it one contract (`new`, `check`, `run`, `scene`,
  `frame-png`, `replay`, `bots`, `import`, `doctor`) over a CLI, an MCP
  server and 15 skills.
- Each skill's worked example runs in CI, so the skills stay true.

**Graphical coding:**

- Godot removed its visual scripting in 4.0. About 0.5% of its users
  used it as their main language.
- Blueprints work in Unreal because of the huge framework under them.

So graphical coding here is a *view over text files*. A graph is a text
file that agents and git read, and it compiles to what the engine
already runs. It comes last, after the scripting runtime (1017) and two
examples that show which building blocks people need (1135).

## Networking, easy to hard

You asked for this only if it doesn't put our velocity at risk. It
doesn't, as long as it stays engine work scheduled on its own. Nothing
networked reaches our game before 0053, which is after launch.

| Step | What | Needs | Browser? | Tickets |
| ---- | ---- | ----- | -------- | ------- |
| 1 | Two copies of a game stay in sync by sending only moves ("lockstep"); a simulated network for tests | Nothing | — | 1140 |
| 2 | Internet play through a small relay with room codes | A $4–6 a month server | Yes (1142) | 1141, 1142 |
| 3 | LAN play, no server | Nothing | No (browsers have no UDP) | 1143 |
| 4 | Steam lobbies and Steam's relayed P2P | The Steam app (0903) | No | 1144 |
| 5 | Sessions and a ready-made lobby screen | — | Yes | 1145 |
| 6 | A "referee" server: holds the dice and the fog of war, so ranked play can't be cheated; runs on a player's machine or a server | A server per match (cheap for turns) | Yes | 1146 |
| 7 | Real-time online play (rollback for 2–4 players; snapshots for many) | Research first | The hard part (below) | 1147 |
| 8 | Accounts, matchmaking, Glicko-2 ratings, verified results | A database, a privacy policy | Yes | 1148 |
| 9 | Chat for lobbies, matches and friends, with filter, block, mute, report | Moderation | Yes | 1149 |
| 10 | MMO scale and global chat: costs, legal duties, go or no-go | Your decision | — | 1150 |

What makes this cheaper for us than for most games:

- The rules already replay from a seed and a list of moves. Online
  turn-based play is the same thing, sent over a wire.
- Saves, reconnects and desync checks come almost free.
- The same rules code runs on a server with no changes.

What the research flagged:

- **The browser is the hard part.** Our web build uses macroquad's own
  loader. Most browser networking crates need a different one
  (`wasm-bindgen`). So the browser talks through a small plugin of our
  own, the way controllers already do. Real-time play in the browser
  waits for 1147.
- **Hosting companies vanish.** In 2026 Hathora's game hosting shut down,
  Unity's Multiplay hosting ended, and PlayFab's free tier shrank.
  SpacetimeDB's license fails our rules. So everything is self-hostable
  first; vendors sit behind the same interfaces.
- **Peer-to-peer turn-based play is for friends.** Both machines hold
  everything, so a modified game can peek at the fog and the next
  rolls. Ranked play needs the referee.
- **No TrueSkill.** It's patented (until about 2029) and trademarked.
  Glicko-2 is public domain.
- **Chat with strangers is a legal commitment.** It brings EU and UK
  online-safety duties (Ofcom put in-game chat *and* matchmaking with
  strangers in scope), children's privacy rules, GDPR, and the ESRB
  "Users Interact" notice. The engine gives the tools; turning on global
  chat is your decision with eyes open (1150).
- **MMO:** an estimated 40–80+ tickets, plus hundreds to thousands of
  dollars a month to run. 1150 measures it and asks you before anything
  is built.

## What it costs the game

**Our pace:** about 250 merged PRs in 20 days. On busy days that's 20+,
across parallel sessions.

**The carve (1103–1118): about 16 tickets, serial.**

- They are bigger than an average ticket. Each moves code without
  changing anything you'd see, and every snapshot must stay identical.
- At one or two a day that's about 2–4 weeks of one lane.
- During the carve, other tickets that touch `ui` must rebase onto it.
  So run it in a lull: after 0804, not during round-3 feel work.
- Your involvement: none. It's all technical.

**Each new game style, importer, and the AI tooling: about 17 tickets.**
**Networking: 11 tickets.**

- These add new files under `engine/` and don't block game tickets.
- They spend session time that could go to the game, and they bring
  things to keep working.
- If 0060 says the army walks around freely, 1119–1125 become game work
  that pays for itself.

**After the engine has its own repository**, a change that needs both
sides costs two PRs and a pin update. ADR-0063 waits to split until
fewer than 1 in 5 game PRs touch the engine. Inside one repository the
extra cost is close to zero.

**Ongoing, once public:**

- Issues to read, docs to keep true, versions to release.
- AI-written pull requests of uneven quality. Godot's foundation banned
  them from its own repository in 2026.
- Servers, once anything online ships.

0059 asks how open you want contributions to be. My recommendation is
issues only at first.

**Your attention:** two decisions now or soon (0059, 0060), a go/no-go on
the engine-only phases, and later 0053 as already planned. Nothing else
in the engine needs you.

## Honest view of the risks

**Competition:**

- Godot (MIT) is free, mature and has an editor.
- Bevy (MIT/Apache) has the biggest Rust community but still no editor
  and slow builds.
- Fyrox reached 1.0 with an editor but is 3D-first.

A new 2D engine won't win on features. It can win on a niche nobody
covers: pure Rust, plain text everything, exact replays, and built for
coding agents. One tiny project ("Arcane") pitches the same idea and has
a handful of users. That shows the niche exists. It doesn't show it's
big.

**Engines die without maintainers.** comfy (archived 2025), Amethyst
(2022) and Moai all faded. bracket-lib, the old roguelike engine our
ADR-0003 turned down, hasn't released since 2022.

**Rust game development has regret stories.** LogLog Games left after
three years, citing slow iteration and thin tools. Hytale's engine
rewrite was cancelled, and the game shipped on its old engine.

The classic advice is "write games, not engines": pull the engine out
of a game that ships, don't build it first. **That's this plan's
order.** We pull from a working game, and each new style must ship an
example that runs in CI before the engine claims it.

**Precedents that worked:**

- Wargroove, a commercial tactics game, was built on Halley, an
  Apache-2.0 open engine.
- inkle open-sourced ink, the writing language behind 80 Days.
- Noel Berry's Celeste 64 code is MIT on top of his open Foster
  framework, while the art stays his.

**macroquad has one main maintainer** and two open security advisories
we already accept. The engine keeps it in one thin crate (`engine-app`),
so a different backend later is a contained job.

## What I recommend

1. **Now:** approve the plan (the five ADRs). I'd run **1102** (folder
   and gate) right away; it touches nothing you're waiting on. Answer
   **0059** (license, name) whenever you like.
2. **After 0804:** the carve, **1103–1118**. Our code gets cleaner either
   way, and it ends with a small example game made from engine parts
   alone.
3. **Then pick how much engine to build:**
   - **A. Stop after the carve.** The engine exists inside our repo and
     the game is cleaner. No public release.
   - **B. Carve, then publish with the AI tooling** (1129–1134). Add new
     styles only when needed: free movement if 0060 says the game walks
     around, others when there's an audience. Networking step 1 (1140)
     early, since it's cheap and proves the replays hold up. The rest
     waits for 0053. **(My recommendation.)**
   - **C. Everything in parallel.** All styles, all networking steps,
     the docs site, then visual scripting, alongside the game. Fastest
     route to "a serious engine", but it splits session time with the
     game for months, and most of it serves users we don't have yet.
4. **Networking:** steps 1–6 are small and match what you said you
   wanted for our game (hotseat, live PvP). Steps 7–8 when ranked or
   real-time play is wanted. Chat scoped to lobbies and friends. Global
   chat and MMO only after 1150 puts real costs in front of you.

## Decisions for you

- **The plan itself:** A, B or C above.
- **0059:** the engine's license (MIT OR Apache-2.0 recommended), its
  name, outside contributions, and when it goes public.
- **0060** (after 0804): does the army ever walk around freely in towns,
  camp or dungeons? Your world-structure answer said "an SRPG/TRPG not a
  traditional open world RPG". This asks whether that still holds for
  towns.
- **Later, as planned:** 0053 multiplayer, after launch.

## Sources

Gathered on 2026-10-14 by the research for this ticket.

**REA and agent tooling**

- https://github.com/morluto/rea
- https://trendshift.io/repositories/82054
- https://agentskills.io/
- https://www.remotion.dev/docs/ai/skills
- https://github.com/Coding-Solo/godot-mcp
- https://github.com/natepiano/bevy_brp/blob/main/mcp/README.md
- https://gamedev.net/news/4376-unity-ai-open-beta-how-to-get-started-with-mcp/
- https://knowledge.edge-online.com/p/ai-goes-native-in-unreal-engine-the-five-main-takeaways-from-state-of-unreal-2026
- https://www.diversion.dev/knowledge-center-articles/how-to-solve-blueprint-merge-conflicts-in-unreal-engine
- https://github.com/jsvd/arcane
- https://www.opensourceforu.com/2026/07/godot-engine-bans-vibe-coding-to-protect-maintainers-from-ai-slop/

**Engines**

- https://bevy.org/news/bevy-0-20/
- https://fyrox.rs/blog/post/fyrox-game-engine-1-0-0/
- https://github.com/not-fl3/macroquad
- https://macroquad.rs/articles/getwasm/
- https://github.com/darthdeus/comfy
- https://crates.io/api/v1/crates/bracket-lib
- https://github.com/amethyst/amethyst
- https://godotengine.org/article/godot-4-will-discontinue-visual-scripting/

**Formats, physics, audio, scripting**

- https://crates.io/api/v1/crates/tiled
- https://docs.rs/crate/tiled/latest/features
- https://ldtk.io/api/
- https://crates.io/api/v1/crates/asefile
- https://www.aseprite.org/docs/cli/
- https://rapier.rs/docs/user_guides/rust/determinism/
- https://github.com/tesselode/kira
- https://crates.io/api/v1/crates/symphonia
- https://rhai.rs/book/start/builds/wasm.html
- https://github.com/mlua-rs/mlua
- https://crates.io/api/v1/crates/egui-snarl

**Open engine, paid game; cost evidence**

- https://github.com/amzeratul/halley
- https://github.com/ExOK/Celeste64
- https://github.com/FosterFramework/Foster
- https://www.mcvuk.com/development/80-days-dev-releases-ink-scripting-language-for-free
- https://github.com/EmbarkStudios/rust-ecosystem
- https://godot.foundation/policies-and-procedures/trademark-policy
- https://geometrian.com/projects/blog/write_games_not_engines.html
- https://noelberry.ca/posts/making_games_in_2025/
- https://loglog.games/blog/leaving-rust-gamedev/
- https://hytale.com/news/2025/11/hytale-is-saved
- https://gamefromscratch.com/?p=3677

**Networking**

- https://github.com/johanhelsing/matchbox
- https://github.com/gschup/ggrs
- https://github.com/naia-lib/naia
- https://github.com/lucaspoffo/renet
- https://github.com/Noxime/steamworks-rs
- https://partner.steamgames.com/doc/features/multiplayer/networking
- https://partner.steamgames.com/doc/features/multiplayer/matchmaking
- https://gafferongames.com/post/deterministic_lockstep/
- https://gafferongames.com/post/snapshot_interpolation/
- https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond
- https://github.com/heroiclabs/nakama
- https://raw.githubusercontent.com/clockworklabs/SpacetimeDB/master/LICENSE.txt
- https://docs.edgegap.com/docs/tools-and-integrations/switch-from-hathora
- https://developer.microsoft.com/en-us/games/products/playfab/pricing/
- https://github.com/atomflunder/skillratings
- http://www.glicko.net/glicko.html
- https://patents.google.com/patent/US8538910B2/en
- https://www.ofcom.org.uk/online-safety/the-online-safety-act-and-gaming-know-the-risks-know-the-rules-know-how-to-comply
- https://blog.promise.legal/eu-digital-services-act-indie-game-studios/
- https://www.federalregister.gov/documents/2025/04/22/2025-05904/childrens-online-privacy-protection-rule
- https://crates.io/crates/rustrict
- https://wiki.projectgorgon.com/wiki/Elder_Game:_How_does_Project_Gorgon_make_money%3F

**Not verified by the research:**

- whether `steamworks-rs` links on the Windows GNU toolchain (1144
  checks);
- whether GGRS runs under our web loader (1147 checks);
- REA's true star count;
- Bevy's current build times and web sizes;
- some Unreal and Godot details known only from news coverage.
