# The writer trial

How Nick finds a writer for the dialogue (ticket 0766): each applicant gets
a **packet**, rewrites Chapter 1's scenes in it, and sends them back; the
entries are read side by side without names; Nick picks. No applicant gets
any access to a repository.

| File | What |
| ---- | ---- |
| `ad.md` | The ad as Nick posted it, and the offer it made |
| `brief.md` | In the packet: what to do, how to send it, what is judged, the offer |
| `terms.md` | In the packet: the packet and the entry in plain words (the signed agreement governs) |
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
   ```

3. Put anything every packet should carry in `dist/trial/extra/`: the
   confidentiality agreement's text, say. (A copy each applicant signed
   isn't one of these: they sign before they get the packet.)

For each applicant, after they signed the agreement:

1. On the Windows demo's itch.io project, make one download key with
   their handle as its label (`docs/releasing.md`, *The demo's itch.io
   pages*) and copy its link.
2. Run:

   ```
   cargo xtask trial-packet --for <handle> --key <the key's link> --due "<the date you agreed>"
   ```

   It writes `dist/trial/<handle>.zip`. The handle is the packet's mark,
   in its zip's name, the first line of its script and its
   `START-HERE.txt`: letters, digits, `-` and `_`. It stops, and makes no
   packet, if a link or the date is missing or if the scenes have a
   problem. The first run downloads the preview from the mirror's kit
   release.
3. Send the zip from your own cloud drive or as an attachment.

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
