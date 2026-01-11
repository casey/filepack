use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Manifest {
  #[serde(default, skip_serializing_if = "Directory::is_empty")]
  pub files: Directory,
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  pub signatures: BTreeMap<PublicKey, Signature>,
}

impl Manifest {
  pub(crate) const FILENAME: &'static str = "filepack.json";

  pub(crate) fn empty_directories(&self) -> BTreeSet<RelativePath> {
    let mut empty = BTreeSet::new();

    for (path, entry) in self.entries() {
      if let Entry::Directory(directory) = entry
        && directory.entries.is_empty()
      {
        empty.insert(path);
      }
    }

    empty
  }

  fn entries(&self) -> Entries {
    self.into()
  }

  pub(crate) fn files(&self) -> BTreeMap<RelativePath, File> {
    let mut files = BTreeMap::new();

    for (path, entry) in self.entries() {
      if let Entry::File(file) = entry {
        let old = files.insert(path, *file);
        assert!(old.is_none());
      }
    }

    files
  }

  #[must_use]
  pub fn fingerprint(&self) -> Hash {
    self.files.fingerprint()
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<Self> {
    Ok(Self::load_with_path(path)?.1)
  }

  pub(crate) fn load_with_path(path: Option<&Utf8Path>) -> Result<(Utf8PathBuf, Self)> {
    let path = if let Some(path) = path {
      if filesystem::metadata(path)?.is_dir() {
        path.join(Manifest::FILENAME)
      } else {
        path.into()
      }
    } else {
      current_dir()?.join(Manifest::FILENAME)
    };

    let json = filesystem::read_to_string_opt(&path)?
      .ok_or_else(|| error::ManifestNotFound { path: &path }.build())?;

    let manifest =
      serde_json::from_str(&json).context(error::DeserializeManifest { path: &path })?;

    Ok((path, manifest))
  }

  pub fn save(&self, path: &Utf8Path) -> Result<()> {
    filesystem::write(path, format!("{}\n", serde_json::to_string(self).unwrap()))
  }

  pub(crate) fn total_size(&self) -> u128 {
    let mut size = 0u128;
    for (_path, entry) in self.entries() {
      if let Entry::File(file) = entry {
        size = size.checked_add(file.size.into()).unwrap();
      }
    }
    size
  }
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  #[test]
  fn empty_manifest_serialization() {
    let manifest = Manifest {
      files: Directory::new(),
      signatures: BTreeMap::new(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert_eq!(json, "{}");
    assert_eq!(serde_json::from_str::<Manifest>(&json).unwrap(), manifest);
  }

  #[test]
  fn manifests_in_readme_are_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```json(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      serde_json::from_str::<Manifest>(&capture[1]).unwrap();
    }
  }

  #[test]
  fn unknown_fields_are_rejected() {
    assert!(
      serde_json::from_str::<Manifest>(&r#"{"hello": []}"#)
        .unwrap_err()
        .to_string()
        .starts_with("unknown field `hello`")
    );
  }
}
