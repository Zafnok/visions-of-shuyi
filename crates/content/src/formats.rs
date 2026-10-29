//! The game's format registry (engine ADR-0007 and ADR-0019, ticket 1173):
//! the engine's standard formats, those configured with the game's rules
//! in their place, and the formats of the game's own kinds, registered
//! once ([`install`]) as the bundle is. A loader asks it for one asset by
//! path ([`load`], [`table`]) or for a folder's ([`Formats::load_each`]);
//! the checks against other assets run after, in the loader.
use crate::prelude::*;

use core::marker::PhantomData;

use once_cell::race::OnceBox;
use shuyi_assets::credits::CreditsRon;
use shuyi_assets::dialogue::Dlg;
use shuyi_assets::font::FontAtlasRon;
use shuyi_assets::format::{NOT_FOUND, RegisterError, TEXT_MAX_BYTES, text};
use shuyi_assets::keymap::KeymapRon;
use shuyi_assets::lighting::LightingRon;
use shuyi_assets::palette::PaletteRon;
pub use shuyi_assets::{AssetKind, AssetPath, DecodeCx, Format, Formats};

use crate::error::ContentError;
use crate::{audio, bundle, cinematic, dialogue, font, keymap, lighting, map, palette, terrain};

/// The registry, once built.
static FORMATS: OnceBox<Formats> = OnceBox::new();

/// The game's registry, the process-wide one: built the first time
/// anything loads through it.
pub fn install() -> &'static Formats {
    FORMATS.get_or_init(|| {
        let mut formats = Formats::standard();
        // None is refused: a test checks it.
        let _ = register(&mut formats);
        Box::new(formats)
    })
}

/// Puts the game's formats in `formats`: the standard ones the game
/// configures in place of the engine's, then those of the game's kinds.
pub fn register(formats: &mut Formats) -> Result<(), RegisterError> {
    formats.replace(PaletteRon::requiring(
        palette::REQUIRED_COLORS.iter().copied(),
    ))?;
    formats.replace(FontAtlasRon::requiring(font::REQUIRED_GLYPHS))?;
    formats.replace(LightingRon::new(lighting::NOON))?;
    formats.replace(CreditsRon::new(
        bundle::display_path(audio::AUDIO_PATH),
        bundle::PRIVATE_DISPLAY_ROOT,
    ))?;
    formats.register(Dlg::<dialogue::Stage>::new())?;
    formats.register(KeymapRon::<keymap::Action>::new())?;
    // The legend's terrain ids, as the terrain file lists them (every
    // bundle the game reads has the same one); its colours are checked
    // when the terrain loads.
    let terrain = terrain::TerrainDef::load(None).unwrap_or_default();
    map::register(formats, terrain.display)?;
    cinematic::register(formats)?;
    formats.register(TableRon::<trpg_core::AiWeights>::default())?;
    formats.register(TableRon::<trpg_core::ArtTable>::default())?;
    formats.register(TableRon::<trpg_core::BasicStock>::default())?;
    formats.register(TableRon::<trpg_core::ItemTable>::default())?;
    formats.register(TableRon::<trpg_core::SkillTable>::default())?;
    Ok(())
}

/// The asset of kind `K` at `path` in the game's bundle. A file the bundle
/// lacks is "not found", as the game's loaders always said.
pub fn load<K: AssetKind>(path: &str) -> Result<K, Vec<ContentError>> {
    let bundle = bundle::install();
    if !bundle.has(path) {
        return Err(vec![ContentError::new(
            bundle.display_path(path),
            NOT_FOUND,
        )]);
    }
    install().load(bundle, path)
}

/// The data table `T` in the file at `path` in the game's bundle.
pub fn table<T: DataTable>(path: &str) -> Result<T, Vec<ContentError>> {
    load::<Table<T>>(path).map(|table| table.0)
}

/// A data table of `trpg_core` read from its file alone: a kind as
/// [`Table`].
pub trait DataTable: Sized + 'static {
    /// What messages call the table.
    const NAME: &'static str;

    /// Parses and checks `source`, attributing errors to `file`.
    fn from_source(file: &str, source: &str) -> Result<Self, Vec<ContentError>>;
}

/// The data table `T` as an asset kind: a type of `trpg_core` can't be one
/// itself, since neither it nor [`AssetKind`] is this crate's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table<T>(pub T);

impl<T: DataTable> AssetKind for Table<T> {
    const NAME: &'static str = T::NAME;
}

/// A data table's RON file (`ron`, `.ron`), read by its
/// [`DataTable::from_source`].
pub struct TableRon<T>(PhantomData<fn() -> T>);

impl<T> Default for TableRon<T> {
    /// The format of table `T`.
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: DataTable> Format for TableRon<T> {
    type Asset = Table<T>;

    fn id(&self) -> &'static str {
        "ron"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["ron"]
    }

    fn max_bytes(&self) -> usize {
        TEXT_MAX_BYTES
    }

    fn decode(
        &self,
        file: &AssetPath,
        bytes: &[u8],
        _cx: &DecodeCx<'_>,
    ) -> Result<Table<T>, Vec<ContentError>> {
        T::from_source(file.shown(), text(file, bytes)?).map(Table)
    }
}

#[cfg(test)]
mod tests;
