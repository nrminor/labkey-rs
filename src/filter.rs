//! Filter types and encoding for `LabKey` query parameters.
//!
//! `LabKey` queries support a rich set of filter operators that are encoded as
//! URL query parameters. A filter on column `"Age"` using the `Equal` operator
//! with value `"25"` becomes `query.Age~eq=25` in the URL.
//!
//! Multi-valued filters (like [`FilterType::In`]) join their values with a
//! separator character. If any value itself contains the separator, the values
//! are wrapped in `{json:[...]}` syntax to avoid ambiguity.

/// The operator for a `LabKey` query filter.
///
/// Each variant corresponds to a URL suffix that the server recognizes.
/// For example, [`FilterType::Equal`] has suffix `"eq"`, so a filter on
/// column `"Age"` with value `"25"` would be encoded as `query.Age~eq=25`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FilterType {
    // Comparison operators
    Equal,
    NotEqual,
    NotEqualOrNull,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,

    // Set membership
    In,
    NotIn,

    // String matching
    Contains,
    DoesNotContain,
    StartsWith,
    DoesNotStartWith,
    ContainsOneOf,
    ContainsNoneOf,

    // Range
    Between,
    NotBetween,

    // Null/blank checks
    IsBlank,
    IsNotBlank,
    HasAnyValue,

    // Membership
    MemberOf,

    // Missing value indicators
    HasMissingValue,
    DoesNotHaveMissingValue,

    // Date-specific variants (same semantics, different URL suffixes)
    DateEqual,
    DateNotEqual,
    DateGreaterThan,
    DateGreaterThanOrEqual,
    DateLessThan,
    DateLessThanOrEqual,

    // Array operators
    ArrayContainsAll,
    ArrayContainsAny,
    ArrayContainsExact,
    ArrayContainsNotExact,
    ArrayContainsNone,
    ArrayIsEmpty,
    ArrayIsNotEmpty,

    // Table-wise search
    Q,
    /// SQL WHERE clause filter (Java-only, applies to the entire table).
    Where,

    // Ontology operators
    OntologyInSubtree,
    OntologyNotInSubtree,

    // Lineage operators
    ExpChildOf,
    ExpParentOf,
    ExpLineageOf,
}

impl FilterType {
    /// The URL suffix used when encoding this filter as a query parameter.
    ///
    /// For example, [`FilterType::Equal`] returns `"eq"`, so a filter on
    /// column `"Name"` becomes `query.Name~eq=value`.
    #[must_use]
    pub fn url_suffix(self) -> &'static str {
        match self {
            Self::Equal => "eq",
            Self::NotEqual => "neq",
            Self::NotEqualOrNull => "neqornull",
            Self::GreaterThan => "gt",
            Self::GreaterThanOrEqual => "gte",
            Self::LessThan => "lt",
            Self::LessThanOrEqual => "lte",
            Self::In => "in",
            Self::NotIn => "notin",
            Self::Contains => "contains",
            Self::DoesNotContain => "doesnotcontain",
            Self::StartsWith => "startswith",
            Self::DoesNotStartWith => "doesnotstartwith",
            Self::ContainsOneOf => "containsoneof",
            Self::ContainsNoneOf => "containsnoneof",
            Self::Between => "between",
            Self::NotBetween => "notbetween",
            Self::IsBlank => "isblank",
            Self::IsNotBlank => "isnonblank",
            Self::HasAnyValue => "",
            Self::MemberOf => "memberof",
            Self::HasMissingValue => "hasmvvalue",
            Self::DoesNotHaveMissingValue => "nomvvalue",
            Self::DateEqual => "dateeq",
            Self::DateNotEqual => "dateneq",
            Self::DateGreaterThan => "dategt",
            Self::DateGreaterThanOrEqual => "dategte",
            Self::DateLessThan => "datelt",
            Self::DateLessThanOrEqual => "datelte",
            Self::ArrayContainsAll => "arraycontainsall",
            Self::ArrayContainsAny => "arraycontainsany",
            Self::ArrayContainsExact => "arraymatches",
            Self::ArrayContainsNotExact => "arraynotmatches",
            Self::ArrayContainsNone => "arraycontainsnone",
            Self::ArrayIsEmpty => "arrayisempty",
            Self::ArrayIsNotEmpty => "arrayisnotempty",
            Self::Q => "q",
            Self::Where => "where",
            Self::OntologyInSubtree => "concept:insubtree",
            Self::OntologyNotInSubtree => "concept:notinsubtree",
            Self::ExpChildOf => "exp:childof",
            Self::ExpParentOf => "exp:parentof",
            Self::ExpLineageOf => "exp:lineageof",
        }
    }

    /// Whether this filter type requires a data value.
    ///
    /// Filters like [`FilterType::IsBlank`] and [`FilterType::HasAnyValue`]
    /// do not require a value — they apply to the column as a whole.
    #[must_use]
    pub fn requires_value(self) -> bool {
        !matches!(
            self,
            Self::IsBlank
                | Self::IsNotBlank
                | Self::HasAnyValue
                | Self::ArrayIsEmpty
                | Self::ArrayIsNotEmpty
                | Self::HasMissingValue
                | Self::DoesNotHaveMissingValue
        )
    }

    /// Whether this filter type accepts multiple values.
    #[must_use]
    pub fn is_multi_valued(self) -> bool {
        self.separator().is_some()
    }

    /// The separator character for multi-valued filters, if applicable.
    ///
    /// Semicolon-separated: [`In`](Self::In), [`NotIn`](Self::NotIn),
    /// [`ContainsOneOf`](Self::ContainsOneOf), [`ContainsNoneOf`](Self::ContainsNoneOf),
    /// and the array operators.
    ///
    /// Comma-separated: [`Between`](Self::Between), [`NotBetween`](Self::NotBetween),
    /// [`ExpLineageOf`](Self::ExpLineageOf).
    #[must_use]
    pub fn separator(self) -> Option<char> {
        match self {
            Self::In
            | Self::NotIn
            | Self::ContainsOneOf
            | Self::ContainsNoneOf
            | Self::ArrayContainsAll
            | Self::ArrayContainsAny
            | Self::ArrayContainsExact
            | Self::ArrayContainsNotExact
            | Self::ArrayContainsNone => Some(';'),
            Self::Between | Self::NotBetween | Self::ExpLineageOf => Some(','),
            _ => None,
        }
    }

    /// Whether this filter applies to the entire table rather than a
    /// specific column. Currently only [`FilterType::Q`] (search) is
    /// table-wise.
    #[must_use]
    pub fn is_table_wise(self) -> bool {
        matches!(self, Self::Q | Self::Where)
    }

    /// Look up a [`FilterType`] by its URL suffix.
    ///
    /// Returns `None` if the suffix doesn't match any known filter type.
    #[must_use]
    pub fn from_url_suffix(suffix: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|ft| ft.url_suffix() == suffix)
    }

    /// All filter type variants, for iteration and lookup.
    const ALL: &[Self] = &[
        Self::Equal,
        Self::NotEqual,
        Self::NotEqualOrNull,
        Self::GreaterThan,
        Self::GreaterThanOrEqual,
        Self::LessThan,
        Self::LessThanOrEqual,
        Self::In,
        Self::NotIn,
        Self::Contains,
        Self::DoesNotContain,
        Self::StartsWith,
        Self::DoesNotStartWith,
        Self::ContainsOneOf,
        Self::ContainsNoneOf,
        Self::Between,
        Self::NotBetween,
        Self::IsBlank,
        Self::IsNotBlank,
        Self::HasAnyValue,
        Self::MemberOf,
        Self::HasMissingValue,
        Self::DoesNotHaveMissingValue,
        Self::DateEqual,
        Self::DateNotEqual,
        Self::DateGreaterThan,
        Self::DateGreaterThanOrEqual,
        Self::DateLessThan,
        Self::DateLessThanOrEqual,
        Self::ArrayContainsAll,
        Self::ArrayContainsAny,
        Self::ArrayContainsExact,
        Self::ArrayContainsNotExact,
        Self::ArrayContainsNone,
        Self::ArrayIsEmpty,
        Self::ArrayIsNotEmpty,
        Self::Q,
        Self::Where,
        Self::OntologyInSubtree,
        Self::OntologyNotInSubtree,
        Self::ExpChildOf,
        Self::ExpParentOf,
        Self::ExpLineageOf,
    ];
}

/// The value(s) for a filter.
#[derive(Debug, Clone)]
pub enum FilterValue {
    /// No value (for filters like [`FilterType::IsBlank`]).
    None,
    /// A single value.
    Single(String),
    /// Multiple values (for [`FilterType::In`], [`FilterType::Between`], etc.).
    Multi(Vec<String>),
}

/// A filter to apply to a `LabKey` query.
///
/// Filters are encoded as URL query parameters in the form
/// `{dataRegionName}.{columnName}~{urlSuffix}={value}`.
///
/// # Examples
///
/// ```
/// use labkey_rs::filter::{Filter, FilterType, FilterValue};
///
/// // Simple equality filter
/// let f = Filter::equal("Name", "John");
/// assert_eq!(f.url_param_name("query"), "query.Name~eq");
/// assert_eq!(f.url_param_value(), "John");
///
/// // Multi-valued IN filter
/// let f = Filter::new(
///     "Status",
///     FilterType::In,
///     FilterValue::Multi(vec!["Active".into(), "Pending".into()]),
/// );
/// assert_eq!(f.url_param_value(), "Active;Pending");
/// ```
#[derive(Debug, Clone)]
pub struct Filter {
    column_name: String,
    op: FilterType,
    value: FilterValue,
}

impl Filter {
    /// Create a new filter with the given column, operator, and value.
    ///
    /// Table-wise operators like [`FilterType::Q`] ignore the column name
    /// and use `"*"` instead, matching the JS client's behavior.
    #[must_use]
    pub fn new(column_name: impl Into<String>, op: FilterType, value: FilterValue) -> Self {
        let column_name = if op.is_table_wise() {
            "*".to_string()
        } else {
            column_name.into()
        };
        Self {
            column_name,
            op,
            value,
        }
    }

    /// Convenience constructor for an equality filter.
    #[must_use]
    pub fn equal(column_name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            column_name,
            FilterType::Equal,
            FilterValue::Single(value.into()),
        )
    }

    /// The column name this filter applies to.
    ///
    /// Table-wise filters like [`FilterType::Q`] and [`FilterType::Where`]
    /// return `"*"`.
    #[must_use]
    pub fn column_name(&self) -> &str {
        &self.column_name
    }

    /// The URL parameter name for this filter (e.g., `"query.Age~eq"`).
    #[must_use]
    pub fn url_param_name(&self, data_region_name: &str) -> String {
        format!(
            "{}.{}~{}",
            data_region_name,
            self.column_name,
            self.op.url_suffix()
        )
    }

    /// The URL parameter value for this filter.
    ///
    /// For multi-valued filters, values are joined with the appropriate
    /// separator. If any value contains the separator character, the
    /// `{json:[...]}` encoding is used instead to avoid ambiguity.
    #[must_use]
    pub fn url_param_value(&self) -> String {
        match (&self.value, self.op.separator()) {
            (FilterValue::None, _) => String::new(),
            (FilterValue::Single(v), _) => v.clone(),
            (FilterValue::Multi(values), Some(sep)) => {
                let needs_json = values.iter().any(|v| v.contains(sep));
                if needs_json {
                    let json_array = serde_json::Value::Array(
                        values
                            .iter()
                            .map(|v| serde_json::Value::String(v.clone()))
                            .collect(),
                    );
                    format!("{{json:{json_array}}}")
                } else {
                    let sep_str = String::from(sep);
                    values.join(&sep_str)
                }
            }
            (FilterValue::Multi(values), None) => {
                // Multi values on a non-multi-valued filter type shouldn't
                // happen with well-formed filters, but handle gracefully.
                values.join(";")
            }
        }
    }
}

/// Encode an array of filters into URL query parameter key-value pairs.
///
/// Filters that require a value but have [`FilterValue::None`] are skipped,
/// since they would be no-ops on the server.
#[must_use]
pub fn encode_filters(filters: &[Filter], data_region_name: &str) -> Vec<(String, String)> {
    filters
        .iter()
        .filter(|f| {
            if f.op.requires_value() {
                !matches!(f.value, FilterValue::None)
            } else {
                true
            }
        })
        .map(|f| (f.url_param_name(data_region_name), f.url_param_value()))
        .collect()
}

/// Merge filters by replacing all filters on a given column.
///
/// Returns a new filter list containing every filter from `base_filters`
/// whose column name does not match `column_name`, followed by all filters
/// from `column_filters`. This matches the JS `Filter.merge` function.
///
/// # Examples
///
/// ```
/// use labkey_rs::filter::{Filter, FilterType, merge};
///
/// let base = vec![
///     Filter::equal("Name", "Alice"),
///     Filter::equal("Age", "30"),
/// ];
/// let replacement = vec![Filter::equal("Name", "Bob")];
/// let merged = merge(&base, "Name", &replacement);
///
/// assert_eq!(merged.len(), 2);
/// assert_eq!(merged[0].column_name(), "Age");
/// assert_eq!(merged[1].column_name(), "Name");
/// ```
#[must_use]
pub fn merge(base_filters: &[Filter], column_name: &str, column_filters: &[Filter]) -> Vec<Filter> {
    let mut result: Vec<Filter> = base_filters
        .iter()
        .filter(|f| f.column_name() != column_name)
        .cloned()
        .collect();
    result.extend(column_filters.iter().cloned());
    result
}

/// Container filter scope for queries.
///
/// Controls which containers' data is included in query results. Not all
/// data types support cross-container queries; in those cases all values
/// behave the same as [`Current`](ContainerFilter::Current).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum ContainerFilter {
    /// Include all folders for which the user has read permission.
    AllFolders,
    /// Include the current project and all folders in it.
    AllInProject,
    /// Include the current project, all folders in it, and the Shared project.
    AllInProjectPlusShared,
    /// Include the current folder only.
    Current,
    /// Include the current folder and its direct children (excluding workbooks).
    CurrentAndFirstChildren,
    /// Include the current folder and its parent folders.
    CurrentAndParents,
    /// Include the current folder and all subfolders.
    CurrentAndSubfolders,
    /// Include the current folder, all subfolders, and the Shared folder.
    CurrentAndSubfoldersPlusShared,
    /// Include the current folder and the project that contains it.
    CurrentPlusProject,
    /// Include the current folder, its project, and any shared folders.
    CurrentPlusProjectAndShared,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_suffix_round_trips_through_from_url_suffix() {
        for ft in FilterType::ALL {
            let suffix = ft.url_suffix();
            let recovered = FilterType::from_url_suffix(suffix);
            // HasAnyValue has empty suffix which collides with nothing,
            // but from_url_suffix("") should still find it.
            assert_eq!(
                recovered,
                Some(*ft),
                "round-trip failed for {ft:?} with suffix {suffix:?}"
            );
        }
    }

    #[test]
    fn from_url_suffix_returns_none_for_unknown() {
        assert!(FilterType::from_url_suffix("nonexistent").is_none());
    }

    #[test]
    fn all_array_contains_variants_are_semicolon_separated() {
        let array_types = [
            FilterType::ArrayContainsAll,
            FilterType::ArrayContainsAny,
            FilterType::ArrayContainsExact,
            FilterType::ArrayContainsNotExact,
            FilterType::ArrayContainsNone,
        ];
        for ft in array_types {
            assert_eq!(ft.separator(), Some(';'), "{ft:?} should use semicolon");
            assert!(ft.is_multi_valued(), "{ft:?} should be multi-valued");
        }
    }

    #[test]
    fn between_variants_are_comma_separated() {
        assert_eq!(FilterType::Between.separator(), Some(','));
        assert_eq!(FilterType::NotBetween.separator(), Some(','));
    }

    #[test]
    fn no_value_filter_types() {
        let no_value_types = [
            FilterType::IsBlank,
            FilterType::IsNotBlank,
            FilterType::HasAnyValue,
            FilterType::ArrayIsEmpty,
            FilterType::ArrayIsNotEmpty,
            FilterType::HasMissingValue,
            FilterType::DoesNotHaveMissingValue,
        ];
        for ft in no_value_types {
            assert!(!ft.requires_value(), "{ft:?} should not require a value");
        }
    }

    #[test]
    fn value_required_filter_types() {
        let value_types = [
            FilterType::Equal,
            FilterType::NotEqual,
            FilterType::GreaterThan,
            FilterType::In,
            FilterType::Contains,
            FilterType::Between,
            FilterType::Q,
        ];
        for ft in value_types {
            assert!(ft.requires_value(), "{ft:?} should require a value");
        }
    }

    #[test]
    fn equal_filter_encoding() {
        let f = Filter::equal("Name", "John");
        assert_eq!(f.url_param_name("query"), "query.Name~eq");
        assert_eq!(f.url_param_value(), "John");
    }

    #[test]
    fn in_filter_with_semicolon_separated_values() {
        let f = Filter::new(
            "Status",
            FilterType::In,
            FilterValue::Multi(vec!["Active".into(), "Pending".into(), "Closed".into()]),
        );
        assert_eq!(f.url_param_name("query"), "query.Status~in");
        assert_eq!(f.url_param_value(), "Active;Pending;Closed");
    }

    #[test]
    fn in_filter_uses_json_encoding_when_value_contains_separator() {
        let f = Filter::new(
            "Tags",
            FilterType::In,
            FilterValue::Multi(vec!["a;b".into(), "c".into()]),
        );
        let value = f.url_param_value();
        assert!(
            value.starts_with("{json:"),
            "should use json encoding, got: {value}"
        );
        assert!(value.ends_with('}'));
        // The inner JSON should be a valid array
        let inner = &value["{json:".len()..value.len() - 1];
        let parsed: Vec<String> =
            serde_json::from_str(inner).expect("inner JSON should be a valid array");
        assert_eq!(parsed, vec!["a;b", "c"]);
    }

    #[test]
    fn between_filter_with_comma_separated_values() {
        let f = Filter::new(
            "Age",
            FilterType::Between,
            FilterValue::Multi(vec!["18".into(), "65".into()]),
        );
        assert_eq!(f.url_param_name("query"), "query.Age~between");
        assert_eq!(f.url_param_value(), "18,65");
    }

    #[test]
    fn has_missing_value_filter_has_empty_value() {
        let f = Filter::new("Col", FilterType::HasMissingValue, FilterValue::None);
        assert_eq!(f.url_param_name("query"), "query.Col~hasmvvalue");
        assert_eq!(f.url_param_value(), "");
    }

    #[test]
    fn is_blank_filter_has_empty_value() {
        let f = Filter::new("Notes", FilterType::IsBlank, FilterValue::None);
        assert_eq!(f.url_param_name("query"), "query.Notes~isblank");
        assert_eq!(f.url_param_value(), "");
    }

    #[test]
    fn has_any_value_filter_has_empty_suffix() {
        let f = Filter::new("Col", FilterType::HasAnyValue, FilterValue::None);
        assert_eq!(f.url_param_name("query"), "query.Col~");
        assert_eq!(f.url_param_value(), "");
    }

    #[test]
    fn encode_filters_produces_correct_pairs() {
        let filters = vec![
            Filter::equal("Name", "Alice"),
            Filter::new(
                "Age",
                FilterType::GreaterThan,
                FilterValue::Single("21".into()),
            ),
        ];
        let pairs = encode_filters(&filters, "query");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("query.Name~eq".into(), "Alice".into()));
        assert_eq!(pairs[1], ("query.Age~gt".into(), "21".into()));
    }

    #[test]
    fn encode_filters_skips_value_required_filter_with_no_value() {
        let filters = vec![
            Filter::equal("Name", "Alice"),
            Filter::new("Age", FilterType::GreaterThan, FilterValue::None),
        ];
        let pairs = encode_filters(&filters, "query");
        assert_eq!(
            pairs.len(),
            1,
            "should skip the no-value GreaterThan filter"
        );
        assert_eq!(pairs[0].0, "query.Name~eq");
    }

    #[test]
    fn encode_filters_keeps_no_value_filter_types() {
        let filters = vec![Filter::new("Notes", FilterType::IsBlank, FilterValue::None)];
        let pairs = encode_filters(&filters, "query");
        assert_eq!(pairs.len(), 1, "IsBlank should not be skipped");
        assert_eq!(pairs[0].0, "query.Notes~isblank");
    }

    #[test]
    fn encode_filters_with_custom_data_region() {
        let filters = vec![Filter::equal("Col", "val")];
        let pairs = encode_filters(&filters, "myRegion");
        assert_eq!(pairs[0].0, "myRegion.Col~eq");
    }

    #[test]
    fn q_filter_forces_column_to_star() {
        let f = Filter::new(
            "IgnoredColumn",
            FilterType::Q,
            FilterValue::Single("term".into()),
        );
        assert_eq!(f.url_param_name("query"), "query.*~q");
        assert_eq!(f.url_param_value(), "term");
    }

    #[test]
    fn multi_value_on_non_multi_filter_falls_back_to_semicolon() {
        // This shouldn't happen with well-formed filters, but the code
        // handles it gracefully by joining with semicolons.
        let f = Filter::new(
            "Col",
            FilterType::Equal,
            FilterValue::Multi(vec!["a".into(), "b".into()]),
        );
        assert_eq!(f.url_param_value(), "a;b");
    }

    #[test]
    fn empty_multi_vec_produces_empty_string() {
        let f = Filter::new("Col", FilterType::In, FilterValue::Multi(vec![]));
        assert_eq!(f.url_param_value(), "");
    }

    #[test]
    fn empty_single_value_produces_empty_string() {
        let f = Filter::new("Col", FilterType::Equal, FilterValue::Single(String::new()));
        assert_eq!(f.url_param_value(), "");
    }

    #[test]
    fn single_element_multi_value() {
        let f = Filter::new(
            "Col",
            FilterType::In,
            FilterValue::Multi(vec!["only".into()]),
        );
        // No separator needed for a single element.
        assert_eq!(f.url_param_value(), "only");
    }

    #[test]
    fn between_json_encoding_contains_correct_values() {
        let f = Filter::new(
            "Amount",
            FilterType::Between,
            FilterValue::Multi(vec!["1,000".into(), "5,000".into()]),
        );
        let value = f.url_param_value();
        assert!(value.starts_with("{json:"));
        assert!(value.ends_with('}'));
        let inner = &value["{json:".len()..value.len() - 1];
        let parsed: Vec<String> =
            serde_json::from_str(inner).expect("inner JSON should be a valid array");
        assert_eq!(parsed, vec!["1,000", "5,000"]);
    }

    #[test]
    fn encode_filters_empty_array_returns_empty() {
        let pairs = encode_filters(&[], "query");
        assert!(pairs.is_empty());
    }

    /// Exhaustive variant counter. Adding a new `FilterType` variant without
    /// updating this function causes a compile error because there is no
    /// wildcard arm.
    fn variant_count(ft: FilterType) -> usize {
        match ft {
            FilterType::Equal
            | FilterType::NotEqual
            | FilterType::NotEqualOrNull
            | FilterType::GreaterThan
            | FilterType::GreaterThanOrEqual
            | FilterType::LessThan
            | FilterType::LessThanOrEqual
            | FilterType::In
            | FilterType::NotIn
            | FilterType::Contains
            | FilterType::DoesNotContain
            | FilterType::StartsWith
            | FilterType::DoesNotStartWith
            | FilterType::ContainsOneOf
            | FilterType::ContainsNoneOf
            | FilterType::Between
            | FilterType::NotBetween
            | FilterType::IsBlank
            | FilterType::IsNotBlank
            | FilterType::HasAnyValue
            | FilterType::MemberOf
            | FilterType::HasMissingValue
            | FilterType::DoesNotHaveMissingValue
            | FilterType::DateEqual
            | FilterType::DateNotEqual
            | FilterType::DateGreaterThan
            | FilterType::DateGreaterThanOrEqual
            | FilterType::DateLessThan
            | FilterType::DateLessThanOrEqual
            | FilterType::ArrayContainsAll
            | FilterType::ArrayContainsAny
            | FilterType::ArrayContainsExact
            | FilterType::ArrayContainsNotExact
            | FilterType::ArrayContainsNone
            | FilterType::ArrayIsEmpty
            | FilterType::ArrayIsNotEmpty
            | FilterType::Q
            | FilterType::Where
            | FilterType::OntologyInSubtree
            | FilterType::OntologyNotInSubtree
            | FilterType::ExpChildOf
            | FilterType::ExpParentOf
            | FilterType::ExpLineageOf => 43,
        }
    }

    #[test]
    fn all_array_covers_every_variant() {
        // variant_count() has an exhaustive match with no wildcard, so adding
        // a new FilterType variant without updating it causes a compile error.
        // The assertion here catches the opposite: a variant listed in the
        // match but missing from (or duplicated in) ALL.
        assert_eq!(
            variant_count(FilterType::Equal),
            FilterType::ALL.len(),
            "ALL array length doesn't match variant count — update ALL or variant_count()"
        );
        // Also verify ALL has no duplicates.
        let mut seen = std::collections::HashSet::new();
        for ft in FilterType::ALL {
            assert!(seen.insert(ft), "duplicate in ALL: {ft:?}");
        }
    }

    #[test]
    fn where_filter_is_table_wise_and_uses_star_column() {
        let f = Filter::new(
            "ignored",
            FilterType::Where,
            FilterValue::Single("x > 5".into()),
        );
        assert_eq!(f.column_name(), "*");
        assert_eq!(f.url_param_name("query"), "query.*~where");
        assert_eq!(f.url_param_value(), "x > 5");
        assert!(FilterType::Where.is_table_wise());
        assert!(FilterType::Where.requires_value());
        assert!(!FilterType::Where.is_multi_valued());
    }

    #[test]
    fn filter_type_url_suffix_snapshot() {
        // Exhaustive snapshot of every FilterType's URL suffix, separator,
        // requires_value, and is_table_wise. Adding or changing a variant
        // will cause this test to fail, forcing an explicit review.
        let expected: Vec<(&str, Option<char>, bool, bool)> = vec![
            ("eq", None, true, false),
            ("neq", None, true, false),
            ("neqornull", None, true, false),
            ("gt", None, true, false),
            ("gte", None, true, false),
            ("lt", None, true, false),
            ("lte", None, true, false),
            ("in", Some(';'), true, false),
            ("notin", Some(';'), true, false),
            ("contains", None, true, false),
            ("doesnotcontain", None, true, false),
            ("startswith", None, true, false),
            ("doesnotstartwith", None, true, false),
            ("containsoneof", Some(';'), true, false),
            ("containsnoneof", Some(';'), true, false),
            ("between", Some(','), true, false),
            ("notbetween", Some(','), true, false),
            ("isblank", None, false, false),
            ("isnonblank", None, false, false),
            ("", None, false, false),
            ("memberof", None, true, false),
            ("hasmvvalue", None, false, false),
            ("nomvvalue", None, false, false),
            ("dateeq", None, true, false),
            ("dateneq", None, true, false),
            ("dategt", None, true, false),
            ("dategte", None, true, false),
            ("datelt", None, true, false),
            ("datelte", None, true, false),
            ("arraycontainsall", Some(';'), true, false),
            ("arraycontainsany", Some(';'), true, false),
            ("arraymatches", Some(';'), true, false),
            ("arraynotmatches", Some(';'), true, false),
            ("arraycontainsnone", Some(';'), true, false),
            ("arrayisempty", None, false, false),
            ("arrayisnotempty", None, false, false),
            ("q", None, true, true),
            ("where", None, true, true),
            ("concept:insubtree", None, true, false),
            ("concept:notinsubtree", None, true, false),
            ("exp:childof", None, true, false),
            ("exp:parentof", None, true, false),
            ("exp:lineageof", Some(','), true, false),
        ];

        assert_eq!(
            FilterType::ALL.len(),
            expected.len(),
            "snapshot length mismatch — update expected list when adding variants"
        );

        for (ft, (suffix, sep, requires_val, table_wise)) in
            FilterType::ALL.iter().zip(expected.iter())
        {
            assert_eq!(ft.url_suffix(), *suffix, "url_suffix mismatch for {ft:?}");
            assert_eq!(ft.separator(), *sep, "separator mismatch for {ft:?}");
            assert_eq!(
                ft.requires_value(),
                *requires_val,
                "requires_value mismatch for {ft:?}"
            );
            assert_eq!(
                ft.is_table_wise(),
                *table_wise,
                "is_table_wise mismatch for {ft:?}"
            );
        }
    }

    #[test]
    fn merge_replaces_column_filters() {
        let base = vec![
            Filter::equal("Name", "Alice"),
            Filter::equal("Age", "30"),
            Filter::equal("Name", "Bob"),
        ];
        let replacement = vec![Filter::equal("Name", "Charlie")];
        let merged = merge(&base, "Name", &replacement);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].column_name(), "Age");
        assert_eq!(merged[0].url_param_value(), "30");
        assert_eq!(merged[1].column_name(), "Name");
        assert_eq!(merged[1].url_param_value(), "Charlie");
    }

    #[test]
    fn merge_with_empty_replacement_removes_column() {
        let base = vec![Filter::equal("Name", "Alice"), Filter::equal("Age", "30")];
        let merged = merge(&base, "Name", &[]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].column_name(), "Age");
    }

    #[test]
    fn merge_with_no_matching_column_preserves_all() {
        let base = vec![Filter::equal("Name", "Alice"), Filter::equal("Age", "30")];
        let replacement = vec![Filter::equal("Status", "Active")];
        let merged = merge(&base, "Status", &replacement);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].column_name(), "Name");
        assert_eq!(merged[1].column_name(), "Age");
        assert_eq!(merged[2].column_name(), "Status");
    }

    #[test]
    fn merge_with_empty_base_returns_column_filters() {
        let replacement = vec![Filter::equal("Name", "Alice")];
        let merged = merge(&[], "Name", &replacement);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].url_param_value(), "Alice");
    }

    #[test]
    fn container_filter_serializes_to_expected_string() {
        let cf = ContainerFilter::CurrentAndSubfolders;
        let json = serde_json::to_value(cf).expect("should serialize");
        assert_eq!(json.as_str(), Some("CurrentAndSubfolders"));
    }

    #[test]
    fn container_filter_deserializes_from_string() {
        let cf: ContainerFilter =
            serde_json::from_str(r#""AllInProjectPlusShared""#).expect("should deserialize");
        assert_eq!(cf, ContainerFilter::AllInProjectPlusShared);
    }

    #[test]
    fn container_filter_round_trips_all_variants() {
        let variants = [
            ContainerFilter::AllFolders,
            ContainerFilter::AllInProject,
            ContainerFilter::AllInProjectPlusShared,
            ContainerFilter::Current,
            ContainerFilter::CurrentAndFirstChildren,
            ContainerFilter::CurrentAndParents,
            ContainerFilter::CurrentAndSubfolders,
            ContainerFilter::CurrentAndSubfoldersPlusShared,
            ContainerFilter::CurrentPlusProject,
            ContainerFilter::CurrentPlusProjectAndShared,
        ];
        for cf in variants {
            let json = serde_json::to_string(&cf).expect("should serialize");
            let recovered: ContainerFilter =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(recovered, cf, "round-trip failed for {cf:?}");
        }
    }
}
