use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Encode, Decode, Magic)]
#[deco(magic = MagicType::Archive)]
pub struct Archive {
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
    options: DecodeOptions,
    loose: &mut BTreeSet<Hash>,
    hash: Hash,
    size: u64,
    totals: Totals,
  ) -> Result<Directory, ArchiveError> {
    let file = self.file(hash, size)?;

    loose.remove(&hash);

    let directory = Directory::decode_from_slice_with_options(options, file)
      .context(archive_error::DirectoryDecode)?;

    directory
      .totals()
      .context(archive_error::DirectoryTotals { hash })?
      .expect(totals)
      .context(archive_error::DirectoryTotals { hash })?;

    Ok(directory)
  }

  fn decode_root(&self, options: DecodeOptions) -> Result<Directory, ArchiveError> {
    let file = self
      .files
      .get(&self.root)
      .context(archive_error::FileMissing { hash: self.root })?;

    Directory::decode_from_slice_with_options(options, file).context(archive_error::DirectoryDecode)
  }

  pub(crate) fn file(&self, hash: Hash, size: u64) -> Result<&[u8], ArchiveError> {
    let file = self
      .files
      .get(&hash)
      .map(Vec::as_slice)
      .context(archive_error::FileMissing { hash })?;

    let actual = file.len().into_u64();

    ensure! {
      actual == size,
      archive_error::FileSizeMismatch {
        actual,
        expected: size,
        hash,
      },
    }

    Ok(file)
  }

  pub(crate) fn fingerprint(&self) -> Result<Fingerprint, ArchiveError> {
    self.fingerprint_with_options(DecodeOptions::default())
  }

  pub(crate) fn fingerprint_with_options(
    &self,
    options: DecodeOptions,
  ) -> Result<Fingerprint, ArchiveError> {
    Ok(Fingerprint(self.package(options)?.hash))
  }

  pub(crate) fn pack(manifest: &Manifest) -> Result<Self, TotalsError> {
    let mut builder = ArchiveBuilder::new();

    let package = builder.pack_directory(&manifest.package)?;

    let unknown = builder.pack_entries(&manifest.unknown)?;

    for (hash, content) in &manifest.embedded {
      builder.files.insert(*hash, content.clone());
    }

    builder.build_package(package, &manifest.signatures, unknown)
  }

  pub(crate) fn package(&self, options: DecodeOptions) -> Result<Entry, ArchiveError> {
    let root = self.decode_root(options)?;

    let package = root
      .entries
      .get(Self::PACKAGE)
      .context(archive_error::PackageMissing)?;

    ensure! {
      package.ty() == EntryType::Directory,
      archive_error::PackageType { ty: package.ty() },
    }

    Ok(*package)
  }

  pub(crate) fn package_component() -> &'static Component {
    Component::new(Self::PACKAGE).unwrap()
  }

  pub(crate) fn root_entries() -> [&'static Component; 2] {
    [Self::package_component(), Self::signatures_component()]
  }

  pub(crate) fn signatures_component() -> &'static Component {
    Component::new(Self::SIGNATURES).unwrap()
  }

  #[cfg(test)]
  pub(crate) fn unpack(&self) -> Result<Manifest, ArchiveError> {
    self.unpack_with_options(DecodeOptions::strict())
  }

  fn unpack_directory(
    &self,
    options: DecodeOptions,
    loose: &mut BTreeSet<Hash>,
    embedded: &mut BTreeMap<Hash, Vec<u8>>,
    hash: Hash,
    size: u64,
    expected_totals: Totals,
  ) -> Result<DirectoryTree, ArchiveError> {
    let directory = self.decode_directory(options, loose, hash, size, expected_totals)?;

    let mut entries = BTreeMap::new();
    for (name, entry) in &directory.entries {
      entries.insert(
        name.clone(),
        self.unpack_entry(options, loose, embedded, entry)?,
      );
    }

    Ok(DirectoryTree { entries })
  }

  fn unpack_entry(
    &self,
    options: DecodeOptions,
    loose: &mut BTreeSet<Hash>,
    embedded: &mut BTreeMap<Hash, Vec<u8>>,
    entry: &Entry,
  ) -> Result<DirectoryTreeEntry, ArchiveError> {
    match entry.info {
      EntryInfo::File => {
        if self.files.contains_key(&entry.hash) {
          let content = self.file(entry.hash, entry.size)?;
          loose.remove(&entry.hash);
          embedded.insert(entry.hash, content.to_vec());
        }
        Ok(DirectoryTreeEntry::File(File {
          hash: entry.hash,
          size: entry.size,
        }))
      }
      EntryInfo::Directory { totals } => Ok(DirectoryTreeEntry::Directory(
        self.unpack_directory(options, loose, embedded, entry.hash, entry.size, totals)?,
      )),
    }
  }

  pub(crate) fn unpack_with_options(
    &self,
    options: DecodeOptions,
  ) -> Result<Manifest, ArchiveError> {
    Ok(self.unpack_with_totals(options)?.0)
  }

  pub(crate) fn unpack_with_totals(
    &self,
    options: DecodeOptions,
  ) -> Result<(Manifest, Totals), ArchiveError> {
    let mut loose = self.files.keys().copied().collect::<BTreeSet<Hash>>();

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

    let root = self.decode_root(options)?;

    loose.remove(&self.root);

    let mut embedded = BTreeMap::new();

    let mut unknown = DirectoryTree::new();
    for (name, entry) in &root.entries {
      if !Self::root_entries().contains(&&**name) {
        unknown.entries.insert(
          name.clone(),
          self.unpack_entry(options, &mut loose, &mut embedded, entry)?,
        );
      }
    }

    let package = root
      .entries
      .get(Self::PACKAGE)
      .context(archive_error::PackageMissing)?;

    let EntryInfo::Directory { totals } = package.info else {
      return Err(ArchiveError::PackageType { ty: package.ty() });
    };

    let package = self.unpack_directory(
      options,
      &mut loose,
      &mut embedded,
      package.hash,
      package.size,
      totals,
    )?;

    for name in package.entries.keys() {
      ensure! {
        !name.is_reserved(),
        archive_error::ReservedPath { name },
      }
    }

    let signatures = {
      let entry = root
        .entries
        .get(Self::SIGNATURES)
        .context(archive_error::SignaturesMissing)?;

      let EntryInfo::Directory { totals } = entry.info else {
        return Err(ArchiveError::SignaturesType { ty: entry.ty() });
      };

      let directory = self.decode_directory(options, &mut loose, entry.hash, entry.size, totals)?;

      let mut signatures = BTreeSet::new();
      for (name, entry) in &directory.entries {
        ensure! {
          *name == ComponentBuf::from_hash(entry.hash),
          archive_error::SignatureName {
            hash: entry.hash,
            name,
          },
        }

        match entry.ty() {
          EntryType::File => {
            let file = self.file(entry.hash, entry.size)?;
            loose.remove(&entry.hash);
            let signature = Decoded::<Attestation>::decode_from_slice_with_options(options, file)
              .context(archive_error::SignatureDecode)?;
            signatures.insert(signature);
          }
          EntryType::Directory => return Err(ArchiveError::SignaturesDirectory),
        }
      }

      signatures
    };

    ensure! {
      loose.is_empty(),
      archive_error::LooseFiles { hashes: loose },
    }

    Ok((
      Manifest {
        embedded,
        package,
        signatures,
        unknown,
      },
      totals,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn archive_packs_metadata_deco() {
    let content = b"foo";
    let mut package = DirectoryTree::new();
    package
      .create_file(
        &Metadata::DECO_FILENAME.parse().unwrap(),
        File::new(content),
      )
      .unwrap();

    let manifest = Manifest {
      embedded: BTreeMap::from([(Hash::bytes(content), content.to_vec())]),
      package,
      signatures: BTreeSet::new(),
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);
  }

  #[test]
  fn decode_error() {
    let junk = b"foo".to_vec();
    let hash = Hash::bytes(&junk);
    let mut files = BTreeMap::new();
    files.insert(hash, junk);
    let archive = Archive { root: hash, files };
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::DirectoryDecode {
        source: DecodeError::Malformed(MalformedError::Truncated)
      })
    );
  }

  #[test]
  fn directory_totals_overflow() {
    let mut builder = ArchiveBuilder::new();

    let (deco, hash) = Directory::new()
      .insert_entry("bar", Entry::file(Hash::bytes(b"bar"), u64::MAX))
      .insert_entry("foo", Entry::file(Hash::bytes(b"foo"), 1))
      .deco();

    let size = deco.len().into_u64();
    builder.files.insert(hash, deco);
    let package = Entry::directory(hash, size, Totals::default());

    let signatures = builder.directory(&Directory::new()).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::DirectoryTotals { hash: h, source: TotalsError::Overflow }) if h == hash,
    );
  }

  #[test]
  fn embedded_file_size_mismatch() {
    let mut builder = ArchiveBuilder::new();

    let hash = Hash::bytes(b"foo");
    builder.files.insert(hash, b"foo".to_vec());

    let mut package = Directory::new();
    package.insert_entry("foo", Entry::file(hash, 100));

    let package = builder.directory(&package).unwrap();

    let signatures = builder.directory(&Directory::new()).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::FileSizeMismatch {
        actual: 3,
        expected: 100,
        hash: h,
      }) if h == hash,
    );
  }

  #[test]
  fn file_hash_mismatch() {
    let mut archive = Archive::pack(&manifest()).unwrap();
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
      unknown: DirectoryTree::new(),
    }
  }

  #[test]
  fn missing_root() {
    let mut archive = Archive::pack(&manifest()).unwrap();
    let missing = Hash::bytes(&[]);
    archive.root = missing;
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::FileMissing { hash }) if hash == missing,
    );
  }

  #[test]
  fn pack_totals_overflow() {
    let mut package = DirectoryTree::new();

    package
      .create_file(
        &"bar".parse().unwrap(),
        File {
          hash: Hash::bytes(b"bar"),
          size: u64::MAX,
        },
      )
      .unwrap();

    package
      .create_file(
        &"foo".parse().unwrap(),
        File {
          hash: Hash::bytes(b"foo"),
          size: 1,
        },
      )
      .unwrap();

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::new(),
      unknown: DirectoryTree::new(),
    };

    assert_eq!(Archive::pack(&manifest).err(), Some(TotalsError::Overflow));
  }

  #[test]
  fn package_missing() {
    let directory = Directory::default().encode_to_vec();
    let root = Hash::bytes(&directory);
    let mut files = BTreeMap::new();
    files.insert(root, directory);
    let archive = Archive { root, files };
    assert_matches!(archive.unpack(), Err(ArchiveError::PackageMissing));
  }

  #[test]
  fn package_totals_mismatch() {
    let mut builder = ArchiveBuilder::new();

    let mut package = Directory::new();
    package.insert_file("foo", b"bar");

    let entry = builder.directory(&package).unwrap();

    let package = Entry::directory(
      entry.hash,
      entry.size,
      Totals {
        directories: 0,
        directory_size: 0,
        file_size: 100,
        files: 1,
      },
    );

    let signatures = builder.directory(&Directory::default()).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::DirectoryTotals {
        hash,
        source: TotalsError::Mismatch {
          actual: Totals {
            directories: 0,
            directory_size: 0,
            file_size: 3,
            files: 1,
          },
          expected: Totals {
            directories: 0,
            directory_size: 0,
            file_size: 100,
            files: 1,
          },
        },
      }) if hash == entry.hash,
    );
  }

  #[test]
  fn reserved_path() {
    let mut builder = ArchiveBuilder::new();

    let mut package = Directory::new();
    package.insert_file("manifest.filepack", b"foo");

    let package = builder.directory(&package).unwrap();

    let signatures = builder.directory(&Directory::new()).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::ReservedPath { name }) if name == "manifest.filepack",
    );
  }

  #[test]
  fn root_totals_overflow() {
    let mut package = DirectoryTree::new();

    package
      .create_file(
        &"foo".parse().unwrap(),
        File {
          hash: Hash::bytes(b"bar"),
          size: u64::MAX,
        },
      )
      .unwrap();

    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();

    let statement = Statement {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: None,
    };

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package,
      signatures: BTreeSet::from([Decoded::Known(private_key.sign(statement))]),
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);
  }

  #[track_caller]
  fn round_trip(manifest: &Manifest) {
    let bytes = Archive::pack(manifest).unwrap().encode_to_vec();
    let archive = Archive::decode_strict(&bytes).unwrap();
    assert_eq!(archive.unpack().unwrap(), *manifest);
  }

  #[test]
  fn round_trip_empty() {
    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
      unknown: DirectoryTree::new(),
    };
    round_trip(&manifest);
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
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);
  }

  #[test]
  fn round_trip_encode_decode() {
    let manifest = manifest();
    round_trip(&manifest);
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
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);
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
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);
  }

  #[test]
  fn round_trip_pack_unpack() {
    let manifest = manifest();
    round_trip(&manifest);
  }

  #[test]
  fn round_trip_with_signature() {
    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
      unknown: DirectoryTree::new(),
    };

    let fingerprint = Archive::pack(&manifest).unwrap().fingerprint().unwrap();

    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let statement = Statement {
      fingerprint,
      timestamp: None,
    };
    let signature = private_key.sign(statement);

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: manifest.package,
      signatures: BTreeSet::from([Decoded::Known(signature)]),
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);
  }

  #[test]
  fn signature_decode_error() {
    let mut builder = ArchiveBuilder::new();

    let package = Directory::new();

    let package = builder.directory(&package).unwrap();

    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();

    let statement = Statement {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: None,
    };

    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u64>();
    map.item(3, &[0u8; 32][..]);
    map.item(2, &statement);
    map.item(1, public_key);
    map.finish();
    let signature_bytes = encoder.finish();

    let signature = builder.file(signature_bytes);

    let mut signatures = Directory::new();
    signatures.insert_entry(&signature.hash.to_string(), signature);

    let signatures = builder.directory(&signatures).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::SignatureDecode {
        source: DecodeError::Malformed(MalformedError::ArrayLength {
          actual: 32,
          expected: 64,
          ..
        }),
      })
    );
  }

  #[test]
  fn signature_file_missing() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::new()).unwrap();

    let missing_file = Hash::bytes(b"foo");

    let mut signatures = Directory::new();
    signatures.insert_entry(&missing_file.to_string(), Entry::file(missing_file, 0));

    let signatures = builder.directory(&signatures).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::FileMissing { hash }) if hash == missing_file,
    );
  }

  #[test]
  fn signature_name_mismatch() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::new()).unwrap();

    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();

    let signature = builder.file(
      private_key
        .sign(Statement {
          fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
          timestamp: None,
        })
        .encode_to_vec(),
    );

    let mut signatures = Directory::new();
    signatures.insert_entry("foo", signature);

    let signatures = builder.directory(&signatures).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::SignatureName { hash, name })
        if hash == signature.hash && name == "foo",
    );
  }

  #[test]
  fn signatures_are_named_by_hash() {
    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();

    let known = Decoded::Known(private_key.sign(Statement {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: None,
    }));

    let unknown = {
      let mut fields = BTreeMap::<u64, Vec<u8>>::decode_from_slice(&known.encode_to_vec()).unwrap();
      assert!(fields.insert(4, b"foo".to_vec()).is_none());
      Decoded::<Attestation>::Unknown(
        Vec::<u8>::decode_from_slice(&fields.encode_to_vec()).unwrap(),
      )
    };

    let known_bytes = known.encode_to_vec();
    let unknown_bytes = unknown.encode_to_vec();

    let manifest = Manifest {
      embedded: BTreeMap::new(),
      package: DirectoryTree::new(),
      signatures: BTreeSet::from([known, unknown]),
      unknown: DirectoryTree::new(),
    };

    round_trip(&manifest);

    let archive = Archive::pack(&manifest).unwrap();

    let root = archive.decode_root(DecodeOptions::strict()).unwrap();

    let signatures = root.entries.get(Archive::SIGNATURES).unwrap();

    let signatures =
      Directory::decode_from_slice(archive.file(signatures.hash, signatures.size).unwrap())
        .unwrap();

    assert_eq!(
      signatures.entries,
      BTreeMap::from([
        (
          ComponentBuf::from_hash(Hash::bytes(&known_bytes)),
          Entry::file(Hash::bytes(&known_bytes), known_bytes.len().into_u64()),
        ),
        (
          ComponentBuf::from_hash(Hash::bytes(&unknown_bytes)),
          Entry::file(Hash::bytes(&unknown_bytes), unknown_bytes.len().into_u64()),
        ),
      ]),
    );
  }

  #[test]
  fn signatures_directory() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::new()).unwrap();

    let subdirectory = builder.directory(&Directory::new()).unwrap();

    let mut signatures = Directory::new();
    signatures.insert_entry(&subdirectory.hash.to_string(), subdirectory);

    let signatures = builder.directory(&signatures).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(archive.unpack(), Err(ArchiveError::SignaturesDirectory));
  }

  #[test]
  fn signatures_missing() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::default()).unwrap();

    let mut root = Directory::new();
    root.insert_entry("package", package);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(archive.unpack(), Err(ArchiveError::SignaturesMissing));
  }

  #[test]
  fn signatures_totals_mismatch() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::new()).unwrap();

    let entry = builder.directory(&Directory::new()).unwrap();

    let signatures = Entry::directory(
      entry.hash,
      entry.size,
      Totals {
        directories: 0,
        directory_size: 0,
        file_size: 1,
        files: 1,
      },
    );

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures);

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::DirectoryTotals {
        hash,
        source: TotalsError::Mismatch {
          actual: Totals {
            directories: 0,
            directory_size: 0,
            file_size: 0,
            files: 0,
          },
          expected: Totals {
            directories: 0,
            directory_size: 0,
            file_size: 1,
            files: 1,
          },
        }
      }) if hash == entry.hash,
    );
  }

  #[test]
  fn unknown_entries() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&Directory::new()).unwrap();

    let signatures = builder.directory(&Directory::new()).unwrap();

    let mut bar = Directory::new();
    bar.insert_entry("baz", builder.file(b"qux".to_vec()));

    let bar = builder.directory(&bar).unwrap();

    let mut root = Directory::new();
    root
      .insert_entry("package", package)
      .insert_entry("signatures", signatures)
      .insert_entry("bar", bar)
      .insert_file("foo", b"bar");

    let root = builder.directory(&root).unwrap();

    let archive = builder.build(root.hash);

    let mut unknown = DirectoryTree::new();

    unknown
      .create_file(&"bar/baz".parse().unwrap(), File::new(b"qux"))
      .unwrap();

    unknown
      .create_file(&"foo".parse().unwrap(), File::new(b"bar"))
      .unwrap();

    let manifest = Manifest {
      embedded: BTreeMap::from([(Hash::bytes(b"qux"), b"qux".to_vec())]),
      package: DirectoryTree::new(),
      signatures: BTreeSet::new(),
      unknown,
    };

    assert_eq!(
      archive
        .unpack_with_options(DecodeOptions::default())
        .unwrap(),
      manifest,
    );

    assert_eq!(archive.unpack().unwrap(), manifest);

    assert_eq!(
      Archive::pack(&manifest).unwrap().encode_to_vec(),
      archive.encode_to_vec(),
    );
  }

  #[test]
  fn unreferenced_files() {
    let mut archive = Archive::pack(&manifest()).unwrap();
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
