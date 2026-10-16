//! Content errors: every problem names the file and, when known, the line and
//! column, so a broken asset can be fixed without reading Rust (ADR-0005).
//! They are Shuyi Engine's (`shuyi_assets::error`, ticket 1103).
pub use shuyi_assets::error::{ContentError, ContentErrors};
