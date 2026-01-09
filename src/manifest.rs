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

  pub(crate) fn files(&self) -> BTreeMap<RelativePath, File> {
    let mut files = BTreeMap::new();
    let mut stack = vec![(&self.files, Vec::new())];
    while let Some((directory, components)) = stack.pop() {
      for (component, entry) in &directory.entries {
        let mut components = components.clone();
        components.push(component.as_str());
        match entry {
          Entry::File(file) => {
            let path = components.join("/").parse::<RelativePath>().unwrap();
            let old = files.insert(path, *file);
            assert!(old.is_none());
          }
          Entry::Directory(directory) => stack.push((directory, components)),
        }
      }
    }
    files
  }

  pub(crate) fn empty_directories(&self) -> BTreeSet<RelativePath> {
    let mut directories = BTreeSet::new();
    let mut stack = vec![(&self.files, Vec::new())];
    while let Some((directory, components)) = stack.pop() {
      for (component, entry) in &directory.entries {
        let mut components = components.clone();
        components.push(component.as_str());
        if let Entry::Directory(directory) = entry {
          if directory.entries.is_empty() {
            let path = components.join("/").parse::<RelativePath>().unwrap();
            directories.insert(path);
          }
          stack.push((directory, components));
        }
      }
    }
    directories
  }

  #[must_use]
  pub fn fingerprint(&self) -> Hash {
    self.files.fingerprint()
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<(Utf8PathBuf, Self)> {
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

  pub(crate) fn total_size(&self) -> Option<u64> {
    let mut size = 0u64;
    let mut stack = vec![&self.files];
    while let Some(current) = stack.pop() {
      for entry in current.entries.values() {
        match entry {
          Entry::File(file) => size = size.checked_add(file.size)?,
          Entry::Directory(directory) => stack.push(directory),
        }
      }
    }
    Some(size)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  #[test]
  fn manifests_in_readme_are_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```json(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      serde_json::from_str::<Manifest>(&capture[1]).unwrap();
    }
  }

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
}
