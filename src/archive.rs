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
    loose: &mut BTreeSet<Hash>,
    hash: Hash,
  ) -> Result<Directory, ArchiveError> {
    let file = self
      .files
      .get(&hash)
      .context(archive_error::FileMissing { hash })?;

    loose.remove(&hash);

    Directory::decode_from_slice(file).context(archive_error::DirectoryDecode)
  }

  pub(crate) fn fingerprint(&self) -> Fingerprint {
    let root = &self.files[&self.root];
    let root = Directory::decode_from_slice(root).unwrap();
    Fingerprint(root.entries[Self::PACKAGE].hash)
  }

  pub(crate) fn pack(manifest: &Manifest) -> Self {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&manifest.files);

    let mut root = BTreeMap::new();

    root.insert(Self::PACKAGE.parse::<ComponentBuf>().unwrap(), package);

    let mut entries = BTreeMap::new();
    for (i, signature) in manifest.signatures.iter().enumerate() {
      let signature = signature.to_string().into_bytes();
      entries.insert(
        i.to_string().parse::<ComponentBuf>().unwrap(),
        builder.entry(EntryType::File, signature),
      );
    }

    let signatures = Directory {
      entries,
      version: Version::Zero,
    };

    let signatures = builder.entry(EntryType::Directory, signatures.encode_to_vec());

    root.insert(Self::SIGNATURES.parse().unwrap(), signatures);

    let root = Directory {
      entries: root,
      version: Version::Zero,
    };

    let entry = builder.entry(EntryType::Directory, root.encode_to_vec());

    builder.build(entry.hash)
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

    let root = self.decode_directory(&mut loose, self.root)?;

    let package = root
      .entries
      .get(Self::PACKAGE)
      .context(archive_error::PackageMissing)?;

    let files = self.unpack_directory(&mut loose, package)?;

    let signatures = {
      let entry = root
        .entries
        .get(Self::SIGNATURES)
        .context(archive_error::SignaturesMissing)?;

      let directory = self.decode_directory(&mut loose, entry.hash)?;

      let mut signatures = BTreeSet::new();
      for entry in directory.entries.values() {
        loose.remove(&entry.hash);
        let bytes = &self.files[&entry.hash];
        let s = str::from_utf8(bytes)
          .context(decode_error::Unicode)
          .context(archive_error::SignatureDecode)?;
        signatures.insert(
          s.parse::<Signature>()
            .context(archive_error::SignatureParse)?,
        );
      }

      signatures
    };

    if !loose.is_empty() {
      return Err(archive_error::UnreferencedFiles { hashes: loose }.build());
    }

    Ok(Manifest { files, signatures })
  }

  fn unpack_directory(
    &self,
    loose: &mut BTreeSet<Hash>,
    entry: &Entry,
  ) -> Result<DirectoryTree, ArchiveError> {
    let directory = self.decode_directory(loose, entry.hash)?;

    let mut entries = BTreeMap::new();
    for (name, entry) in &directory.entries {
      let crate_entry = match entry.ty {
        EntryType::File => DirectoryTreeEntry::File(File {
          hash: entry.hash,
          size: entry.size,
        }),
        EntryType::Directory => DirectoryTreeEntry::Directory(self.unpack_directory(loose, entry)?),
      };
      entries.insert(name.clone(), crate_entry);
    }

    Ok(DirectoryTree { entries })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
    let mut files = DirectoryTree::new();

    files
      .create_file(&"foo".parse().unwrap(), File::new(b"bar"))
      .unwrap();

    Manifest {
      files,
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
      files: DirectoryTree::new(),
      signatures: BTreeSet::new(),
    };
    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_empty_directory() {
    let mut files = DirectoryTree::new();
    files.create_directory(&"foo/bar".parse().unwrap()).unwrap();

    let manifest = Manifest {
      files,
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
    let mut files = DirectoryTree::new();

    for (name, content) in [("foo", b"aaa"), ("bar", b"bbb"), ("baz", b"ccc")] {
      files
        .create_file(&name.parse().unwrap(), File::new(content))
        .unwrap();
    }

    let manifest = Manifest {
      files,
      signatures: BTreeSet::new(),
    };

    let archive = Archive::pack(&manifest);
    let bytes = archive.encode_to_vec();
    let decoded = Archive::decode_from_slice(&bytes).unwrap();
    assert_eq!(decoded.unpack().unwrap(), manifest);
  }

  #[test]
  fn round_trip_nested_directories() {
    let mut files = DirectoryTree::new();

    files
      .create_file(&"a/b/c".parse().unwrap(), File::new(b"foo"))
      .unwrap();

    files
      .create_file(&"a/d".parse().unwrap(), File::new(b"bar"))
      .unwrap();

    let manifest = Manifest {
      files,
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
      files: DirectoryTree::new(),
      signatures: BTreeSet::new(),
    };

    let private_key = test::PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let message = Message {
      fingerprint: manifest.fingerprint(),
      timestamp: None,
    };
    let signature = private_key.sign(&message);

    let manifest = Manifest {
      files: manifest.files,
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

    let package = builder.entry(EntryType::Directory, package.encode_to_vec());

    let signature = builder.entry(EntryType::File, b"\xff".to_vec());

    let signatures = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([("0".parse().unwrap(), signature)]),
    };

    let signatures = builder.entry(EntryType::Directory, signatures.encode_to_vec());

    let root = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        ("package".parse().unwrap(), package),
        ("signatures".parse().unwrap(), signatures),
      ]),
    };

    let root = builder.entry(EntryType::Directory, root.encode_to_vec());

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::SignatureDecode {
        source: DecodeError::Unicode { .. }
      })
    );
  }

  #[test]
  fn signature_parse_error() {
    let mut builder = ArchiveBuilder::new();

    let package = Directory {
      version: Version::Zero,
      entries: BTreeMap::new(),
    };

    let package = builder.entry(EntryType::Directory, package.encode_to_vec());

    let signature = builder.entry(EntryType::File, b"not-a-signature".to_vec());

    let signatures = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([("0".parse().unwrap(), signature)]),
    };

    let signatures = builder.entry(EntryType::Directory, signatures.encode_to_vec());

    let root = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([
        ("package".parse().unwrap(), package),
        ("signatures".parse().unwrap(), signatures),
      ]),
    };

    let root = builder.entry(EntryType::Directory, root.encode_to_vec());

    let archive = builder.build(root.hash);

    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::SignatureParse {
        source: SignatureError::Bech32 { .. }
      })
    );
  }

  #[test]
  fn signatures_missing() {
    let mut builder = ArchiveBuilder::new();

    let package = builder.entry(EntryType::Directory, Directory::default().encode_to_vec());

    let root = Directory {
      version: Version::Zero,
      entries: BTreeMap::from([("package".parse().unwrap(), package)]),
    };

    let root = builder.entry(EntryType::Directory, root.encode_to_vec());

    let archive = builder.build(root.hash);

    assert_matches!(archive.unpack(), Err(ArchiveError::SignaturesMissing));
  }

  #[test]
  fn unreferenced_files() {
    let mut archive = Archive::pack(&manifest());
    let file = b"foo".to_vec();
    let hash = Hash::bytes(&file);
    archive.files.insert(hash, file);
    assert_matches!(
      archive.unpack(),
      Err(ArchiveError::UnreferencedFiles { hashes: Ticked(hashes) })
        if hashes == BTreeSet::from([hash]),
    );
  }
}
