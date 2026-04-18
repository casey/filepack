use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", untagged)]
pub(crate) enum Entry {
  Directory(Directory),
  File(File),
}
