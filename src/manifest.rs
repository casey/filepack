use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Manifest {
  pub files: Directory,
  pub notes: Vec<Note>,
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

  pub(crate) fn sign(&mut self, keychain: &Keychain, key: &KeyName, force: bool) -> Result {
    let fingerprint = self.verify_notes()?;

    let message = Message { fingerprint };

    let digest = message.digest();

    let (key, signature) = keychain.sign(key, digest)?;

    for note in &mut self.notes {
      if note.digest(fingerprint) == digest {
        ensure! {
          note.signatures.insert(key, signature).is_none() || force,
          error::SignatureAlreadyExists { key },
        }
        return Ok(());
      }
    }

    self.notes.push(Note::from_message(message, key, signature));

    Ok(())
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

  pub(crate) fn verify_notes(&self) -> Result<Hash> {
    let fingerprint = self.fingerprint();

    let mut digests = BTreeMap::new();
    let mut signatures = BTreeMap::new();
    for (i, note) in self.notes.iter().enumerate() {
      if let Some(first) = digests.insert(note.digest(fingerprint), i) {
        return Err(error::DuplicateNote { first, second: i }.build());
      }

      for &key in note.signatures.keys() {
        if let Some(first) = signatures.insert(key, i) {
          return Err(
            error::DuplicateSignature {
              first,
              key,
              second: i,
            }
            .build(),
          );
        }
      }

      if note.verify(fingerprint)? == 0 {
        return Err(error::UnsignedNote { index: i }.build());
      }
    }

    Ok(fingerprint)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  #[test]
  fn empty_manifest_serialization() {
    let manifest = Manifest {
      files: Directory::new(),
      notes: Vec::new(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert_eq!(json, r#"{"files":{},"signatures":{}}"#);
    assert_eq!(serde_json::from_str::<Manifest>(&json).unwrap(), manifest);
  }

  #[test]
  fn manifests_in_readme_are_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```json(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let manifest = capture[1].replace(
        "…",
        concat!(
          "3f814a19e6db6431959f0393d362920846224af1d44ceee851e0caded9412d93",
          "9a221a15f6ba9a5d118a570a6b1cc48c95c7fb73581eeec1e33afdb4d0163907"
        ),
      );
      serde_json::from_str::<Manifest>(&manifest).unwrap();
    }
  }

  #[test]
  fn unknown_fields_are_rejected() {
    assert!(
      serde_json::from_str::<Manifest>(r#"{"hello": []}"#)
        .unwrap_err()
        .to_string()
        .starts_with("unknown field `hello`")
    );
  }
}
