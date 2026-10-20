use super::*;

/// Package sort key.
#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Default, Deserialize, Display, EnumIter, Json, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Sort {
  /// Title.
  #[default]
  Title,
  /// Creator.
  Creator,
  /// Year.
  Year,
  /// Media type.
  Media,
  /// Number of files.
  Files,
  /// Total size.
  Size,
}
