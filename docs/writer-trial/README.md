# The writer trial

How Nick finds a writer for the dialogue (ticket 0766): each applicant gets
a **packet**, rewrites Chapter 1's scenes in it, and sends them back; the
entries are read side by side without names; Nick picks. No applicant gets
any access to a repository.

| File | What |
| ---- | ---- |
| `ad.md` | The ad as Nick posted it, and the offer it made |
| `brief.md` | In the packet: what to do, how to send it, what is judged, the offer |
| `terms.md` | In the packet: the packet and the entry in plain words (the two signed agreements govern) |
| `start-here.txt` | In the packet, filled in for each applicant: the mark, the due date, the links |
| `packet.txt` | Every other page in the packet, cut from the private story documents at build time |
| `reading-guide.md` | The top of the comparison page |

## Sending a packet

Once, in the game's folder (none of these are ever committed: `dist/` is
ignored):

1. `cargo shuyi private-assets` (the scenes and the story pages are there).
2. Write `dist/trial/links.txt`, one `name = value` a line:

   ```
   demo = <the web demo's itch.io page>
   demo-password = <its password>
   preview = <the web script preview's itch.io page>
   preview-password = <its password>
   email = <where entries are sent>
   ```

3. Put anything every packet should carry in `dist/trial/extra/`: the
   two agreements' text, say. (A copy each applicant signed
   isn't one of these: they sign before they get the packet.)

The packet holds text files only, no program (Nick, 2026-11-14: an
applicant asked to run an `.exe` may balk). The applicant plays the web
demo and sees their script in the **web script preview** (ticket 0767),
both in the browser. The preview's itch.io project has to exist first:
`docs/releasing.md`, *The script preview's itch.io page* (an HTML project,
Restricted, with a password of its own; its name in the repository
variable `ITCH_PREVIEW_PROJECT`; then **Actions → Story kit → Run
workflow** pushes it).

**Send no packet before ticket 0768 is in** (Nick, 2026-11-15: writers
shouldn't be expected to be technical). The format is easy to break by
hand: a reply's reaction lines must be indented by exactly two spaces, and
every `@choice`, `@if` and `@scene` needs its closing line. 0768 puts the
editor colors and indenting in the packet; until its folder
(`tools/vscode-dlg/`) is there, `trial-packet` refuses to make a packet.

**Two more before the first packet** (from the test audit, 2026-11-19; each
under a day, no decision needed). `dist/trial/before-sending.md` is the short
list to play through once both are in:

- **0778**: the web preview is pushed to itch.io only by hand. Until it
  lands, Story kit republishes the preview by itself every night, so
  applicants would get a new preview mid-trial. After it, push both pages
  the same day: Actions → Demo, then Actions → Story kit, both by hand.
- **1303**: unnamed enemies show the soldier face in unit info. Until it
  lands, every Red Company enemy but Harl shows a `portrait` placeholder box.

Test as an applicant in a **new Chrome profile** (or after clearing the site
data of `html-classic.itch.zone`), in a normal window. A private or incognito
window may refuse itch's game frame its storage, so settings and a suspended
battle would vanish for a reason applicants won't have.

For each applicant, after they signed both agreements with Ibasho Corp. (the
confidentiality agreement and the Writer Trial and AI Voice Acknowledgment):

1. Run:

   ```
   cargo xtask trial-packet --for <handle> --due "<the date you agreed>"
   ```

   It writes `dist/trial/<handle>.zip`. The handle is the packet's mark,
   in its zip's name, the first line of its script and its
   `START-HERE.txt`: letters, digits, `-` and `_`. It stops, and makes no
   packet, if a link or the date is missing or if the scenes have a
   problem.
2. Send the zip from your own cloud drive or as an attachment.

The web pages share one password each, so a leaked password doesn't say
whose it was (the packet's mark does, if the packet itself leaks). Change
a password when an applicant drops out (`docs/releasing.md`).

## Comparing the entries

Put each entry in `dist/trial/entries/<handle>/` (its `.dlg` file, and its
note beside it), then:

```
cargo xtask trial-compare
```

It writes `dist/trial/compare.html`: the reading guide, then each scene
with the current draft and every entry side by side, named Entry A, B, C …
in a random order, with the problems the game's checks find in each. Which
entry is whose is in `dist/trial/compare-key.txt`: open it after picking.
Each run shuffles the letters again.
