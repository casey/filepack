use super::*;

pub(crate) struct ArchiveBuilder {
  pub(crate) files: BTreeMap<Hash, Vec<u8>>,
}

impl ArchiveBuilder {
  pub(crate) fn build_package(
    mut self,
    package: Entry,
    signatures: &BTreeSet<Signature>,
  ) -> Archive {
    let mut root = BTreeMap::new();

    root.insert(Archive::package_component().to_owned(), package);

    let mut entries = BTreeMap::new();
    for (i, signature) in signatures.iter().enumerate() {
      entries.insert(
        i.to_string().parse::<ComponentBuf>().unwrap(),
        self.entry(EntryType::File, signature.encode_to_vec()),
      );
    }

    let signatures = Directory {
      entries,
      version: Version::Zero,
    };

    let signatures = self.entry(EntryType::Directory, signatures.encode_to_vec());

    root.insert(Archive::signatures_component().to_owned(), signatures);

    let root = Directory {
      entries: root,
      version: Version::Zero,
    };

    let entry = self.entry(EntryType::Directory, root.encode_to_vec());

    self.build(entry.hash)
  }

  pub(crate) fn build(self, root: Hash) -> Archive {
    Archive {
      files: self.files,
      root,
      version: Version::Zero,
    }
  }

  pub(crate) fn directory(&mut self, directory: &DirectoryTree) -> Entry {
    let directory = Directory {
      version: Version::Zero,
      entries: directory
        .entries
        .iter()
        .map(|(name, entry)| {
          let entry = match entry {
            DirectoryTreeEntry::File(file) => Entry {
              hash: file.hash,
              size: file.size,
              ty: EntryType::File,
            },
            DirectoryTreeEntry::Directory(directory) => self.directory(directory),
          };

          (name.clone(), entry)
        })
        .collect(),
    };

    self.entry(EntryType::Directory, directory.encode_to_vec())
  }

  pub(crate) fn entry(&mut self, ty: EntryType, file: Vec<u8>) -> Entry {
    let size = file.len().into_u64();
    let hash = Hash::bytes(&file);
    self.files.insert(hash, file);
    Entry { ty, hash, size }
  }

  pub(crate) fn new() -> Self {
    Self {
      files: BTreeMap::new(),
    }
  }
}
