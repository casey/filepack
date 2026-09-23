use super::*;

#[serde_as]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
  #[serde_as(as = "BTreeMap<serde_with::Same, serde_with::hex::Hex>")]
  pub embedded: BTreeMap<Hash, Vec<u8>>,
  pub package: DirectoryTree,
  #[serde_as(as = "SetPreventDuplicates<serde_with::Same>")]
  pub signatures: BTreeSet<Attestation>,
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

  pub(crate) fn from_json(json: &str, path: &Utf8Path) -> Result<Self> {
    let manifest =
      serde_json::from_str::<Self>(json).context(error::DeserializeManifest { path })?;

    let hashes = manifest
      .files()
      .values()
      .map(|file| file.hash)
      .collect::<BTreeSet<Hash>>();

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
    Loader::load(path)?.unpack()
  }

  pub fn save(&self, path: &Utf8Path) -> Result {
    let deco = Archive::pack(self)
      .context(error::ManifestTotals { path })?
      .encode_magic_bytes();
    filesystem::write(path, deco)
  }

  pub(crate) fn sign(
    &mut self,
    fingerprint: Fingerprint,
    options: SignOptions,
    keychain: &Keychain,
    key: &KeyName,
  ) -> Result {
    let statement = Statement {
      version: Version::Zero,
      fingerprint,
      timestamp: options
        .timestamp
        .then(now)
        .transpose()
        .context(error::Time)?,
    };

    let signature = keychain.sign(key, statement)?;

    self.signatures.insert(signature);

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
