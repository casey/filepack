use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Archive {
  pub(crate) version: Version,
  pub(crate) root: Hash,
  pub(crate) files: BTreeMap<Hash, Vec<u8>>,
}

impl Archive {
  const PACKAGE: &str = "package";
  const SIGNATURES: &str = "signatures";

  pub(crate) fn archive(manifest: &Manifest) -> Self {
    let mut builder = ArchiveBuilder::new();

    let package = builder.directory(&manifest.files);

    let mut root = BTreeMap::new();

    root.insert(Self::PACKAGE.parse::<Component>().unwrap(), package);

    let mut entries = BTreeMap::new();
    for (i, signature) in manifest.signatures.iter().enumerate() {
      let signature = signature.to_string().into_bytes();
      entries.insert(
        i.to_string().parse::<Component>().unwrap(),
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

    Directory::decode_from_vec(file.clone()).context(archive_error::Decode)
  }

  pub(crate) fn fingerprint(&self) -> Fingerprint {
    let root = &self.files[&self.root];
    let root = Directory::decode_from_vec(root.clone()).unwrap();
    Fingerprint(root.entries[&Self::PACKAGE.parse::<Component>().unwrap()].hash)
  }

  pub(crate) fn unarchive(&self) -> Result<Manifest, ArchiveError> {
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

    let files = self.unarchive_directory(&mut loose, package)?;

    let signatures_entry = root
      .entries
      .get(Self::SIGNATURES)
      .context(archive_error::SignaturesMissing)?;

    let signatures_dir = self.decode_directory(&mut loose, signatures_entry.hash)?;

    let mut signatures = BTreeSet::new();
    for entry in signatures_dir.entries.values() {
      loose.remove(&entry.hash);
      let bytes = &self.files[&entry.hash];
      let s = str::from_utf8(bytes)
        .context(decode_error::Unicode)
        .context(archive_error::Decode)?;
      signatures.insert(
        s.parse::<Signature>()
          .context(archive_error::SignatureParse)?,
      );
    }

    if !loose.is_empty() {
      return Err(archive_error::UnreferencedFiles { hashes: loose }.build());
    }

    Ok(Manifest { files, signatures })
  }

  fn unarchive_directory(
    &self,
    loose: &mut BTreeSet<Hash>,
    entry: &Entry,
  ) -> Result<DirectoryTree, ArchiveError> {
    let dir = self.decode_directory(loose, entry.hash)?;

    let mut entries = BTreeMap::new();

    for (name, entry) in &dir.entries {
      let crate_entry = match entry.ty {
        EntryType::File => DirectoryTreeEntry::File(File {
          hash: entry.hash,
          size: entry.size,
        }),
        EntryType::Directory => {
          DirectoryTreeEntry::Directory(self.unarchive_directory(loose, entry)?)
        }
      };
      entries.insert(name.clone(), crate_entry);
    }

    Ok(DirectoryTree { entries })
  }
}

impl Encode for Archive {
  fn encode(&self, encoder: &mut Encoder) {
    let mut encoder = encoder.map::<u8>(3);
    encoder.item(0, self.version);
    encoder.item(1, self.root);
    encoder.item(2, &self.files);
  }
}

impl Decode for Archive {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.map::<u8>()?;

    let version = decoder.required_key(0)?;
    let root = decoder.required_key(1)?;
    let files = decoder.required_key::<BTreeMap<Hash, Vec<u8>>>(2)?;

    Ok(Self {
      version,
      root,
      files,
    })
  }
}
