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

  pub fn fingerprint(&self) -> Fingerprint {
    Fingerprint(self.files.fingerprint())
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

  pub(crate) fn message(&self, include_time: bool) -> Result<Message> {
    let fingerprint = self.verify_notes()?;

    Ok(Message {
      fingerprint,
      time: include_time.then(now).transpose()?,
    })
  }

  pub fn save(&self, path: &Utf8Path) -> Result {
    filesystem::write(
      path,
      format!("{}\n", serde_json::to_string_pretty(self).unwrap()),
    )
  }

  pub(crate) fn sign(
    &mut self,
    options: SignOptions,
    keychain: &Keychain,
    key: &KeyName,
  ) -> Result {
    let message = self.message(options.time)?;

    let serialized = message.serialize();

    let (key, signature) = keychain.sign(key, &message, &serialized)?;

    for note in &mut self.notes {
      if note.message(message.fingerprint) == message {
        ensure! {
          !note.has_signature(key) || options.overwrite,
          error::SignatureAlreadyExists { key },
        }
        note.signatures.insert(signature);
        return Ok(());
      }
    }

    self.notes.push(Note::from_message(message, signature));

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

  pub(crate) fn verify_notes(&self) -> Result<Fingerprint> {
    let fingerprint = self.fingerprint();

    let mut digests = BTreeMap::new();
    let mut signatures = BTreeMap::new();
    for (index, note) in self.notes.iter().enumerate() {
      for signature in &note.signatures {
        let public_key = signature.public_key();
        if let Some(first) = signatures.insert(public_key, index) {
          return Err(
            error::DuplicateSignature {
              first,
              public_key,
              second: index,
            }
            .build(),
          );
        }
      }

      if let Some(first) = digests.insert(note.message(fingerprint), index) {
        return Err(
          error::DuplicateNote {
            first,
            second: index,
          }
          .build(),
        );
      }

      if note.verify(fingerprint)? == 0 {
        return Err(error::UnsignedNote { index }.build());
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
    assert_eq!(json, r#"{"files":{},"notes":[]}"#);
    assert_eq!(serde_json::from_str::<Manifest>(&json).unwrap(), manifest);
  }

  #[test]
  fn manifests_in_readme_are_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```json(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let manifest = capture[1].replace("…", test::SIGNATURE);
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
