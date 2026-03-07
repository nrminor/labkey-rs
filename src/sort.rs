//! Sort specification types for `LabKey` query endpoints.
//!
//! `LabKey` query endpoints accept a comma-separated sort string where each
//! segment is a column name optionally prefixed with `-` for descending order.
//! For example, `"Name,-Created"` sorts by `Name` ascending then `Created`
//! descending.
//!
//! This module provides [`QuerySort`] as a parsed, validated representation of
//! that wire format, replacing raw `Option<String>` on query option structs.
//! [`QuerySort`] round-trips through its [`Display`](std::fmt::Display) and
//! [`parse`](QuerySort::parse) implementations.
//!
//! The [`SortDirection`] and [`ColumnSort`] types are also used by the
//! `getData` endpoint (replacing the former `GetDataSort` / `GetDataSortDirection`)
//! and by [`QueryViewSort`](crate::query::QueryViewSort) in query detail responses.
//!
//! # Examples
//!
//! ```
//! use labkey_rs::sort::{ColumnSort, QuerySort, SortDirection};
//!
//! let sort = QuerySort::parse("Name,-Created");
//! assert_eq!(sort.columns().len(), 2);
//! assert_eq!(sort.columns()[0].column(), "Name");
//! assert_eq!(sort.columns()[0].direction(), SortDirection::Ascending);
//! assert_eq!(sort.columns()[1].column(), "Created");
//! assert_eq!(sort.columns()[1].direction(), SortDirection::Descending);
//!
//! // Display produces the wire format string.
//! assert_eq!(sort.to_string(), "Name,-Created");
//!
//! // Build programmatically.
//! let sort = QuerySort::from(vec![
//!     ColumnSort::ascending("Name"),
//!     ColumnSort::descending("Age"),
//! ]);
//! assert_eq!(sort.to_string(), "Name,-Age");
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

/// Direction of a column sort.
///
/// Used by [`ColumnSort`] and [`QuerySort`], and also by
/// [`QueryViewSort`](crate::query::QueryViewSort) for deserialized query
/// detail responses.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortDirection {
    /// Sort in ascending order (the default when no `-` prefix is present).
    #[default]
    Ascending,
    /// Sort in descending order (indicated by a `-` prefix in the wire format).
    Descending,
}

impl SortDirection {
    /// Returns the opposite direction.
    #[must_use]
    pub fn reversed(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ascending => f.write_str("ASC"),
            Self::Descending => f.write_str("DESC"),
        }
    }
}

impl Serialize for SortDirection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::Ascending => "ASC",
            Self::Descending => "DESC",
        })
    }
}

impl<'de> Deserialize<'de> for SortDirection {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "ASC" | "+" => Ok(Self::Ascending),
            "DESC" | "-" => Ok(Self::Descending),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["ASC", "DESC", "+", "-"],
            )),
        }
    }
}

/// A single column sort specification: a column name paired with a direction.
///
/// Constructed via [`ascending`](Self::ascending),
/// [`descending`](Self::descending), or [`new`](Self::new).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSort {
    column: String,
    direction: SortDirection,
}

impl ColumnSort {
    /// Create a sort on `column` in the given `direction`.
    ///
    /// Empty column names are accepted here; [`QuerySort::parse`] silently
    /// skips empty segments, but programmatic construction does not filter
    /// because the caller chose to create it explicitly.
    #[must_use]
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }

    /// Create an ascending sort on `column`.
    #[must_use]
    pub fn ascending(column: impl Into<String>) -> Self {
        Self::new(column, SortDirection::Ascending)
    }

    /// Create a descending sort on `column`.
    #[must_use]
    pub fn descending(column: impl Into<String>) -> Self {
        Self::new(column, SortDirection::Descending)
    }

    /// The column name being sorted.
    #[must_use]
    pub fn column(&self) -> &str {
        &self.column
    }

    /// The sort direction.
    #[must_use]
    pub fn direction(&self) -> SortDirection {
        self.direction
    }
}

impl fmt::Display for ColumnSort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.direction == SortDirection::Descending {
            f.write_str("-")?;
        }
        f.write_str(&self.column)
    }
}

/// A parsed sort specification for `LabKey` query endpoints.
///
/// Wraps a `Vec<ColumnSort>` and provides [`parse`](Self::parse) for the
/// comma-separated wire format and a [`Display`](fmt::Display) impl that
/// produces it. Parsing is infallible: empty segments are silently skipped,
/// and column name validity is left to the server.
///
/// # Wire format
///
/// The `LabKey` sort string is a comma-separated list of column names. A `-`
/// prefix indicates descending order. For example:
///
/// - `"Name"` → sort by Name ascending
/// - `"-Created"` → sort by Created descending
/// - `"Name,-Created,Age"` → multi-column sort
///
/// # Construction
///
/// ```
/// use labkey_rs::sort::{ColumnSort, QuerySort};
///
/// // From a wire-format string:
/// let sort = QuerySort::parse("Name,-Created");
///
/// // Programmatically:
/// let sort = QuerySort::from(vec![
///     ColumnSort::ascending("Name"),
///     ColumnSort::descending("Created"),
/// ]);
///
/// // Both produce the same wire string:
/// assert_eq!(sort.to_string(), "Name,-Created");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuerySort {
    columns: Vec<ColumnSort>,
}

impl QuerySort {
    /// Parse a comma-separated sort specification string.
    ///
    /// Empty segments (from leading/trailing commas or double commas) are
    /// silently skipped. A bare `-` with no column name is also skipped.
    /// Column name validity is not checked — the server is the authority
    /// on what column names are legal.
    ///
    /// # Examples
    ///
    /// ```
    /// use labkey_rs::sort::{QuerySort, SortDirection};
    ///
    /// let sort = QuerySort::parse("Name,-Created");
    /// assert_eq!(sort.columns().len(), 2);
    /// assert_eq!(sort.columns()[1].direction(), SortDirection::Descending);
    ///
    /// // Empty string produces an empty sort.
    /// let empty = QuerySort::parse("");
    /// assert!(empty.columns().is_empty());
    /// ```
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let columns = s
            .split(',')
            .filter_map(|segment| {
                let segment = segment.trim();
                if segment.is_empty() {
                    return None;
                }
                if let Some(col) = segment.strip_prefix('-') {
                    let col = col.trim();
                    if col.is_empty() {
                        return None;
                    }
                    Some(ColumnSort::descending(col))
                } else if let Some(col) = segment.strip_prefix('+') {
                    let col = col.trim();
                    if col.is_empty() {
                        return None;
                    }
                    Some(ColumnSort::ascending(col))
                } else {
                    Some(ColumnSort::ascending(segment))
                }
            })
            .collect();
        Self { columns }
    }

    /// The individual column sorts in order.
    #[must_use]
    pub fn columns(&self) -> &[ColumnSort] {
        &self.columns
    }

    /// Returns `true` if this sort specification has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// The number of columns in this sort specification.
    #[must_use]
    pub fn len(&self) -> usize {
        self.columns.len()
    }
}

impl From<Vec<ColumnSort>> for QuerySort {
    fn from(columns: Vec<ColumnSort>) -> Self {
        Self { columns }
    }
}

impl fmt::Display for QuerySort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{col}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SortDirection ---

    #[test]
    fn sort_direction_default_is_ascending() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
    }

    #[test]
    fn sort_direction_reversed() {
        assert_eq!(
            SortDirection::Ascending.reversed(),
            SortDirection::Descending
        );
        assert_eq!(
            SortDirection::Descending.reversed(),
            SortDirection::Ascending
        );
    }

    #[test]
    fn sort_direction_display_matches_wire_format() {
        assert_eq!(SortDirection::Ascending.to_string(), "ASC");
        assert_eq!(SortDirection::Descending.to_string(), "DESC");
    }

    #[test]
    fn sort_direction_serializes_to_wire_strings() {
        assert_eq!(
            serde_json::to_value(SortDirection::Ascending)
                .expect("SortDirection::Ascending should serialize to JSON"),
            serde_json::json!("ASC")
        );
        assert_eq!(
            serde_json::to_value(SortDirection::Descending)
                .expect("SortDirection::Descending should serialize to JSON"),
            serde_json::json!("DESC")
        );
    }

    #[test]
    fn sort_direction_deserializes_asc_desc_strings() {
        let asc: SortDirection = serde_json::from_value(serde_json::json!("ASC"))
            .expect("\"ASC\" should deserialize to SortDirection::Ascending");
        assert_eq!(asc, SortDirection::Ascending);

        let desc: SortDirection = serde_json::from_value(serde_json::json!("DESC"))
            .expect("\"DESC\" should deserialize to SortDirection::Descending");
        assert_eq!(desc, SortDirection::Descending);
    }

    #[test]
    fn sort_direction_deserializes_plus_minus_aliases() {
        let asc: SortDirection = serde_json::from_value(serde_json::json!("+"))
            .expect("\"+\" should deserialize to SortDirection::Ascending");
        assert_eq!(asc, SortDirection::Ascending);

        let desc: SortDirection = serde_json::from_value(serde_json::json!("-"))
            .expect("\"-\" should deserialize to SortDirection::Descending");
        assert_eq!(desc, SortDirection::Descending);
    }

    #[test]
    fn sort_direction_rejects_unknown_variant() {
        let result = serde_json::from_value::<SortDirection>(serde_json::json!("RANDOM"));
        assert!(
            result.is_err(),
            "unknown variant should fail deserialization"
        );
    }

    #[test]
    fn sort_direction_round_trips_through_serde() {
        for dir in [SortDirection::Ascending, SortDirection::Descending] {
            let json = serde_json::to_value(dir).expect("SortDirection should serialize to JSON");
            let back: SortDirection =
                serde_json::from_value(json).expect("SortDirection should round-trip through JSON");
            assert_eq!(back, dir);
        }
    }

    #[test]
    fn sort_direction_variant_count_regression() {
        let count = match SortDirection::Ascending {
            SortDirection::Ascending | SortDirection::Descending => 2,
        };
        assert_eq!(count, 2);
    }

    // --- ColumnSort ---

    #[test]
    fn column_sort_ascending_constructor() {
        let cs = ColumnSort::ascending("Name");
        assert_eq!(cs.column(), "Name");
        assert_eq!(cs.direction(), SortDirection::Ascending);
    }

    #[test]
    fn column_sort_descending_constructor() {
        let cs = ColumnSort::descending("Created");
        assert_eq!(cs.column(), "Created");
        assert_eq!(cs.direction(), SortDirection::Descending);
    }

    #[test]
    fn column_sort_display_ascending_has_no_prefix() {
        assert_eq!(ColumnSort::ascending("Name").to_string(), "Name");
    }

    #[test]
    fn column_sort_display_descending_has_minus_prefix() {
        assert_eq!(ColumnSort::descending("Age").to_string(), "-Age");
    }

    #[test]
    fn column_sort_new_with_explicit_direction() {
        let cs = ColumnSort::new("Score", SortDirection::Descending);
        assert_eq!(cs.column(), "Score");
        assert_eq!(cs.direction(), SortDirection::Descending);
    }

    // --- QuerySort ---

    #[test]
    fn parse_single_ascending_column() {
        let sort = QuerySort::parse("Name");
        assert_eq!(sort.len(), 1);
        assert_eq!(sort.columns()[0].column(), "Name");
        assert_eq!(sort.columns()[0].direction(), SortDirection::Ascending);
    }

    #[test]
    fn parse_single_descending_column() {
        let sort = QuerySort::parse("-Created");
        assert_eq!(sort.len(), 1);
        assert_eq!(sort.columns()[0].column(), "Created");
        assert_eq!(sort.columns()[0].direction(), SortDirection::Descending);
    }

    #[test]
    fn parse_multi_column_sort() {
        let sort = QuerySort::parse("Name,-Created,Age");
        assert_eq!(sort.len(), 3);
        assert_eq!(sort.columns()[0], ColumnSort::ascending("Name"));
        assert_eq!(sort.columns()[1], ColumnSort::descending("Created"));
        assert_eq!(sort.columns()[2], ColumnSort::ascending("Age"));
    }

    #[test]
    fn parse_empty_string_produces_empty_sort() {
        let sort = QuerySort::parse("");
        assert!(sort.is_empty());
        assert_eq!(sort.len(), 0);
    }

    #[test]
    fn parse_skips_empty_segments() {
        let sort = QuerySort::parse(",Name,,Age,");
        assert_eq!(sort.len(), 2);
        assert_eq!(sort.columns()[0].column(), "Name");
        assert_eq!(sort.columns()[1].column(), "Age");
    }

    #[test]
    fn parse_skips_bare_minus() {
        let sort = QuerySort::parse("-,Name");
        assert_eq!(sort.len(), 1);
        assert_eq!(sort.columns()[0].column(), "Name");
    }

    #[test]
    fn parse_handles_explicit_plus_prefix() {
        let sort = QuerySort::parse("+Name,-Age");
        assert_eq!(sort.len(), 2);
        assert_eq!(sort.columns()[0], ColumnSort::ascending("Name"));
        assert_eq!(sort.columns()[1], ColumnSort::descending("Age"));
    }

    #[test]
    fn parse_trims_whitespace_around_segments() {
        let sort = QuerySort::parse(" Name , -Created ");
        assert_eq!(sort.len(), 2);
        assert_eq!(sort.columns()[0].column(), "Name");
        assert_eq!(sort.columns()[1].column(), "Created");
    }

    #[test]
    fn display_round_trips_through_parse() {
        let original = "Name,-Created,Age";
        let sort = QuerySort::parse(original);
        assert_eq!(sort.to_string(), original);
    }

    #[test]
    fn display_empty_sort_is_empty_string() {
        let sort = QuerySort::from(vec![]);
        assert_eq!(sort.to_string(), "");
    }

    #[test]
    fn from_vec_constructs_query_sort() {
        let sort = QuerySort::from(vec![
            ColumnSort::ascending("A"),
            ColumnSort::descending("B"),
        ]);
        assert_eq!(sort.len(), 2);
        assert_eq!(sort.to_string(), "A,-B");
    }

    #[test]
    fn parse_lookup_column_with_slash() {
        let sort = QuerySort::parse("Lookup/Name,-Lookup/Created");
        assert_eq!(sort.len(), 2);
        assert_eq!(sort.columns()[0].column(), "Lookup/Name");
        assert_eq!(sort.columns()[1].column(), "Lookup/Created");
        assert_eq!(sort.to_string(), "Lookup/Name,-Lookup/Created");
    }
}
