# Story

The story's documents are not in this repository. Since ticket 0159
(ADR-0117) they live in the private assets repository's `story/`: Nick's
beats, the bible, the characters, the outline, the chapter beat sheets,
the ledger, the names registry, the craft notes and the writer's guide,
and in `story/design/` the story parts of `docs/design/` (a design
document that lost one says so where it was).

To get them, with access to the private repository:

```bash
cargo shuyi private-assets
```

It checks them out into `assets-private/story/` (git-ignored here); start
with its `README.md`. A change to them is a pull request in the private
repository, then this repository's that moves `assets-private.rev`: the
`content-change` skill. How the story is written: ADR-0011 and the
`story-writing` skill.

The game's own content is beside them, in the private repository's
`content/` since ticket 0160: the scripts (`dialogue/*.dlg`), chapters,
battles, maps, world maps, cinematics, openings, skirmishes and the data
tables, laid out like `assets/` and checked out into
`assets-private/content/`, which every build that has it lays over
`assets/` (ADR-0117). A build without it starts the test chapter on the
public stand-ins (ADR-0121).
