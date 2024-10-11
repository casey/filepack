use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Manifest {
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  pub(crate) files: BTreeMap<RelativePath, Entry>,
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  pub(crate) signatures: BTreeMap<PublicKey, Signature>,
}

impl Manifest {
  pub(crate) const FILENAME: &'static str = "filepack.json";

  pub(crate) fn root_hash(&self) -> Hash {
    let canonical = Self {
      files: self.files.clone(),
      signatures: BTreeMap::new(),
    };

    let mut hasher = blake3::Hasher::new();

    serde_json::to_writer(&mut hasher, &canonical).unwrap();

    hasher.finalize().into()
  }

  pub(crate) fn to_json(&self) -> String {
    serde_json::to_string(self).unwrap()
  }

  pub(crate) fn total_size(&self) -> u128 {
    self
      .files
      .values()
      .map(|entry| u128::from(entry.size))
      .sum()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  #[test]
  fn manifests_in_readme_are_valid() {
    let readme = fs::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```json(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      serde_json::from_str::<Manifest>(&capture[1]).unwrap();
    }
  }

  #[test]
  fn empty_manifest_serialization() {
    let manifest = Manifest {
      files: BTreeMap::new(),
      signatures: BTreeMap::new(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert_eq!(json, "{}");
    assert_eq!(serde_json::from_str::<Manifest>(&json).unwrap(), manifest);
  }
}
