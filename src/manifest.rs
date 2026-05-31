use super::*;

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
  #[serde_as(as = "BTreeMap<serde_with::Same, serde_with::hex::Hex>")]
  pub embedded: BTreeMap<Hash, Vec<u8>>,
  pub package: DirectoryTree,
  #[serde_as(as = "SetPreventDuplicates<serde_with::Same>")]
  pub signatures: BTreeSet<Signature>,
}

impl Manifest {
  pub(crate) const FILENAME: &'static str = "manifest.filepack";

  pub(crate) fn empty_directories(&self) -> BTreeSet<RelativePath> {
    let mut empty = BTreeSet::new();

    for (path, entry) in self.entries() {
      if let DirectoryTreeEntry::Directory(directory) = entry
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
      if let DirectoryTreeEntry::File(file) = entry {
        let old = files.insert(path, *file);
        assert!(old.is_none());
      }
    }

    files
  }

  pub fn fingerprint(&self) -> Fingerprint {
    Archive::pack(self).fingerprint().unwrap()
  }

  pub(crate) fn from_json(json: &str, path: &Utf8Path) -> Result<Self> {
    let manifest =
      serde_json::from_str::<Self>(json).context(error::DeserializeManifest { path: &path })?;

    let mut unexpected = BTreeSet::new();
    for (file_path, file) in manifest.files() {
      if manifest.embedded.contains_key(&file.hash) && file_path != Metadata::CBOR_FILENAME {
        unexpected.insert(file_path);
      }
    }

    ensure! {
      unexpected.is_empty(),
      error::UnexpectedEmbeddedFiles { path, unexpected },
    }

    manifest.verify_signatures()?;

    Ok(manifest)
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<Self> {
    Ok(Self::load_with_opt_path(path)?.1)
  }

  pub(crate) fn opt_path(path: Option<&Utf8Path>) -> Result<Utf8PathBuf> {
    Ok(if let Some(path) = path {
      if path.is_dir() {
        path.join(Self::FILENAME)
      } else {
        path.into()
      }
    } else {
      current_dir()?.join(Self::FILENAME)
    })
  }

  pub(crate) fn load_with_opt_path(path: Option<&Utf8Path>) -> Result<(Utf8PathBuf, Self)> {
    let path = Self::opt_path(path)?;

    let manifest = Self::load_with_path(&path, &path)?;

    Ok((path, manifest))
  }

  pub(crate) fn load_with_path(path: &Utf8Path, display_path: &Utf8Path) -> Result<Self> {
    let archive = Archive::load_with_path(path, display_path)?;

    let manifest = archive
      .unpack()
      .context(error::UnarchiveManifest { path: display_path })?;

    manifest.verify_signatures()?;

    Ok(manifest)
  }

  pub fn save(&self, path: &Utf8Path) -> Result {
    let cbor = Archive::pack(self).encode_to_vec();
    filesystem::write(path, cbor)
  }

  pub(crate) fn sign(
    &mut self,
    options: SignOptions,
    keychain: &Keychain,
    key: &KeyName,
  ) -> Result {
    let statement = self.statement(options.timestamp)?;

    let signature = keychain.sign(key, &statement)?;

    self.signatures.insert(signature);

    Ok(())
  }

  pub(crate) fn statement(&self, timestamp: bool) -> Result<Statement> {
    Ok(Statement {
      fingerprint: self.fingerprint(),
      timestamp: timestamp.then(now).transpose()?,
    })
  }

  pub(crate) fn total_size(&self) -> u128 {
    let mut size = 0u128;
    for (_path, entry) in self.entries() {
      if let DirectoryTreeEntry::File(file) = entry {
        size = size.checked_add(file.size.into()).unwrap();
      }
    }
    size
  }

  pub(crate) fn total_size_u64(&self) -> u64 {
    self.total_size().try_into().unwrap_or(u64::MAX)
  }

  fn verify_signatures(&self) -> Result {
    let fingerprint = self.fingerprint();

    for signature in &self.signatures {
      signature.verify(fingerprint)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  #[test]
  fn duplicate_signatures_are_rejected() {
    assert_matches_regex!(
      serde_json::from_str::<Manifest>(&format!(
        r#"{{"package":{{}},"signatures":["{}","{}"]}}"#,
        test::SIGNATURE,
        test::SIGNATURE,
      ))
      .unwrap_err()
      .to_string(),
      r"invalid entry: found duplicate value at line 1 column \d+",
    );
  }

  #[test]
  fn embedded_serializes_as_hex() {
    let manifest = Manifest {
      embedded: BTreeMap::from([(Hash::bytes(b"foo"), b"foo".to_vec())]),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
    };

    let json = serde_json::to_string(&manifest).unwrap();

    assert_eq!(
      json,
      format!(
        r#"{{"embedded":{{"{hash}":"666f6f"}},"package":{{}},"signatures":[]}}"#,
        hash = Hash::bytes(b"foo"),
      ),
    );

    assert_eq!(serde_json::from_str::<Manifest>(&json).unwrap(), manifest);
  }

  #[test]
  fn empty_manifest_serialization() {
    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    assert_eq!(json, r#"{"embedded":{},"package":{},"signatures":[]}"#);
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
