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

    /// Human-readable display text for this filter type.
    ///
    /// Returns labels like `"Equals"`, `"Is Greater Than"`, or
    /// `"Does Not Contain"`. These match the `displayText` property from
    /// the upstream JS client and the `displayValue` field from the Java
    /// client's `Filter.Operator` enum.
    ///
    /// # Examples
    ///
    /// ```
    /// use labkey_rs::filter::FilterType;
    ///
    /// assert_eq!(FilterType::Equal.display_text(), "Equals");
    /// assert_eq!(FilterType::GreaterThan.display_text(), "Is Greater Than");
    /// assert_eq!(FilterType::IsBlank.display_text(), "Is Blank");
    /// ```
    #[must_use]
    pub fn display_text(self) -> &'static str {
        match self {
            Self::Equal | Self::DateEqual => "Equals",
            Self::NotEqual | Self::DateNotEqual | Self::NotEqualOrNull => "Does Not Equal",
            Self::GreaterThan | Self::DateGreaterThan => "Is Greater Than",
            Self::GreaterThanOrEqual | Self::DateGreaterThanOrEqual => {
                "Is Greater Than or Equal To"
            }
            Self::LessThan | Self::DateLessThan => "Is Less Than",
            Self::LessThanOrEqual | Self::DateLessThanOrEqual => "Is Less Than or Equal To",
            Self::In => "Equals One Of",
            Self::NotIn => "Does Not Equal Any Of",
            Self::Contains => "Contains",
            Self::DoesNotContain => "Does Not Contain",
            Self::StartsWith => "Starts With",
            Self::DoesNotStartWith => "Does Not Start With",
            Self::ContainsOneOf => "Contains One Of",
            Self::ContainsNoneOf => "Does Not Contain Any Of",
            Self::Between => "Between",
            Self::NotBetween => "Not Between",
            Self::IsBlank => "Is Blank",
            Self::IsNotBlank => "Is Not Blank",
            Self::HasAnyValue => "Has Any Value",
            Self::MemberOf => "Member Of",
            Self::HasMissingValue => "Has a missing value indicator",
            Self::DoesNotHaveMissingValue => "Does not have a missing value indicator",
            Self::ArrayContainsAll => "Contains All",
            Self::ArrayContainsAny => "Contains Any",
            Self::ArrayContainsExact => "Contains Exactly",
            Self::ArrayContainsNotExact => "Does Not Contain Exactly",
            Self::ArrayContainsNone => "Contains None",
            Self::ArrayIsEmpty => "Is Empty",
            Self::ArrayIsNotEmpty => "Is Not Empty",
            Self::Q => "Search",
            Self::Where => "Where",
            Self::OntologyInSubtree => "Is In Subtree",
            Self::OntologyNotInSubtree => "Is Not In Subtree",
            Self::ExpChildOf => "Is Child Of",
            Self::ExpParentOf => "Is Parent Of",
            Self::ExpLineageOf => "In The Lineage Of",
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

    /// The canonical programmatic name for this filter type.
    ///
    /// Returns the `LabKey` cross-client canonical name as used by the Java
    /// client's `Filter.Operator.getProgrammaticName()`. These names appear
    /// in server configurations and are the primary keys accepted by
    /// [`from_name`](Self::from_name).
    ///
    /// # Examples
    ///
    /// ```
    /// use labkey_rs::filter::FilterType;
    ///
    /// assert_eq!(FilterType::Equal.programmatic_name(), "EQUAL");
    /// assert_eq!(FilterType::IsBlank.programmatic_name(), "MISSING");
    /// assert_eq!(FilterType::GreaterThan.programmatic_name(), "GREATER_THAN");
    /// ```
    #[must_use]
    pub fn programmatic_name(self) -> &'static str {
        match self {
            Self::Equal => "EQUAL",
            Self::NotEqual => "NOT_EQUAL",
            Self::NotEqualOrNull => "NOT_EQUAL_OR_MISSING",
            Self::GreaterThan => "GREATER_THAN",
            Self::GreaterThanOrEqual => "GREATER_THAN_OR_EQUAL",
            Self::LessThan => "LESS_THAN",
            Self::LessThanOrEqual => "LESS_THAN_OR_EQUAL",
            Self::In => "IN",
            Self::NotIn => "NOT_IN",
            Self::Contains => "CONTAINS",
            Self::DoesNotContain => "DOES_NOT_CONTAIN",
            Self::StartsWith => "STARTS_WITH",
            Self::DoesNotStartWith => "DOES_NOT_START_WITH",
            Self::ContainsOneOf => "CONTAINS_ONE_OF",
            Self::ContainsNoneOf => "CONTAINS_NONE_OF",
            Self::Between => "BETWEEN",
            Self::NotBetween => "NOT_BETWEEN",
            Self::IsBlank => "MISSING",
            Self::IsNotBlank => "NOT_MISSING",
            Self::HasAnyValue => "HAS_ANY_VALUE",
            Self::MemberOf => "MEMBER_OF",
            Self::HasMissingValue => "MV_INDICATOR",
            Self::DoesNotHaveMissingValue => "NO_MV_INDICATOR",
            Self::DateEqual => "DATE_EQUAL",
            Self::DateNotEqual => "DATE_NOT_EQUAL",
            Self::DateGreaterThan => "DATE_GREATER_THAN",
            Self::DateGreaterThanOrEqual => "DATE_GREATER_THAN_OR_EQUAL",
            Self::DateLessThan => "DATE_LESS_THAN",
            Self::DateLessThanOrEqual => "DATE_LESS_THAN_OR_EQUAL",
            Self::ArrayContainsAll => "ARRAY_CONTAINS_ALL",
            Self::ArrayContainsAny => "ARRAY_CONTAINS_ANY",
            Self::ArrayContainsExact => "ARRAY_CONTAINS_EXACT",
            Self::ArrayContainsNotExact => "ARRAY_CONTAINS_NOT_EXACT",
            Self::ArrayContainsNone => "ARRAY_CONTAINS_NONE",
            Self::ArrayIsEmpty => "ARRAY_ISEMPTY",
            Self::ArrayIsNotEmpty => "ARRAY_ISNOTEMPTY",
            Self::Q => "Q",
            Self::Where => "WHERE",
            Self::OntologyInSubtree => "ONTOLOGY_IN_SUBTREE",
            Self::OntologyNotInSubtree => "ONTOLOGY_NOT_IN_SUBTREE",
            Self::ExpChildOf => "EXP_CHILD_OF",
            Self::ExpParentOf => "EXP_PARENT_OF",
            Self::ExpLineageOf => "EXP_LINEAGE_OF",
        }
    }

    /// Look up a [`FilterType`] by its `LabKey` programmatic name.
    ///
    /// Accepts the canonical names returned by
    /// [`programmatic_name`](Self::programmatic_name) (e.g., `"EQUAL"`,
    /// `"GREATER_THAN"`, `"MISSING"`). Also accepts the JS `Types` object
    /// keys where they differ from the Java names (e.g., `"ISBLANK"` as an
    /// alias for `"MISSING"`).
    ///
    /// Returns `None` if the name doesn't match any known filter type.
    ///
    /// # Examples
    ///
    /// ```
    /// use labkey_rs::filter::FilterType;
    ///
    /// // Java programmatic name
    /// assert_eq!(FilterType::from_name("GREATER_THAN"), Some(FilterType::GreaterThan));
    ///
    /// // JS alias
    /// assert_eq!(FilterType::from_name("ISBLANK"), Some(FilterType::IsBlank));
    ///
    /// // Unknown name
    /// assert_eq!(FilterType::from_name("NOPE"), None);
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        // Primary: check canonical programmatic names.
        let primary = Self::ALL
            .iter()
            .copied()
            .find(|ft| ft.programmatic_name() == name);
        if primary.is_some() {
            return primary;
        }

        // Secondary: JS Types keys that differ from Java programmatic names.
        match name {
            "NEQ_OR_NULL" => Some(Self::NotEqualOrNull),
            "ISBLANK" => Some(Self::IsBlank),
            "NONBLANK" => Some(Self::IsNotBlank),
            "HAS_MISSING_VALUE" => Some(Self::HasMissingValue),
            "DOES_NOT_HAVE_MISSING_VALUE" => Some(Self::DoesNotHaveMissingValue),
            _ => None,
        }
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

    /// The filter operator.
    #[must_use]
    pub fn filter_type(&self) -> FilterType {
        self.op
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

/// Build a human-readable description of a list of filters.
///
/// Joins each filter's [`display_text`](FilterType::display_text) and value
/// with `" AND "`, producing strings like
/// `"Is Greater Than 10 AND Is Less Than 100"`.
///
/// This is the Rust equivalent of the JS client's `getFilterDescription`
/// function, but operates on an already-parsed filter slice rather than a
/// raw URL.
///
/// # Examples
///
/// ```
/// use labkey_rs::filter::{Filter, FilterType, FilterValue, description};
///
/// let filters = vec![
///     Filter::new("Age", FilterType::GreaterThan, FilterValue::Single("10".into())),
///     Filter::new("Age", FilterType::LessThan, FilterValue::Single("100".into())),
/// ];
/// assert_eq!(
///     description(&filters),
///     "Is Greater Than 10 AND Is Less Than 100",
/// );
/// ```
#[must_use]
pub fn description(filters: &[Filter]) -> String {
    filters
        .iter()
        .map(|f| {
            let text = f.filter_type().display_text();
            let value = f.url_param_value();
            if value.is_empty() {
                text.to_string()
            } else {
                format!("{text} {value}")
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

/// Parse filter parameters from key-value pairs back into [`Filter`] objects.
///
/// This is the inverse of [`encode_filters`]: given an iterator of
/// `(key, value)` parameter pairs and a data region name, it finds entries
/// matching the `{dataRegionName}.{column}~{suffix}` pattern, looks up the
/// filter type by URL suffix, and reconstructs the corresponding filters.
///
/// Parameters whose suffix doesn't match a known [`FilterType`] are
/// silently skipped, matching the JS client's behavior.
///
/// # Examples
///
/// ```
/// use labkey_rs::filter::{Filter, FilterType, from_parameters, encode_filters};
///
/// // Round-trip: encode then parse back
/// let original = vec![
///     Filter::equal("Name", "Alice"),
///     Filter::new(
///         "Age",
///         FilterType::GreaterThan,
///         labkey_rs::filter::FilterValue::Single("21".into()),
///     ),
/// ];
/// let params = encode_filters(&original, "query");
/// let recovered: Vec<Filter> = from_parameters(
///     params.iter().map(|(k, v)| (k.as_str(), v.as_str())),
///     "query",
/// );
/// assert_eq!(recovered.len(), 2);
/// assert_eq!(recovered[0].column_name(), "Name");
/// assert_eq!(recovered[1].column_name(), "Age");
/// ```
pub fn from_parameters<'a>(
    params: impl IntoIterator<Item = (&'a str, &'a str)>,
    data_region_name: &str,
) -> Vec<Filter> {
    let prefix = format!("{data_region_name}.");
    let mut filters = Vec::new();

    for (key, value) in params {
        let Some(rest) = key.strip_prefix(&prefix) else {
            continue;
        };
        let Some(tilde_pos) = rest.find('~') else {
            continue;
        };
        let column_name = &rest[..tilde_pos];
        let suffix = &rest[tilde_pos + 1..];

        let Some(filter_type) = FilterType::from_url_suffix(suffix) else {
            continue;
        };

        let filter_value = if !filter_type.requires_value() {
            FilterValue::None
        } else if filter_type.is_multi_valued() {
            parse_multi_value(value, filter_type)
        } else {
            FilterValue::Single(value.to_string())
        };

        filters.push(Filter::new(column_name, filter_type, filter_value));
    }

    filters
}

/// Parse a multi-valued filter parameter string into a [`FilterValue::Multi`].
///
/// Handles both the `{json:[...]}` encoding and plain separator-delimited
/// values.
fn parse_multi_value(value: &str, filter_type: FilterType) -> FilterValue {
    if value.starts_with("{json:") && value.ends_with('}') {
        let inner = &value["{json:".len()..value.len() - 1];
        if let Ok(values) = serde_json::from_str::<Vec<String>>(inner) {
            return FilterValue::Multi(values);
        }
    }

    if let Some(sep) = filter_type.separator() {
        let values: Vec<String> = value.split(sep).map(String::from).collect();
        FilterValue::Multi(values)
    } else {
        FilterValue::Single(value.to_string())
    }
}

/// The JSON type of a `LabKey` column, as reported in query metadata.
///
/// `LabKey` uses these type strings in column metadata responses to
/// describe the data type of a column. They determine which
/// [`FilterType`] operators are applicable via
/// [`filter_types_for_column_type`] and [`default_filter_for_column_type`].
///
/// The variants match the `JsonType` type from the JS client's `Types.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum JsonColumnType {
    /// Array/multi-value column.
    Array,
    /// Boolean column (`true`/`false`).
    Boolean,
    /// Date or date-time column.
    Date,
    /// Floating-point numeric column.
    Float,
    /// Integer numeric column.
    Int,
    /// Text/string column.
    String,
    /// Time-of-day column (no date component).
    Time,
}

impl JsonColumnType {
    /// Parse a `LabKey` JSON type string into a [`JsonColumnType`].
    ///
    /// Matching is case-insensitive, following the JS client's behavior
    /// of calling `toLowerCase()` on the type string before lookup.
    /// Also accepts common aliases like `"datetime"` for [`Date`](Self::Date)
    /// and `"double"` for [`Float`](Self::Float).
    ///
    /// Returns `None` for unrecognized type strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use labkey_rs::filter::JsonColumnType;
    ///
    /// assert_eq!(JsonColumnType::from_type_string("int"), Some(JsonColumnType::Int));
    /// assert_eq!(JsonColumnType::from_type_string("STRING"), Some(JsonColumnType::String));
    /// assert_eq!(JsonColumnType::from_type_string("unknown"), None);
    /// ```
    #[must_use]
    pub fn from_type_string(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "array" => Some(Self::Array),
            "boolean" => Some(Self::Boolean),
            "date" | "datetime" => Some(Self::Date),
            "float" | "double" => Some(Self::Float),
            "int" | "integer" => Some(Self::Int),
            "string" => Some(Self::String),
            "time" => Some(Self::Time),
            _ => None,
        }
    }
}

/// Returns the applicable [`FilterType`] operators for a given column type.
///
/// This encodes the same domain knowledge as the JS client's
/// `TYPES_BY_JSON_TYPE` mapping: which filter operators make sense for
/// each data type. For example, string columns support `Contains` and
/// `StartsWith`, while integer columns support `Between` but not
/// `Contains`.
///
/// When `mv_enabled` is `true`, [`FilterType::HasMissingValue`] and
/// [`FilterType::DoesNotHaveMissingValue`] are appended to the list,
/// matching the JS client's `getFilterTypesForType` behavior.
///
/// # Examples
///
/// ```
/// use labkey_rs::filter::{FilterType, JsonColumnType, filter_types_for_column_type};
///
/// let types = filter_types_for_column_type(JsonColumnType::Int, false);
/// assert!(types.contains(&FilterType::Equal));
/// assert!(types.contains(&FilterType::Between));
/// assert!(!types.contains(&FilterType::Contains));
/// ```
#[must_use]
pub fn filter_types_for_column_type(
    column_type: JsonColumnType,
    mv_enabled: bool,
) -> Vec<FilterType> {
    let mut types = match column_type {
        JsonColumnType::Array => vec![
            FilterType::ArrayContainsAll,
            FilterType::ArrayContainsAny,
            FilterType::ArrayContainsExact,
            FilterType::ArrayContainsNone,
            FilterType::ArrayContainsNotExact,
            FilterType::ArrayIsEmpty,
            FilterType::ArrayIsNotEmpty,
        ],
        JsonColumnType::Boolean => vec![
            FilterType::HasAnyValue,
            FilterType::Equal,
            FilterType::NotEqualOrNull,
            FilterType::IsBlank,
            FilterType::IsNotBlank,
        ],
        JsonColumnType::Date => vec![
            FilterType::DateEqual,
            FilterType::DateNotEqual,
            FilterType::IsBlank,
            FilterType::IsNotBlank,
            FilterType::DateGreaterThan,
            FilterType::DateLessThan,
            FilterType::DateGreaterThanOrEqual,
            FilterType::DateLessThanOrEqual,
        ],
        JsonColumnType::Time => vec![
            FilterType::Equal,
            FilterType::NotEqualOrNull,
            FilterType::IsBlank,
            FilterType::IsNotBlank,
            FilterType::GreaterThan,
            FilterType::LessThan,
            FilterType::GreaterThanOrEqual,
            FilterType::LessThanOrEqual,
            FilterType::Between,
            FilterType::NotBetween,
        ],
        JsonColumnType::Float | JsonColumnType::Int => vec![
            FilterType::HasAnyValue,
            FilterType::Equal,
            FilterType::NotEqualOrNull,
            FilterType::IsBlank,
            FilterType::IsNotBlank,
            FilterType::GreaterThan,
            FilterType::LessThan,
            FilterType::GreaterThanOrEqual,
            FilterType::LessThanOrEqual,
            FilterType::In,
            FilterType::NotIn,
            FilterType::Between,
            FilterType::NotBetween,
        ],
        JsonColumnType::String => vec![
            FilterType::HasAnyValue,
            FilterType::Equal,
            FilterType::NotEqualOrNull,
            FilterType::IsBlank,
            FilterType::IsNotBlank,
            FilterType::GreaterThan,
            FilterType::LessThan,
            FilterType::GreaterThanOrEqual,
            FilterType::LessThanOrEqual,
            FilterType::Contains,
            FilterType::DoesNotContain,
            FilterType::DoesNotStartWith,
            FilterType::StartsWith,
            FilterType::In,
            FilterType::NotIn,
            FilterType::ContainsOneOf,
            FilterType::ContainsNoneOf,
            FilterType::Between,
            FilterType::NotBetween,
        ],
    };

    if mv_enabled {
        types.push(FilterType::HasMissingValue);
        types.push(FilterType::DoesNotHaveMissingValue);
    }

    types
}

/// Returns the default [`FilterType`] for a given column type.
///
/// This matches the JS client's `TYPES_BY_JSON_TYPE_DEFAULT` mapping.
/// For example, string columns default to [`FilterType::Contains`],
/// while numeric columns default to [`FilterType::Equal`].
///
/// # Examples
///
/// ```
/// use labkey_rs::filter::{FilterType, JsonColumnType, default_filter_for_column_type};
///
/// assert_eq!(default_filter_for_column_type(JsonColumnType::String), FilterType::Contains);
/// assert_eq!(default_filter_for_column_type(JsonColumnType::Int), FilterType::Equal);
/// assert_eq!(default_filter_for_column_type(JsonColumnType::Date), FilterType::DateEqual);
/// ```
#[must_use]
pub fn default_filter_for_column_type(column_type: JsonColumnType) -> FilterType {
    match column_type {
        JsonColumnType::Array => FilterType::ArrayContainsAll,
        JsonColumnType::Date => FilterType::DateEqual,
        JsonColumnType::String => FilterType::Contains,
        JsonColumnType::Boolean
        | JsonColumnType::Float
        | JsonColumnType::Int
        | JsonColumnType::Time => FilterType::Equal,
    }
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
    fn encode_filters_preserves_special_characters_in_column_names_and_values() {
        let filters = vec![
            Filter::equal("En<cod ed", "Va|ue?"),
            Filter::new(
                "Col&Name",
                FilterType::GreaterThan,
                FilterValue::Single("100%".into()),
            ),
        ];
        let pairs = encode_filters(&filters, "query");

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "query.En<cod ed~eq");
        assert_eq!(pairs[0].1, "Va|ue?");
        assert_eq!(pairs[1].0, "query.Col&Name~gt");
        assert_eq!(pairs[1].1, "100%");
    }

    #[test]
    fn container_filter_rejects_unknown_variant_on_deserialization() {
        let result = serde_json::from_str::<ContainerFilter>(r#""FutureFilter""#);
        assert!(
            result.is_err(),
            "ContainerFilter uses standard serde enum deserialization and should reject unknown variants"
        );
        let err_msg = result
            .expect_err("unknown variant should fail deserialization")
            .to_string();
        assert!(
            err_msg.contains("FutureFilter"),
            "error message should mention the unrecognized variant, got: {err_msg}"
        );
    }

    #[test]
    fn encode_filters_handles_duplicate_column_and_type_as_separate_entries() {
        let filters = vec![Filter::equal("Name", "Alice"), Filter::equal("Name", "Bob")];
        let pairs = encode_filters(&filters, "query");

        assert_eq!(
            pairs.len(),
            2,
            "duplicate column+type should produce two entries"
        );
        assert_eq!(pairs[0], ("query.Name~eq".into(), "Alice".into()));
        assert_eq!(pairs[1], ("query.Name~eq".into(), "Bob".into()));
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

    // ── display_text tests ──────────────────────────────────────────

    #[test]
    fn display_text_covers_all_variants() {
        for ft in FilterType::ALL {
            let text = ft.display_text();
            assert!(
                !text.is_empty(),
                "display_text for {ft:?} should not be empty"
            );
        }
    }

    #[test]
    fn display_text_matches_upstream_js_values() {
        // Spot-check a representative sample against the JS Types.ts displayText values.
        assert_eq!(FilterType::Equal.display_text(), "Equals");
        assert_eq!(FilterType::NotEqual.display_text(), "Does Not Equal");
        assert_eq!(FilterType::GreaterThan.display_text(), "Is Greater Than");
        assert_eq!(
            FilterType::GreaterThanOrEqual.display_text(),
            "Is Greater Than or Equal To"
        );
        assert_eq!(FilterType::LessThan.display_text(), "Is Less Than");
        assert_eq!(
            FilterType::LessThanOrEqual.display_text(),
            "Is Less Than or Equal To"
        );
        assert_eq!(FilterType::In.display_text(), "Equals One Of");
        assert_eq!(FilterType::NotIn.display_text(), "Does Not Equal Any Of");
        assert_eq!(FilterType::Contains.display_text(), "Contains");
        assert_eq!(
            FilterType::DoesNotContain.display_text(),
            "Does Not Contain"
        );
        assert_eq!(FilterType::StartsWith.display_text(), "Starts With");
        assert_eq!(
            FilterType::DoesNotStartWith.display_text(),
            "Does Not Start With"
        );
        assert_eq!(FilterType::ContainsOneOf.display_text(), "Contains One Of");
        assert_eq!(
            FilterType::ContainsNoneOf.display_text(),
            "Does Not Contain Any Of"
        );
        assert_eq!(FilterType::Between.display_text(), "Between");
        assert_eq!(FilterType::NotBetween.display_text(), "Not Between");
        assert_eq!(FilterType::IsBlank.display_text(), "Is Blank");
        assert_eq!(FilterType::IsNotBlank.display_text(), "Is Not Blank");
        assert_eq!(FilterType::HasAnyValue.display_text(), "Has Any Value");
        assert_eq!(FilterType::MemberOf.display_text(), "Member Of");
        assert_eq!(FilterType::Q.display_text(), "Search");
    }

    #[test]
    fn display_text_date_variants_match_non_date_counterparts() {
        // JS reuses the same displayText for date variants.
        assert_eq!(
            FilterType::DateEqual.display_text(),
            FilterType::Equal.display_text()
        );
        assert_eq!(
            FilterType::DateNotEqual.display_text(),
            FilterType::NotEqual.display_text()
        );
        assert_eq!(
            FilterType::DateGreaterThan.display_text(),
            FilterType::GreaterThan.display_text()
        );
        assert_eq!(
            FilterType::DateLessThan.display_text(),
            FilterType::LessThan.display_text()
        );
        assert_eq!(
            FilterType::DateGreaterThanOrEqual.display_text(),
            FilterType::GreaterThanOrEqual.display_text()
        );
        assert_eq!(
            FilterType::DateLessThanOrEqual.display_text(),
            FilterType::LessThanOrEqual.display_text()
        );
    }

    // ── programmatic_name / from_name tests ─────────────────────────

    #[test]
    fn programmatic_name_round_trips_through_from_name() {
        for ft in FilterType::ALL {
            let name = ft.programmatic_name();
            let recovered = FilterType::from_name(name);
            assert_eq!(
                recovered,
                Some(*ft),
                "from_name round-trip failed for {ft:?} with name {name:?}"
            );
        }
    }

    #[test]
    fn from_name_accepts_js_aliases() {
        assert_eq!(
            FilterType::from_name("NEQ_OR_NULL"),
            Some(FilterType::NotEqualOrNull)
        );
        assert_eq!(FilterType::from_name("ISBLANK"), Some(FilterType::IsBlank));
        assert_eq!(
            FilterType::from_name("NONBLANK"),
            Some(FilterType::IsNotBlank)
        );
        assert_eq!(
            FilterType::from_name("HAS_MISSING_VALUE"),
            Some(FilterType::HasMissingValue)
        );
        assert_eq!(
            FilterType::from_name("DOES_NOT_HAVE_MISSING_VALUE"),
            Some(FilterType::DoesNotHaveMissingValue)
        );
    }

    #[test]
    fn from_name_returns_none_for_unknown() {
        assert!(FilterType::from_name("NOPE").is_none());
        assert!(FilterType::from_name("").is_none());
        assert!(FilterType::from_name("equal").is_none()); // case-sensitive
    }

    #[test]
    fn programmatic_name_matches_java_canonical_names() {
        // Spot-check against Java Filter.Operator programmaticName values.
        assert_eq!(FilterType::Equal.programmatic_name(), "EQUAL");
        assert_eq!(FilterType::NotEqual.programmatic_name(), "NOT_EQUAL");
        assert_eq!(
            FilterType::NotEqualOrNull.programmatic_name(),
            "NOT_EQUAL_OR_MISSING"
        );
        assert_eq!(FilterType::GreaterThan.programmatic_name(), "GREATER_THAN");
        assert_eq!(FilterType::IsBlank.programmatic_name(), "MISSING");
        assert_eq!(FilterType::IsNotBlank.programmatic_name(), "NOT_MISSING");
        assert_eq!(
            FilterType::HasMissingValue.programmatic_name(),
            "MV_INDICATOR"
        );
        assert_eq!(
            FilterType::DoesNotHaveMissingValue.programmatic_name(),
            "NO_MV_INDICATOR"
        );
        assert_eq!(
            FilterType::ArrayIsEmpty.programmatic_name(),
            "ARRAY_ISEMPTY"
        );
        assert_eq!(
            FilterType::ArrayIsNotEmpty.programmatic_name(),
            "ARRAY_ISNOTEMPTY"
        );
        assert_eq!(FilterType::Where.programmatic_name(), "WHERE");
    }

    // ── description tests ───────────────────────────────────────────

    #[test]
    fn description_single_filter() {
        let filters = vec![Filter::equal("Name", "Alice")];
        assert_eq!(description(&filters), "Equals Alice");
    }

    #[test]
    fn description_multiple_filters_joined_with_and() {
        let filters = vec![
            Filter::new(
                "Age",
                FilterType::GreaterThan,
                FilterValue::Single("10".into()),
            ),
            Filter::new(
                "Age",
                FilterType::LessThan,
                FilterValue::Single("100".into()),
            ),
        ];
        assert_eq!(
            description(&filters),
            "Is Greater Than 10 AND Is Less Than 100"
        );
    }

    #[test]
    fn description_no_value_filter_omits_trailing_space() {
        let filters = vec![Filter::new("Notes", FilterType::IsBlank, FilterValue::None)];
        assert_eq!(description(&filters), "Is Blank");
    }

    #[test]
    fn description_empty_filters_returns_empty_string() {
        assert_eq!(description(&[]), "");
    }

    // ── from_parameters tests ───────────────────────────────────────

    #[test]
    fn from_parameters_round_trips_with_encode_filters() {
        let original = vec![
            Filter::equal("Name", "Alice"),
            Filter::new(
                "Age",
                FilterType::GreaterThan,
                FilterValue::Single("21".into()),
            ),
        ];
        let params = encode_filters(&original, "query");
        let recovered = from_parameters(
            params.iter().map(|(k, v)| (k.as_str(), v.as_str())),
            "query",
        );
        assert_eq!(recovered.len(), 2);
        assert_eq!(recovered[0].column_name(), "Name");
        assert_eq!(recovered[0].filter_type(), FilterType::Equal);
        assert_eq!(recovered[0].url_param_value(), "Alice");
        assert_eq!(recovered[1].column_name(), "Age");
        assert_eq!(recovered[1].filter_type(), FilterType::GreaterThan);
        assert_eq!(recovered[1].url_param_value(), "21");
    }

    #[test]
    fn from_parameters_skips_unknown_suffixes() {
        let params = [
            ("query.Name~eq", "Alice"),
            ("query.Col~unknownsuffix", "val"),
        ];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].column_name(), "Name");
    }

    #[test]
    fn from_parameters_skips_wrong_data_region() {
        let params = [("query.Name~eq", "Alice"), ("other.Name~eq", "Bob")];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].url_param_value(), "Alice");
    }

    #[test]
    fn from_parameters_skips_params_without_tilde() {
        let params = [("query.Name~eq", "Alice"), ("query.sort", "+Name")];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
    }

    #[test]
    fn from_parameters_handles_multi_valued_in_filter() {
        let params = [("query.Status~in", "Active;Pending;Closed")];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].filter_type(), FilterType::In);
        assert_eq!(filters[0].url_param_value(), "Active;Pending;Closed");
    }

    #[test]
    fn from_parameters_handles_json_encoded_multi_value() {
        let params = [(r"query.Tags~in", r#"{json:["a;b","c"]}"#)];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        // The json-encoded value should round-trip back to json encoding
        // since the values contain the separator.
        let value = filters[0].url_param_value();
        assert!(
            value.starts_with("{json:"),
            "should re-encode as json, got: {value}"
        );
    }

    #[test]
    fn from_parameters_handles_no_value_filter() {
        let params = [("query.Notes~isblank", "")];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].filter_type(), FilterType::IsBlank);
        assert_eq!(filters[0].url_param_value(), "");
    }

    #[test]
    fn from_parameters_handles_between_filter() {
        let params = [("query.Age~between", "18,65")];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].filter_type(), FilterType::Between);
        assert_eq!(filters[0].url_param_value(), "18,65");
    }

    #[test]
    fn from_parameters_handles_table_wise_q_filter() {
        let params = [("query.*~q", "search term")];
        let filters = from_parameters(params, "query");
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].filter_type(), FilterType::Q);
        assert_eq!(filters[0].column_name(), "*");
        assert_eq!(filters[0].url_param_value(), "search term");
    }

    #[test]
    fn from_parameters_empty_input_returns_empty() {
        let filters = from_parameters(std::iter::empty::<(&str, &str)>(), "query");
        assert!(filters.is_empty());
    }

    #[test]
    fn filter_type_accessor_returns_correct_op() {
        let f = Filter::equal("Name", "Alice");
        assert_eq!(f.filter_type(), FilterType::Equal);

        let f = Filter::new("Col", FilterType::Between, FilterValue::None);
        assert_eq!(f.filter_type(), FilterType::Between);
    }

    // ── JsonColumnType tests ────────────────────────────────────────

    #[test]
    fn json_column_type_from_type_string_case_insensitive() {
        assert_eq!(
            JsonColumnType::from_type_string("int"),
            Some(JsonColumnType::Int)
        );
        assert_eq!(
            JsonColumnType::from_type_string("INT"),
            Some(JsonColumnType::Int)
        );
        assert_eq!(
            JsonColumnType::from_type_string("Int"),
            Some(JsonColumnType::Int)
        );
        assert_eq!(
            JsonColumnType::from_type_string("string"),
            Some(JsonColumnType::String)
        );
        assert_eq!(
            JsonColumnType::from_type_string("STRING"),
            Some(JsonColumnType::String)
        );
    }

    #[test]
    fn json_column_type_from_type_string_accepts_aliases() {
        assert_eq!(
            JsonColumnType::from_type_string("datetime"),
            Some(JsonColumnType::Date)
        );
        assert_eq!(
            JsonColumnType::from_type_string("double"),
            Some(JsonColumnType::Float)
        );
        assert_eq!(
            JsonColumnType::from_type_string("integer"),
            Some(JsonColumnType::Int)
        );
    }

    #[test]
    fn json_column_type_from_type_string_returns_none_for_unknown() {
        assert!(JsonColumnType::from_type_string("").is_none());
        assert!(JsonColumnType::from_type_string("varchar").is_none());
        assert!(JsonColumnType::from_type_string("bigint").is_none());
    }

    #[test]
    fn json_column_type_from_type_string_covers_all_variants() {
        let cases = [
            ("array", JsonColumnType::Array),
            ("boolean", JsonColumnType::Boolean),
            ("date", JsonColumnType::Date),
            ("float", JsonColumnType::Float),
            ("int", JsonColumnType::Int),
            ("string", JsonColumnType::String),
            ("time", JsonColumnType::Time),
        ];
        for (input, expected) in cases {
            assert_eq!(
                JsonColumnType::from_type_string(input),
                Some(expected),
                "from_type_string({input:?}) should return {expected:?}"
            );
        }
    }

    // ── filter_types_for_column_type tests ──────────────────────────

    #[test]
    fn filter_types_for_string_includes_text_operators() {
        let types = filter_types_for_column_type(JsonColumnType::String, false);
        assert!(types.contains(&FilterType::Contains));
        assert!(types.contains(&FilterType::StartsWith));
        assert!(types.contains(&FilterType::DoesNotContain));
        assert!(types.contains(&FilterType::DoesNotStartWith));
        assert!(types.contains(&FilterType::ContainsOneOf));
        assert!(types.contains(&FilterType::ContainsNoneOf));
    }

    #[test]
    fn filter_types_for_int_excludes_text_operators() {
        let types = filter_types_for_column_type(JsonColumnType::Int, false);
        assert!(!types.contains(&FilterType::Contains));
        assert!(!types.contains(&FilterType::StartsWith));
        assert!(types.contains(&FilterType::Equal));
        assert!(types.contains(&FilterType::Between));
        assert!(types.contains(&FilterType::In));
    }

    #[test]
    fn filter_types_for_date_uses_date_specific_operators() {
        let types = filter_types_for_column_type(JsonColumnType::Date, false);
        assert!(types.contains(&FilterType::DateEqual));
        assert!(types.contains(&FilterType::DateGreaterThan));
        assert!(
            !types.contains(&FilterType::Equal),
            "date should use DateEqual, not Equal"
        );
    }

    #[test]
    fn filter_types_for_array_uses_array_operators() {
        let types = filter_types_for_column_type(JsonColumnType::Array, false);
        assert!(types.contains(&FilterType::ArrayContainsAll));
        assert!(types.contains(&FilterType::ArrayContainsAny));
        assert!(types.contains(&FilterType::ArrayIsEmpty));
        assert!(!types.contains(&FilterType::Equal));
    }

    #[test]
    fn filter_types_for_boolean_is_limited() {
        let types = filter_types_for_column_type(JsonColumnType::Boolean, false);
        assert!(types.contains(&FilterType::Equal));
        assert!(types.contains(&FilterType::IsBlank));
        assert!(!types.contains(&FilterType::GreaterThan));
        assert!(!types.contains(&FilterType::Contains));
    }

    #[test]
    fn filter_types_mv_enabled_appends_missing_value_operators() {
        let without_mv = filter_types_for_column_type(JsonColumnType::Int, false);
        let with_mv = filter_types_for_column_type(JsonColumnType::Int, true);
        assert!(!without_mv.contains(&FilterType::HasMissingValue));
        assert!(!without_mv.contains(&FilterType::DoesNotHaveMissingValue));
        assert!(with_mv.contains(&FilterType::HasMissingValue));
        assert!(with_mv.contains(&FilterType::DoesNotHaveMissingValue));
        assert_eq!(with_mv.len(), without_mv.len() + 2);
    }

    #[test]
    fn filter_types_float_and_int_have_same_operators() {
        let int_types = filter_types_for_column_type(JsonColumnType::Int, false);
        let float_types = filter_types_for_column_type(JsonColumnType::Float, false);
        assert_eq!(int_types, float_types);
    }

    // ── default_filter_for_column_type tests ────────────────────────

    #[test]
    fn default_filter_for_column_type_matches_js_defaults() {
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::Array),
            FilterType::ArrayContainsAll
        );
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::Boolean),
            FilterType::Equal
        );
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::Date),
            FilterType::DateEqual
        );
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::Float),
            FilterType::Equal
        );
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::Int),
            FilterType::Equal
        );
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::String),
            FilterType::Contains
        );
        assert_eq!(
            default_filter_for_column_type(JsonColumnType::Time),
            FilterType::Equal
        );
    }

    #[test]
    fn default_filter_is_always_in_the_types_list() {
        let all_types = [
            JsonColumnType::Array,
            JsonColumnType::Boolean,
            JsonColumnType::Date,
            JsonColumnType::Float,
            JsonColumnType::Int,
            JsonColumnType::String,
            JsonColumnType::Time,
        ];
        for ct in all_types {
            let default = default_filter_for_column_type(ct);
            let applicable = filter_types_for_column_type(ct, false);
            assert!(
                applicable.contains(&default),
                "default filter {default:?} for {ct:?} should be in the applicable types list"
            );
        }
    }
}
