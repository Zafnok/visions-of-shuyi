# The `music/` folder

The game's music tracks (OGG Vorbis), named by `assets/audio/audio.ron`.
They are **not** embedded in the game: this folder ships next to the exe and
next to the WASM on the web, and the game loads a track when it first plays
it ([ADR-0026](adr/0026-audio-cues-and-music-files.md)).

Each track's length is in `audio.ron` too (`length_ms`); replace a file
and `cargo test -p trpg-content` prints the new value to write.

Every file in the folder ships (`shuyi.ron`'s `export` part), so nothing but
the tracks goes in it.
