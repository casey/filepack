use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Default, Deserialize, Display, EnumIter, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Sort {
  #[default]
  Title,
  Creator,
  Year,
  Media,
  Files,
  Size,
}
