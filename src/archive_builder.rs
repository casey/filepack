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
        self.file(signature.encode_to_vec()),
      );
    }

    let signatures = Directory {
      entries,
      version: Version::Zero,
    };

    let signatures = self.directory(&signatures);

    root.insert(Archive::signatures_component().to_owned(), signatures);

    let root = Directory {
      entries: root,
      version: Version::Zero,
    };

    let entry = self.directory(&root);

    self.build(entry.hash)
  }

  pub(crate) fn directory(&mut self, directory: &Directory) -> Entry {
    let (hash, size) = self.add(directory.encode_to_vec());

    let totals = Totals::directory(directory).expect("totals overflow");

    Entry {
      ty: EntryType::Directory,
      hash,
      size,
      totals: Some(totals),
    }
  }

  pub(crate) fn file(&mut self, file: Vec<u8>) -> Entry {
    let (hash, size) = self.add(file);

    Entry {
      ty: EntryType::File,
      hash,
      size,
      totals: None,
    }
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn pack_directory(&mut self, directory: &DirectoryTree) -> Entry {
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
              totals: None,
            },
            DirectoryTreeEntry::Directory(directory) => self.pack_directory(directory),
          };

          (name.clone(), entry)
        })
        .collect(),
    };

    self.directory(&directory)
  }
}
