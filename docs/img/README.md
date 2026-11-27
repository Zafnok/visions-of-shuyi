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

## The stills, `shots/`

The README's nine stills are `shuyi.ron`'s **shots** (ticket 1389): its
`shots` part names each and holds the `frame-png` arguments that draw it,
and shot `<name>` is always `shots/<name>.png`, 1600×1024 (scale 2). Make
them all again, with the bought art:

```bash
cargo run -q -p trpg-tools --features private-assets -- shuyi frame-png --shots
```

or one with `--shot <name>` in place of `--shots`. Plain `cargo shuyi
frame-png --shot <name>`, which a stale shot's error names, draws without
the bought art: add the feature as above. Look at a remade picture before
committing it. Changing a picture's moment means changing its arguments in
`shuyi.ron` (its comment says what each shows).

A test checks that each file is still what its arguments draw, pixel for
pixel, under 256 KiB, and that `shots/` holds nothing else. It needs the
bought art, so the gates never run it (ADR-0040); run it after a change
the stills show (the title, a screen, Chapter 1's battle, the art):

```bash
cargo test -p trpg-tools --features private-assets the_readme_s_shots
```

The README's Playdate picture, `docs/design/playdate/q3r2-e-grey.png`, is
0061's chosen mockup, not a screenshot.
