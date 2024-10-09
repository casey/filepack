use super::*;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Manifest {
  pub(crate) files: BTreeMap<RelativePath, Entry>,
  #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
  pub(crate) signatures: BTreeMap<PublicKey, Signature>,
}

impl Manifest {
  pub(crate) const FILENAME: &'static str = "filepack.json";

  pub(crate) fn root_hash(&self) -> Hash {
    let mut hasher = blake3::Hasher::new();

    for (path, entry) in &self.files {
      hasher.update(&u64::try_from(path.len()).unwrap().to_le_bytes());
      hasher.update(path.as_str().as_bytes());
      hasher.update(&entry.size.to_le_bytes());
      hasher.update(entry.hash.as_bytes());
    }

    hasher.finalize().into()
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
}
