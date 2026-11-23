use super::*;

/// Lint configuration.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Lints {
  /// Expected lints.
  #[json(example = "media-missing")]
  pub(crate) expect: Option<NonEmptyOrderedSet<Lint>>,
}
