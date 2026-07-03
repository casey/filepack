use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Encode, Decode)]
pub(crate) struct Archive {
  #[n(0)]
  pub(crate) version: Version,
  #[n(1)]
  pub(crate) root: Hash,
  #[n(2)]
  pub(crate) files: BTreeMap<Hash, Vec<u8>>,
}

impl Archive {
  const PACKAGE: &str = "package";
  const SIGNATURES: &str = "signatures";

  fn decode_directory(
    &self,
    loose: Option<&mut BTreeSet<Hash>>,
    hash: Hash,
  ) -> Result<Directory, ArchiveError> {
    let file = self
      .files
      .get(&hash)
      .context(archive_error::FileMissing { hash })?;

    if let Some(loose) = loose {
      loose.remove(&hash);
    }

    Directory::decode_from_slice(file).context(archive_error::DirectoryDecode)
  }

  pub(crate) fn file(&self, hash: Hash) -> Result<&[u8], ArchiveError> {
    self
      .files
      .get(&hash)
      .map(Vec::as_slice)
      .context(archive_error::FileMissing { hash })
  }

  pub(crate) fn fingerprint(&self) -> Result<Fingerprint, ArchiveError> {
    let root = self.decode_directory(None, self.root)?;

    let package = root
      .entries
      .get(Self::PACKAGE)
      .context(archive_error::PackageMissing)?;

    ensure! {
      package.ty == EntryType::Directory,
      archive_error::PackageType { ty: package.ty },
    }

    Ok(Fingerprint(package.hash))
  }

  pub(crate) fn load_with_opt_path(path: Option<&Utf8Path>) -> Result<(Utf8PathBuf, Self)> {
    let path = Manifest::opt_path(path)?;

    let archive = Self::load_with_path(&path, &path)?;

    Ok((path, archive))
  }

  pub(crate) fn load_with_path(path: &Utf8Path, display_path: &Utf8Path) -> Result<Self> {
    let cbor = filesystem::read_opt(path)?
      .ok_or_else(|| error::ManifestNotFound { path: display_path }.build())?;

    Self::decode_from_slice(&cbor).context(error::DecodeManifest { path: display_path })
  }

  pub(crate) fn pack(manifest: &Manifest) -> Self {
    let mut builder = ArchiveBuilder::new();

    let package = builder.pack_directory(&manifest.package);

    for (hash, content) in &manifest.embedded {
      builder.files.insert(*hash, content.clone());
    }

    builder.build_package(package, &manifest.signatures)
  }

  pub(crate) fn package_component() -> &'static Component {
    Component::new(Self::PACKAGE).unwrap()
  }

  pub(crate) fn signatures_component() -> &'static Component {
    Component::new(Self::SIGNATURES).unwrap()
  }

  pub(crate) fn unpack(&self) -> Result<Manifest, ArchiveError> {
    let mut loose = self.files.keys().copied().collect();

    ensure! {
      self.files.contains_key(&self.root),
      archive_error::FileMissing { hash: self.root },
    }

    for (&expected, file) in &self.files {
      let actual = Hash::bytes(file);
      ensure! {
        actual == expected,
        archive_error::FileHashMismatch { actual, expected },
      }
    }

    let root = self.decode_directory(Some(&mut loose), self.root)?;

    let package = root
      .entries
      .get(Self::PACKAGE)
      .context(archive_error::PackageMissing)?;

    ensure! {
      package.ty == EntryType::Directory,
      archive_error::PackageType { ty: package.ty },
    }

    let mut embedded = BTreeMap::new();

    let (tree, totals) = self.unpack_directory(&mut loose, &mut embedded, package, None)?;

    ensure! {
      package.totals == Some(totals),
      archive_error::TotalsMismatch { hash: package.hash },
    }

    let package = tree;

    let signatures = {
      let entry = root
        .entries
        .get(Self::SIGNATURES)
        .context(archive_error::SignaturesMissing)?;

      ensure! {
        entry.ty == EntryType::Directory,
        archive_error::SignaturesType { ty: entry.ty },
      }

      let directory = self.decode_directory(Some(&mut loose), entry.hash)?;

      let totals = Totals::new(&directory).context(archive_error::TotalsOverflow)?;

      ensure! {
        entry.totals == Some(totals),
        archive_error::TotalsMismatch { hash: entry.hash },
      }

      let mut signatures = BTreeSet::new();
      for entry in directory.entries.values() {
        loose.remove(&entry.hash);
        let signature = Signature::decode_from_slice(&self.files[&entry.hash])
          .context(archive_error::SignatureDecode)?;
        signatures.insert(signature);
      }

      signatures
    };

    ensure! {
      loose.is_empty(),
      archive_error::LooseFiles { hashes: loose },
    }

    {
      let unexpected = embedded
        .keys()
        .filter(|path| **path != Metadata::CBOR_FILENAME)
        .cloned()
        .collect::<BTreeSet<RelativePath>>();

      ensure! {
        unexpected.is_empty(),
        archive_error::UnexpectedEmbeddedFiles { paths: unexpected },
      }
    }

    let embedded = embedded
      .into_values()
      .map(|hash| (hash, self.files[&hash].clone()))
      .collect();

    Ok(Manifest {
      embedded,
      package,
      signatures,
    })
  }

  fn unpack_directory(
    &self,
    loose: &mut BTreeSet<Hash>,
    embedded: &mut BTreeMap<RelativePath, Hash>,
    entry: &Entry,
    prefix: Option<&RelativePath>,
  ) -> Result<(DirectoryTree, Totals), ArchiveError> {
    let directory = self.decode_directory(Some(loose), entry.hash)?;

    let mut entries = BTreeMap::new();
    for (name, entry) in &directory.entries {
      let crate_entry = match entry.ty {
        EntryType::File => {
          if self.files.contains_key(&entry.hash) {
            loose.remove(&entry.hash);
            embedded.insert(RelativePath::join_opt(prefix, name), entry.hash);
          }
          DirectoryTreeEntry::File(File {
            hash: entry.hash,
            size: entry.size,
          })
        }
        EntryType::Directory => {
          let (tree, totals) = self.unpack_directory(
            loose,
            embedded,
            entry,
            Some(&RelativePath::join_opt(prefix, name)),
          )?;

          ensure! {
            entry.totals == Some(totals),
            archive_error::TotalsMismatch { hash: entry.hash },
          }

          DirectoryTreeEntry::Directory(tree)
        }
      };
      entries.insert(name.clone(), crate_entry);
    }

    let totals = Totals::new(&directory).context(archive_error::TotalsOverflow)?;

    Ok((DirectoryTree { entries }, totals))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn archive_packs_metadata_cbor() {
    let content = b"foo";
    let mut package = DirectoryTree::new();
    package
      .create_file(
        &Metadata::CBOR_FILENAME.parse().unwrap(),
        File::new(content),
      )
      .unwrap();

    let manifest = Manifest {
      embedded: BTreeMap::from([(Hash::bytes(content), content.to_vec())]),
      package,
      signatures: BTreeSet::new(),
    };

    let archive = Archive::pack(&manifest);
    assert_eq!(archive.unpack().unwrap(), manifest);
  }

  #[test]
  fn decode_error() {
    let junk = b"foo".to_vec();
    let hash = Hash::bytes(&junk);
    let mut files = BTreeMap::new();
    files.insert(hash, junk);
    let archive = Archive {
      version: Version::Zero,
      root: hash,
      files,
    };
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::DirectoryDecode {
        source: DecodeError::UnexpectedType {
          expected: MajorType::Map,
          actual: MajorType::Text,
        }
      })
    );
  }

  #[test]
  fn file_hash_mismatch() {
    let mut archive = Archive::pack(&manifest());
    let &expected = archive.files.keys().next().unwrap();
    archive.files.insert(expected, b"foo".to_vec());
    let actual = Hash::bytes(b"foo");
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::FileHashMismatch { actual: a, expected: e })
        if a == actual && e == expected,
    );
  }

  fn manifest() -> Manifest {
    let mut package = DirectoryTree::new();

    package
      .create_file(&"foo".parse().unwrap(), File::new(b"bar"))
      .unwrap();

    Manifest {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::new(),
    }
  }

  #[test]
  fn missing_root() {
    let mut archive = Archive::pack(&manifest());
    let missing = Hash::bytes(&[]);
    archive.root = missing;
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::FileMissing { hash }) if hash == missing,
    );
  }

  #[test]
  fn package_missing() {
    let directory = Directory::default().encode_to_vec();
    let root = Hash::bytes(&directory);
    let mut files = BTreeMap::new();
    files.insert(root, directory);
    let archive = Archive {
      version: Version::Zero,
      root,
      files,
    };
    assert_matches!(archive.unpack(), Err(ArchiveError::PackageMissing));
  }

  #[test]
  fn round_trip_empty() {
    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
    };
    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_empty_directory() {
    let mut package = DirectoryTree::new();
    package
      .create_directory(&"foo/bar".parse().unwrap())
      .unwrap();

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::new(),
    };

    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_encode_decode() {
    let manifest = manifest();
    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_multiple_files() {
    let mut package = DirectoryTree::new();

    for (name, content) in [("foo", b"aaa"), ("bar", b"bbb"), ("baz", b"ccc")] {
      package
        .create_file(&name.parse().unwrap(), File::new(content))
        .unwrap();
    }

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::new(),
    };

    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_nested_directories() {
    let mut package = DirectoryTree::new();

    package
      .create_file(&"a/b/c".parse().unwrap(), File::new(b"foo"))
      .unwrap();

    package
      .create_file(&"a/d".parse().unwrap(), File::new(b"bar"))
      .unwrap();

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::new(),
    };

    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_pack_unpack() {
    let manifest = manifest();
    let archive = Archive::pack(&manifest);
    assert_eq!(archive.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_with_signature() {
    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
    };

    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let statement = Statement {
      fingerprint: manifest.fingerprint(),
      timestamp: None,
    };
    let signature = private_key.sign(&statement);

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: manifest.package,
      signatures: BTreeSet::from([signature]),
    };

    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn signature_decode_error() {
    let mut builder = ArchiveBuilder::new();

    let package = Directory {
      version: Version::Zero,
      entries: BTreeMap::new(),
    };

    let package = builder.directory(&package);

    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();

    let statement = Statement {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: None,
    };

    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>(3);
    map.item(0, public_key);
    map.item(1, &statement);
    map.item(2, &[0u8; 32][..]);
    drop(map);
    let signature_bytes = encoder.finish();

    let signature = builder.file(signature_bytes);

    let signatures = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([("0".parse().unwrap(), signature)]),
    };

    let signatures = builder.directory(&signatures);

    let root = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        ("package".parse().unwrap(), package),
        ("signatures".parse().unwrap(), signatures),
      ]),
    };

    let root = builder.directory(&root);

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::SignatureDecode {
        source: DecodeError::ArrayLength {
          actual: 32,
          expected: 64,
          ..
        },
      })
    );
  }

  #[test]
  fn signatures_missing() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::default());

    let root = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([("package".parse().unwrap(), package)]),
    };

    let root = builder.directory(&root);

    let archive = builder.build(root.hash);

    assert_matches!(archive.unpack(), Err(ArchiveError::SignaturesMissing));
  }

  #[test]
  fn totals_mismatch() {
    let mut builder = ArchiveBuilder::new();

    let child = builder.directory(&Directory::default());

    let child_hash = child.hash;

    let child = Entry {
      totals: Some(Totals {
        files: 1,
        ..child.totals.unwrap()
      }),
      ..child
    };

    let package = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([("foo".parse().unwrap(), child)]),
    };

    let package = builder.directory(&package);

    let archive = builder.build_package(package, &BTreeSet::new());

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::TotalsMismatch { hash }) if hash == child_hash,
    );
  }

  #[test]
  fn totals_overflow() {
    let mut builder = ArchiveBuilder::new();

    let directory = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([(
        "foo".parse().unwrap(),
        Entry {
          ty: EntryType::File,
          hash: Hash::bytes(b"foo"),
          size: u64::MAX,
          totals: None,
        },
      )]),
    };

    let package = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        ("bar".parse().unwrap(), builder.directory(&directory)),
        ("baz".parse().unwrap(), builder.directory(&directory)),
      ]),
    };

    let cbor = package.encode_to_vec();
    let hash = Hash::bytes(&cbor);
    builder.files.insert(hash, cbor.clone());

    let package = Entry {
      ty: EntryType::Directory,
      hash,
      size: cbor.len().into_u64(),
      totals: Some(Totals {
        directories: 1,
        directory_size: 0,
        file_size: 0,
        files: 0,
      }),
    };

    let archive = builder.build_package(package, &BTreeSet::new());

    assert_matches!(archive.unpack(), Err(ArchiveError::TotalsOverflow));
  }

  #[test]
  fn unexpected_embedded_files() {
    let content = b"foo";

    let mut package = DirectoryTree::new();
    for path in &["bar/bob", "baz"] {
      package
        .create_file(&path.parse().unwrap(), File::new(content))
        .unwrap();
    }

    let mut builder = ArchiveBuilder::new();

    let package = builder.pack_directory(&package);

    builder.files.insert(Hash::bytes(content), content.to_vec());

    let signatures = builder.directory(&Directory::default());

    let root = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        ("package".parse().unwrap(), package),
        ("signatures".parse().unwrap(), signatures),
      ]),
    };

    let root = builder.directory(&root);

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::UnexpectedEmbeddedFiles { paths })
        if paths.to_string() == "`bar/bob`, `baz`",
    );
  }

  #[test]
  fn unreferenced_files() {
    let mut archive = Archive::pack(&manifest());
    let file = b"foo".to_vec();
    let hash = Hash::bytes(&file);
    archive.files.insert(hash, file);
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::LooseFiles { hashes: Ticked(hashes) })
        if hashes == BTreeSet::from([hash]),
    );
  }
}
