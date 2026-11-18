//! `readme-pictures` (ticket 1264): the README's intro animation,
//! `docs/img/intro.gif`, drawn again from the game with no hand step.
//!
//! The title's intro cinematic plays to its song; the picture shows two
//! stretches of it, 56 s to 74.5 s (the party sets off, the brigand comes
//! down, the combat scene) and 114.7 s to 117.7 s (a moment of the whole
//! field, then the logo, from the cut at 115.8 s, and the menu land), at
//! 10 frames a second: the moments ticket 0125 chose. `frame-png --gif`
//! draws at most 64 frames in one run, and one run can't skip from 74.5 s
//! to 114.7 s, so the stretches are drawn in four clips and joined
//! here. The join cuts off the bottom 16 rows (the help bar and the debug
//! key's hint, which `frame-png` has no option to leave out), keeps only
//! the pixels each frame changes (the engine's GIF stores every frame
//! whole, about twice the size), shows a frame that repeats the last one
//! as one longer frame, and holds the menu for 3 s before the loop. The
//! colours stay exact: each frame's palette is the frame's own colours.
//!
//! The README's other pictures and their commands: `docs/img/README.md`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// `readme-pictures --help`.
pub const USAGE: &str = "\
usage: cargo run -q -p xtask --features private-assets -- readme-pictures

Writes docs/img/intro.gif, the README's intro animation: the title's
intro cinematic from 56 s to 74.5 s and 114.7 s to 117.7 s of its song, at 10
frames a second, drawn by `frame-png --gif` in four clips and joined
(docs/img/README.md). Needs the bought art (`cargo shuyi private-assets`).";

/// Where the picture goes, from the repository's root.
pub const OUT: &str = "docs/img/intro.gif";

/// One run of `frame-png`: `count` frames a tenth of a second apart, the
/// first `start` tenths of a second into the title's song.
struct Clip {
    start: u32,
    count: u32,
}

/// The clips, in the order they play.
const CLIPS: [Clip; 4] = [
    // 56.0 to 62.3: the party sets off east along the road.
    Clip {
        start: 560,
        count: 64,
    },
    // 62.4 to 68.7: the party moves up; the brigand comes down to the lead.
    Clip {
        start: 624,
        count: 64,
    },
    // 68.8 to 74.4: the brigand attacks; the combat scene.
    Clip {
        start: 688,
        count: 57,
    },
    // 114.7 to 117.7: the whole field a moment, then the logo lands at
    // 115.8 (the song's last chord), then the menu.
    Clip {
        start: 1147,
        count: 31,
    },
];

/// The rows cut off the bottom of every frame: the help bar and the debug
/// key's hint.
const CUT_ROWS: u16 = 16;

/// How long the last frame (the menu) shows before the loop, in
/// hundredths of a second.
const HOLD: u16 = 300;

/// The longest `--wait` the Harness takes is 2000 frames (33.3 s): a
/// clip's start is waited in pieces of 30 s.
const WAIT_PIECE: u32 = 300;

/// `frame-png`'s arguments for `clip`, written to `out`.
fn clip_args(clip: &Clip, out: &str) -> Vec<String> {
    let mut args = vec![out.to_owned(), "--scale".into(), "1".into()];
    let mut left = clip.start;
    while left > 0 {
        let piece = left.min(WAIT_PIECE);
        args.push("--wait".into());
        args.push(format!("{}.{}", piece / 10, piece % 10));
        left -= piece;
    }
    let count = clip.count.to_string();
    args.extend(["--every", "0.1", "--count", &count, "--gif"].map(String::from));
    args
}

/// Draws the four clips with `frame_png` (`frame-png`'s arguments, its
/// exit code) and writes the joined picture to [`OUT`] under `root`.
pub fn command(root: &Path, args: &[String], mut frame_png: impl FnMut(&[String]) -> u8) -> u8 {
    if !args.is_empty() {
        eprintln!("{USAGE}");
        return 2;
    }
    let dir = std::env::temp_dir().join(format!("xtask-readme-pictures-{}", std::process::id()));
    let result = draw(root, &dir, &mut frame_png);
    let _ = fs::remove_dir_all(&dir);
    match result {
        Ok(summary) => {
            println!("readme-pictures: {summary}");
            0
        }
        Err(e) => {
            eprintln!("readme-pictures: {e}");
            1
        }
    }
}

fn draw(
    root: &Path,
    dir: &Path,
    frame_png: &mut impl FnMut(&[String]) -> u8,
) -> Result<String, String> {
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut clips = Vec::new();
    for (i, clip) in CLIPS.iter().enumerate() {
        let path = dir.join(format!("clip-{i}.gif"));
        if frame_png(&clip_args(clip, &path.to_string_lossy())) != 0 {
            return Err(format!(
                "frame-png failed on clip {} of {}",
                i + 1,
                CLIPS.len()
            ));
        }
        clips.push(fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?);
    }
    let (bytes, frames) = join(&clips, CUT_ROWS, HOLD)?;
    let out = root.join(OUT);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&out, &bytes).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(format!("{OUT}: {frames} frames, {} bytes", bytes.len()))
}

/// Joins GIFs of one size into one that loops forever: `cut_rows` cut off
/// the bottom, each frame stored as the part it changes, a repeated frame
/// shown as one longer frame, the last held `hold` hundredths of a second.
/// Returns the GIF and its number of frames.
fn join(clips: &[Vec<u8>], cut_rows: u16, hold: u16) -> Result<(Vec<u8>, usize), String> {
    let mut joiner: Option<Joiner> = None;
    for bytes in clips {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options
            .read_info(bytes.as_slice())
            .map_err(|e| e.to_string())?;
        let (width, height) = (decoder.width(), decoder.height());
        let kept = height.saturating_sub(cut_rows);
        let joiner = match &mut joiner {
            Some(joiner) => joiner,
            empty => empty.insert(Joiner::new(width, kept)?),
        };
        if (width, kept) != (joiner.width, joiner.height) {
            return Err(format!(
                "a clip of {width}×{height} among clips of another size"
            ));
        }
        let mut canvas = vec![0; usize::from(width) * usize::from(height) * 4];
        while let Some(frame) = decoder.read_next_frame().map_err(|e| e.to_string())? {
            paint(&mut canvas, width, frame);
            joiner.push(&canvas[..joiner.len()], frame.delay)?;
        }
    }
    joiner.ok_or_else(|| "no clips".to_owned())?.finish(hold)
}

/// Paints a decoded frame (RGBA, its own rectangle) over `canvas`, but its
/// transparent pixels.
fn paint(canvas: &mut [u8], width: u16, frame: &gif::Frame<'_>) {
    let width = usize::from(width);
    for (y, row) in frame
        .buffer
        .chunks_exact(usize::from(frame.width) * 4)
        .enumerate()
    {
        let start = ((usize::from(frame.top) + y) * width + usize::from(frame.left)) * 4;
        for (dst, src) in canvas[start..start + row.len()]
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(row.as_chunks::<4>().0)
        {
            if src[3] != 0 {
                *dst = *src;
            }
        }
    }
}

/// The joined GIF as it is written: the last picture, and the frame that
/// waits for its delay to be known.
struct Joiner {
    width: u16,
    height: u16,
    encoder: gif::Encoder<Vec<u8>>,
    last: Vec<u8>,
    pending: Option<gif::Frame<'static>>,
    frames: usize,
}

impl Joiner {
    fn new(width: u16, height: u16) -> Result<Self, String> {
        let mut encoder =
            gif::Encoder::new(Vec::new(), width, height, &[]).map_err(|e| e.to_string())?;
        encoder
            .set_repeat(gif::Repeat::Infinite)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            width,
            height,
            encoder,
            last: Vec::new(),
            pending: None,
            frames: 0,
        })
    }

    /// Bytes in one picture (RGBA).
    fn len(&self) -> usize {
        usize::from(self.width) * usize::from(self.height) * 4
    }

    /// Adds `image` (RGBA, the whole picture) shown `delay` hundredths.
    fn push(&mut self, image: &[u8], delay: u16) -> Result<(), String> {
        if image == self.last.as_slice() {
            if let Some(pending) = &mut self.pending {
                pending.delay = pending.delay.saturating_add(delay);
            }
            return Ok(());
        }
        let frame = self.changes(image, delay)?;
        self.flush()?;
        self.pending = Some(frame);
        self.last = image.to_vec();
        Ok(())
    }

    /// The part of `image` that differs from the last picture, as a frame
    /// drawn over it: the pixels that didn't change are transparent when
    /// the palette has room for that.
    fn changes(&self, image: &[u8], delay: u16) -> Result<gif::Frame<'static>, String> {
        let width = usize::from(self.width);
        let pixel = |buf: &[u8], x: usize, y: usize| -> [u8; 4] {
            let i = (y * width + x) * 4;
            [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
        };
        let first = self.last.is_empty();
        let (mut left, mut top, mut right, mut bottom) = (width, usize::from(self.height), 0, 0);
        for y in 0..usize::from(self.height) {
            for x in 0..width {
                if first || pixel(image, x, y) != pixel(&self.last, x, y) {
                    left = left.min(x);
                    right = right.max(x + 1);
                    top = top.min(y);
                    bottom = bottom.max(y + 1);
                }
            }
        }
        let colours = |only_changed: bool| {
            let mut map: HashMap<[u8; 3], u8> = HashMap::new();
            let mut palette = Vec::new();
            let mut indices = Vec::with_capacity((right - left) * (bottom - top));
            let mut too_many = false;
            for y in top..bottom {
                for x in left..right {
                    let p = pixel(image, x, y);
                    if only_changed && p == pixel(&self.last, x, y) {
                        indices.push(None);
                        continue;
                    }
                    let rgb = [p[0], p[1], p[2]];
                    let next = map.len();
                    let index = *map.entry(rgb).or_insert_with(|| {
                        palette.extend_from_slice(&rgb);
                        u8::try_from(next).unwrap_or_else(|_| {
                            too_many = true;
                            0
                        })
                    });
                    indices.push(Some(index));
                }
            }
            (map.len(), palette, indices, too_many)
        };
        // With a free index, the unchanged pixels are transparent;
        // otherwise every pixel of the rectangle is stored.
        let (mut count, mut palette, mut indices, mut too_many) = colours(!first);
        let mut transparent = None;
        if !first && count < 256 {
            transparent = u8::try_from(count).ok();
            palette.extend_from_slice(&[0, 0, 0]);
        } else if !first {
            (count, palette, indices, too_many) = colours(false);
        }
        if too_many {
            return Err(format!(
                "a frame of {count} colours: a GIF frame has at most 256"
            ));
        }
        let buffer = indices
            .into_iter()
            .map(|i| i.or(transparent).unwrap_or(0))
            .collect::<Vec<_>>();
        let narrow = |n: usize| u16::try_from(n).map_err(|e| e.to_string());
        Ok(gif::Frame {
            delay,
            dispose: gif::DisposalMethod::Keep,
            transparent,
            left: narrow(left)?,
            top: narrow(top)?,
            width: narrow(right - left)?,
            height: narrow(bottom - top)?,
            palette: Some(palette),
            buffer: Cow::Owned(buffer),
            ..gif::Frame::default()
        })
    }

    /// Writes the frame that waited.
    fn flush(&mut self) -> Result<(), String> {
        if let Some(frame) = self.pending.take() {
            self.encoder
                .write_frame(&frame)
                .map_err(|e| e.to_string())?;
            self.frames += 1;
        }
        Ok(())
    }

    /// Holds the last frame `hold` hundredths and ends the GIF.
    fn finish(mut self, hold: u16) -> Result<(Vec<u8>, usize), String> {
        if let Some(pending) = &mut self.pending {
            pending.delay = hold;
        }
        self.flush()?;
        let frames = self.frames;
        Ok((
            self.encoder.into_inner().map_err(|e| e.to_string())?,
            frames,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A GIF of `width`×`height` whose frames are `(colour of every pixel
    /// but the first, colour of the first pixel)`, each `delay` long.
    fn clip(width: u16, height: u16, frames: &[([u8; 3], [u8; 3])], delay: u16) -> Vec<u8> {
        let mut encoder = gif::Encoder::new(Vec::new(), width, height, &[]).unwrap();
        for &(fill, first) in frames {
            let mut buffer = vec![1; usize::from(width) * usize::from(height)];
            buffer[0] = 0;
            let mut palette = first.to_vec();
            palette.extend_from_slice(&fill);
            let frame = gif::Frame {
                delay,
                width,
                height,
                palette: Some(palette),
                buffer: Cow::Owned(buffer),
                ..gif::Frame::default()
            };
            encoder.write_frame(&frame).unwrap();
        }
        encoder.into_inner().unwrap()
    }

    /// A stored frame: left, top, width, height, delay, transparent.
    type Rect = (u16, u16, u16, u16, u16, bool);

    /// The joined GIF's size, its frames and the whole picture after
    /// each, in RGBA.
    fn frames(bytes: &[u8]) -> (u16, u16, Vec<Rect>, Vec<Vec<u8>>) {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options.read_info(bytes).unwrap();
        assert_eq!(decoder.repeat(), gif::Repeat::Infinite);
        let (width, height) = (decoder.width(), decoder.height());
        let mut canvas = vec![0; usize::from(width) * usize::from(height) * 4];
        let (mut rects, mut pictures) = (Vec::new(), Vec::new());
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            rects.push((
                frame.left,
                frame.top,
                frame.width,
                frame.height,
                frame.delay,
                frame.transparent.is_some(),
            ));
            paint(&mut canvas, width, frame);
            pictures.push(canvas.clone());
        }
        (width, height, rects, pictures)
    }

    const RED: [u8; 3] = [255, 0, 0];
    const BLUE: [u8; 3] = [0, 0, 255];
    const GREEN: [u8; 3] = [0, 255, 0];

    #[test]
    fn clip_args_wait_in_pieces_of_30_seconds() {
        let args = clip_args(
            &Clip {
                start: 624,
                count: 64,
            },
            "a.gif",
        );
        assert_eq!(
            args.join(" "),
            "a.gif --scale 1 --wait 30.0 --wait 30.0 --wait 2.4 --every 0.1 --count 64 --gif"
        );
        let args = clip_args(
            &Clip {
                start: 1160,
                count: 31,
            },
            "b.gif",
        );
        assert_eq!(
            args.join(" "),
            "b.gif --scale 1 --wait 30.0 --wait 30.0 --wait 30.0 --wait 26.0 --every 0.1 --count 31 --gif"
        );
    }

    #[test]
    fn the_clips_follow_the_old_pictures_moments() {
        let tenths: Vec<_> = CLIPS
            .iter()
            .map(|c| (c.start, c.start + c.count - 1))
            .collect();
        assert_eq!(tenths, [(560, 623), (624, 687), (688, 744), (1147, 1177)]);
    }

    #[test]
    fn join_cuts_merges_repeats_stores_changes_and_holds_the_last() {
        let a = clip(4, 3, &[(RED, RED), (RED, RED), (RED, BLUE)], 10);
        let b = clip(4, 3, &[(GREEN, BLUE)], 10);
        let (bytes, count) = join(&[a, b], 1, 300).unwrap();
        let (width, height, rects, pictures) = frames(&bytes);
        assert_eq!((width, height, count), (4, 2, 3));
        assert_eq!(
            rects,
            [
                // The first whole; the repeat added to its delay.
                (0, 0, 4, 2, 20, false),
                // One pixel changed: only it.
                (0, 0, 1, 1, 10, true),
                // Every pixel but the first changed; held at the end.
                (0, 0, 4, 2, 300, true),
            ]
        );
        let rgba = |c: [u8; 3]| [c[0], c[1], c[2], 255];
        let picture = |first: [u8; 3], rest: [u8; 3]| {
            let mut p = rgba(first).to_vec();
            (1..8).for_each(|_| p.extend_from_slice(&rgba(rest)));
            p
        };
        assert_eq!(
            pictures,
            [picture(RED, RED), picture(BLUE, RED), picture(BLUE, GREEN)]
        );
    }

    #[test]
    fn join_refuses_clips_of_two_sizes_and_no_clips() {
        let a = clip(4, 3, &[(RED, RED)], 10);
        let b = clip(5, 3, &[(RED, RED)], 10);
        assert!(join(&[a, b], 1, 300).unwrap_err().contains("5×3"));
        assert_eq!(join(&[], 1, 300).unwrap_err(), "no clips");
        assert!(join(&[b"not a gif".to_vec()], 1, 300).is_err());
    }

    #[test]
    fn join_refuses_a_frame_of_more_than_256_colours() {
        let mut joiner = Joiner::new(300, 1).unwrap();
        let image: Vec<u8> = (0..300u32)
            .flat_map(|i| {
                let [low, high, ..] = i.to_le_bytes();
                [low, high, 0, 255]
            })
            .collect();
        assert!(joiner.push(&image, 10).unwrap_err().contains("300 colours"));
    }

    #[test]
    fn a_change_of_256_colours_is_stored_whole_without_transparency() {
        let mut first = vec![0; 256 * 4];
        first
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|p| p[3] = 255);
        let second: Vec<u8> = (0..=255u8).flat_map(|i| [i, 255 - i, 7, 255]).collect();
        let mut joiner = Joiner::new(256, 1).unwrap();
        joiner.push(&first, 10).unwrap();
        joiner.push(&second, 10).unwrap();
        let (bytes, _) = joiner.finish(300).unwrap();
        let (_, _, rects, pictures) = frames(&bytes);
        assert_eq!(rects[1], (0, 0, 256, 1, 300, false));
        assert_eq!(pictures, [first, second]);
    }

    #[test]
    fn command_draws_four_clips_and_writes_the_picture() {
        let root =
            std::env::temp_dir().join(format!("xtask-readme-pictures-test-{}", std::process::id()));
        let mut runs = Vec::new();
        let code = command(&root, &[], |args| {
            runs.push(args[1..].join(" "));
            fs::write(&args[0], clip(4, 20, &[(RED, BLUE)], 10)).unwrap();
            0
        });
        assert_eq!(code, 0);
        assert_eq!(runs.len(), 4);
        assert!(runs[0].starts_with("--scale 1 --wait 30.0 --wait 26.0 --every 0.1 --count 64"));
        let (width, height, rects, _) = frames(&fs::read(root.join(OUT)).unwrap());
        assert_eq!((width, height, rects.len()), (4, 4, 1));
        assert_eq!(command(&root, &["--bogus".into()], |_| 0), 2);
        assert_eq!(command(&root, &[], |_| 1), 1);
        fs::remove_dir_all(&root).unwrap();
    }
}
