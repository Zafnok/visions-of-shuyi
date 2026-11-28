//! Ids as the data files write them: a bare string (`"iron_sword"`), where
//! a save writes the id's own form (`("iron_sword")`). A definition read
//! straight from a data file (ticket 1392) names these functions on its id
//! fields with `#[serde(deserialize_with = …)]`; the ids' own `Deserialize`
//! stays the save's.
use crate::prelude::*;

use serde::{Deserialize, Deserializer};

/// One id, written as a bare string.
///
/// # Errors
/// What the deserializer reports when the value isn't a string.
pub fn one<'de, D: Deserializer<'de>, T: From<String>>(d: D) -> Result<T, D::Error> {
    String::deserialize(d).map(T::from)
}

/// A list of ids, each written as a bare string.
///
/// # Errors
/// What the deserializer reports when the value isn't a list of strings.
pub fn list<'de, D: Deserializer<'de>, T: From<String>>(d: D) -> Result<Vec<T>, D::Error> {
    let names = Vec::<String>::deserialize(d)?;
    Ok(names.into_iter().map(T::from).collect())
}

/// An optional id, written `Some("…")` or `None`.
///
/// # Errors
/// What the deserializer reports when the value isn't an optional string.
pub fn option<'de, D: Deserializer<'de>, T: From<String>>(d: D) -> Result<Option<T>, D::Error> {
    Ok(Option::<String>::deserialize(d)?.map(T::from))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{ArtId, ItemId};

    #[derive(Deserialize)]
    struct Row {
        #[serde(deserialize_with = "super::one")]
        art: ArtId,
        #[serde(deserialize_with = "super::list")]
        items: Vec<ItemId>,
        #[serde(default, deserialize_with = "super::option")]
        spare: Option<ItemId>,
    }

    #[test]
    fn reads_bare_strings() {
        let row: Row = ron::from_str(r#"(art: "vault", items: ["a", "b"], spare: Some("c"))"#)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(row.art, ArtId::new("vault"));
        assert_eq!(row.items, [ItemId::new("a"), ItemId::new("b")]);
        assert_eq!(row.spare, Some(ItemId::new("c")));
        let row: Row =
            ron::from_str(r#"(art: "vault", items: [])"#).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(row.spare, None);
    }

    #[test]
    fn a_save_still_writes_the_id_form() {
        let text = ron::to_string(&ItemId::new("a")).unwrap_or_default();
        assert_eq!(text, r#"("a")"#);
    }
}
