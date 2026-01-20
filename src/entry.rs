use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", untagged)]
pub enum Entry {
  Directory(Directory),
  File(File),
}

impl Entry {
  pub(crate) fn fingerprint(&self, component: &Component) -> Hash {
    let mut hasher = FingerprintHasher::new(FingerprintPrefix::Entry);

    hasher.field(0, component.as_bytes());

    let inner = match self {
      Self::Directory(directory) => directory.fingerprint(),
      Self::File(file) => file.fingerprint(),
    };

    hasher.field(1, inner.as_bytes());

    hasher.finalize()
  }
}
