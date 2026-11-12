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

    for (path, entry) in self.package.descendants() {
      if let DirectoryTreeEntry::Directory(directory) = entry
        && directory.entries.is_empty()
      {
        empty.insert(path);
      }
    }

    empty
  }

  pub(crate) fn files(&self) -> BTreeMap<RelativePath, File> {
    self.package.files()
  }

  pub(crate) fn from_json(json: &str, path: &Utf8Path) -> Result<Self, ManifestError> {
    let manifest =
      serde_json::from_str::<Self>(json).context(manifest_error::DeserializeManifest { path })?;

    for name in Archive::root_entries() {
      ensure! {
        !manifest.unknown.entries.contains_key(name),
        manifest_error::UnknownEntryConflict { name: name.to_owned(), path },
      }
    }

    if let Some(path) = manifest.reserved_filename() {
      return Err(manifest_error::ReservedFilename { path }.build());
    }

    let files = manifest
      .package
      .files()
      .into_values()
      .chain(manifest.unknown.files().into_values())
      .collect::<Vec<File>>();

    let hashes = files
      .iter()
      .map(|file| file.hash)
      .collect::<BTreeSet<Hash>>();

    let mut unreferenced = BTreeSet::new();
    for (&expected, content) in &manifest.embedded {
      let actual = Hash::bytes(content);

      ensure! {
        actual == expected,
        manifest_error::EmbeddedFileHashMismatch { actual, expected, path },
      }

      if !hashes.contains(&expected) {
        unreferenced.insert(expected);
      }
    }

    ensure! {
      unreferenced.is_empty(),
      manifest_error::UnreferencedEmbeddedFiles { hashes: unreferenced, path },
    }

    for file in files {
      if let Some(content) = manifest.embedded.get(&file.hash) {
        let actual = content.len().into_u64();

        ensure! {
          actual == file.size,
          manifest_error::EmbeddedFileSizeMismatch {
            actual,
            expected: file.size,
            hash: file.hash,
            path,
          },
        }
      }
    }

    Ok(manifest)
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<Self, ManifestError> {
    Self::load_with_options(DecodeOptions::default(), path)
  }

  pub fn load_with_options(
    options: DecodeOptions,
    path: Option<&Utf8Path>,
  ) -> Result<Self, ManifestError> {
    Loader::load_with_options(options, path)?.unpack()
  }

  pub(crate) fn pack(&self, path: &Utf8Path) -> Result<Archive, ManifestError> {
    Archive::pack(self).context(manifest_error::ManifestTotals { path })
  }

  pub(crate) fn reserved_filename(&self) -> Option<RelativePath> {
    for (path, _entry) in self.package.descendants() {
      if path.filename().is_reserved() {
        return Some(path);
      }
    }

    None
  }

  pub fn save(&self, path: &Utf8Path) -> Result<(), ManifestError> {
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

  #[cfg(test)]
  pub(crate) fn with_package(package: DirectoryTree) -> Self {
    Self {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::new(),
      unknown: DirectoryTree::new(),
    }
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
    let manifest = Manifest::with_package(DirectoryTree::new());
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
  fn reserved_filename() {
    #[track_caller]
    fn case(path: &str, expected: &str) {
      let mut package = DirectoryTree::new();

      package
        .create_file(&path.parse().unwrap(), File::new(b"foo"))
        .unwrap();

      let manifest = Manifest::with_package(package);

      assert_eq!(
        manifest.reserved_filename(),
        Some(expected.parse().unwrap()),
      );
    }

    case("METADATA.FILEMETA", "METADATA.FILEMETA");
    case("foo/.FILEPACK", "foo/.FILEPACK");
    case("manifest.filepack/foo", "manifest.filepack");
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
