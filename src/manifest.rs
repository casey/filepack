use super::*;

/// Package manifest unpacked from a package archive.
#[serde_as]
#[derive(Debug, Deserialize, Json, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
  /// File contents embedded in the manifest, keyed by hash.
  pub embedded: BTreeMap<Hash, Bytes>,
  /// Files and directories in the package, as a tree of path components.
  pub package: DirectoryTree,
  /// Signatures over the package fingerprint.
  #[serde_as(as = "SetPreventDuplicates<serde_with::Same>")]
  pub signatures: BTreeSet<Decoded<Attestation>>,
  /// Entries in the manifest archive not recognized by this version of Filepack.
  #[serde(default, skip_serializing_if = "DirectoryTree::is_empty")]
  pub unknown: DirectoryTree,
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

  fn file_hashes(&self) -> BTreeSet<Hash> {
    let mut hashes = BTreeSet::new();
    let mut stack = self
      .package
      .entries
      .values()
      .chain(self.unknown.entries.values())
      .collect::<Vec<&DirectoryTreeEntry>>();
    while let Some(entry) = stack.pop() {
      match entry {
        DirectoryTreeEntry::Directory(directory) => stack.extend(directory.entries.values()),
        DirectoryTreeEntry::File(file) => {
          hashes.insert(file.hash);
        }
      }
    }
    hashes
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

  pub(crate) fn from_json(json: &str, path: &Utf8Path) -> Result<Self> {
    let manifest =
      serde_json::from_str::<Self>(json).context(error::DeserializeManifest { path })?;

    for name in Archive::root_entries() {
      ensure! {
        !manifest.unknown.entries.contains_key(name),
        error::UnknownEntryConflict { name: name.to_owned(), path },
      }
    }

    for name in manifest.package.entries.keys() {
      ensure! {
        !name.is_reserved(),
        error::ReservedPath { name },
      }
    }

    let hashes = manifest.file_hashes();

    let mut unreferenced = BTreeSet::new();
    for (&expected, content) in &manifest.embedded {
      let actual = Hash::bytes(content);

      ensure! {
        actual == expected,
        error::EmbeddedFileHashMismatch { actual, expected, path },
      }

      if !hashes.contains(&expected) {
        unreferenced.insert(expected);
      }
    }

    ensure! {
      unreferenced.is_empty(),
      error::UnreferencedEmbeddedFiles { hashes: unreferenced, path },
    }

    Ok(manifest)
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<Self> {
    Self::load_with_options(DecodeOptions::default(), path)
  }

  pub fn load_with_options(options: DecodeOptions, path: Option<&Utf8Path>) -> Result<Self> {
    Loader::load_with_options(options, path)?.unpack()
  }

  pub(crate) fn pack(&self, path: &Utf8Path) -> Result<Archive> {
    Archive::pack(self).context(error::ManifestTotals { path })
  }

  pub fn save(&self, path: &Utf8Path) -> Result {
    filesystem::write(path, self.pack(path)?.encode_to_vec())?;
    Ok(())
  }

  pub(crate) fn sign(
    &mut self,
    fingerprint: Fingerprint,
    options: SignOptions,
    keychain: &Keychain,
    key: &KeyName,
  ) -> Result {
    let statement = Statement {
      fingerprint,
      timestamp: options
        .timestamp
        .then(now)
        .transpose()
        .context(error::Time)?,
    };

    let signature = keychain.sign(key, statement)?;

    self.signatures.insert(Decoded::Known(signature));

    Ok(())
  }

  pub(crate) fn signatures(&self) -> impl Iterator<Item = &Attestation> {
    self.signatures.iter().filter_map(Decoded::known)
  }

  pub(crate) fn unknown_signatures(&self) -> usize {
    self
      .signatures
      .iter()
      .filter(|signature| signature.known().is_none())
      .count()
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
        Attestation::EXAMPLE,
        Attestation::EXAMPLE,
      ))
      .unwrap_err()
      .to_string(),
      r"invalid entry: found duplicate value at line 1 column \d+",
    );
  }

  #[test]
  fn embedded_serializes_as_hex() {
    let manifest = Manifest {
      embedded: BTreeMap::from([(Hash::bytes(b"foo"), b"foo".to_vec().into())]),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
      unknown: DirectoryTree::new(),
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
      unknown: DirectoryTree::new(),
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
      let manifest = capture[1].replace("…", Attestation::EXAMPLE);
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
