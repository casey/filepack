use super::*;

#[derive(Default)]
pub(crate) struct ArchiveBuilder {
  pub(crate) files: BTreeMap<Hash, Vec<u8>>,
}

impl ArchiveBuilder {
  fn add(&mut self, file: Vec<u8>) -> (Hash, u64) {
    let size = file.len().into_u64();
    let hash = Hash::bytes(&file);
    self.files.insert(hash, file);
    (hash, size)
  }

  pub(crate) fn build(self, root: Hash) -> Archive {
    Archive {
      files: self.files,
      root,
      version: Version::Zero,
    }
  }

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
        self.file_entry(signature.encode_to_vec()),
      );
    }

    let signatures = Directory {
      entries,
      version: Version::Zero,
    };

    let signatures = self.directory_entry(&signatures);

    root.insert(Archive::signatures_component().to_owned(), signatures);

    let root = Directory {
      entries: root,
      version: Version::Zero,
    };

    let entry = self.directory_entry(&root);

    self.build(entry.hash)
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
              ty: EntryType::File,
              hash: file.hash,
              size: file.size,
            },
            DirectoryTreeEntry::Directory(directory) => self.directory(directory),
          };

          (name.clone(), entry)
        })
        .collect(),
    };

    self.directory_entry(&directory)
  }

  pub(crate) fn directory_entry(&mut self, directory: &Directory) -> Entry {
    let (hash, size) = self.add(directory.encode_to_vec());

    Entry {
      ty: EntryType::Directory,
      hash,
      size,
    }
  }

  pub(crate) fn file_entry(&mut self, file: Vec<u8>) -> Entry {
    let (hash, size) = self.add(file);

    Entry {
      ty: EntryType::File,
      hash,
      size,
    }
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }
}
