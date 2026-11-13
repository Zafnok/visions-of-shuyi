# The README's pictures

How each picture the top-level `README.md` shows is made, so it can be made
again after the game, the art or the title changes. All are screenshots of
the running game, drawn without a window by `frame-png` with the bought art
(`CLAUDE.md`, *Bought art*: commit them only as screenshots of the game).
First check the private repository out:

```bash
cargo shuyi private-assets
```

## The intro animation, `intro.gif`

One command, with no hand step (ticket 1264):

```bash
cargo run -q -p xtask --features private-assets -- readme-pictures
```

It draws the title's intro cinematic at 10 frames a second from 56 s to
74.5 s of the title song (the party sets off, the brigand comes down to
the lead, the combat scene) and from 114.7 s to 117.7 s (a moment of the
whole field, then the logo, from the cut at 115.8 s, and the menu land),
in four `frame-png --every 0.1 --count N --gif` clips (`frame-png` draws
at most 64 frames a run, and one run can't skip from 74.5 s to 114.7 s).
It joins them into one GIF that loops, without the bottom 16 rows (the
help bar and the debug key's hint, which `frame-png` can't leave out),
each frame stored as the part it changes, the menu held 3 s before the
loop. The colors are exact. The clips and their moments
are in `crates/xtask/src/readme_pictures.rs`. Picking other moments means
changing that file's `CLIPS`; the cinematic's cuts are in
`assets-private/content/cinematics/title.ron`.

## The other pictures

From ticket 0125's notes, 1600×1024 (`frame-png`'s scale 2), each made by
one command written to `docs/img/<name>.png`. They have not been drawn again
since 0125 (2026-10-15): the keys below follow Chapter 1's start as it was
then, so check each picture against the game before committing it.

```bash
cargo run -q -p xtask --features private-assets -- frame-png docs/img/<name>.png <steps>
```

With `N="f f Down Down f f Down f"` (New Game, Classic, "familiar with this
game", the lead as offered, Start), `B` = `--keys "$N" --wait 6 --keys "<80
× f>" --wait 3` (the opening scene clicked through to the battle), and
`A="f Right Right Right Up Up f"` (turn 2: the lead steps up to the brigand
who came down in the enemy phase):

| Picture | Steps |
| ------- | ----- |
| `battle-forecast` | `$B --keys "Space f" --wait 15 --keys "$A" --wait 1.5 --keys "f Down" --wait 0.5` |
| `battle-glyphs` | the same, built without `--features private-assets` |
| `combat-scene` | the same, then `--keys "f f" --wait 1.0` instead of the last keys (the archer) |
| `battle-ranges` | `$B --keys f --wait 0.5` |
| `spell-menu` | `$B --keys "Down f f" --wait 1 --keys "Down f" --wait 0.5` |
| `dialogue` | `--keys "$N" --wait 6 --keys "<60 × f>" --wait 4` |
| `lead-select` | `--keys "f f Down Down f" --wait 0.5` |
| `title` | `--wait 3` |
| `key-bindings` | `--keys "Down Down f"`, thirteen `Down`s, then `f` |

`<80 × f>` is the key `f` written 80 times, space-separated. The README's
Playdate picture, `docs/design/playdate/q3r2-e-grey.png`, is 0061's chosen
mockup, not a screenshot.
