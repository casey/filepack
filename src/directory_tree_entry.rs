use super::*;

/// Entry in a directory tree.
#[derive(Debug, Deserialize, Json, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum DirectoryTreeEntry {
  Directory(DirectoryTree),
  File(File),
}
