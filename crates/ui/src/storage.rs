//! Persistence (ADR-0009): saves (0802) and settings (0805) go through
//! [`Storage`] so `trpg-ui` stays pure (ADR-0004). Shuyi Engine's
//! (`shuyi_ui::storage`, ticket 1109), re-exported here under its old
//! name. `app` supplies the real implementation per platform;
//! [`MemoryStorage`] is for tests and the test harness.

pub use shuyi_ui::storage::*;
