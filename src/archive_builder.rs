use super::*;

#[derive(Default)]
pub(crate) struct ArchiveBuilder {
  pub(crate) files: BTreeMap<Hash, Vec<u8>>,
}

impl ArchiveBuilder {
  fn add_file(&mut self, file: Vec<u8>) -> (Hash, u64) {
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
        ComponentBuf::from_integer(i),
        self.file(signature.encode_to_vec()),
      );
    }

    let signatures = Directory::with_entries(entries);

    let signatures = self.directory(&signatures);

    root.insert(Archive::signatures_component().to_owned(), signatures);

    let root = Directory::with_entries(root);

    let (hash, _size) = self.add_file(root.encode_to_vec());

    self.build(hash)
  }

  pub(crate) fn directory(&mut self, directory: &Directory) -> Entry {
    let (hash, size) = self.add_file(directory.encode_to_vec());
    Entry::Directory {
      hash,
      size,
      total_file_size: directory.total_file_size().unwrap(),
    }
  }

  pub(crate) fn file(&mut self, file: Vec<u8>) -> Entry {
    let (hash, size) = self.add_file(file);

    Entry::file(hash, size)
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn pack_directory(&mut self, directory: &DirectoryTree) -> Entry {
    let directory = Directory::with_entries(
      directory
        .entries
        .iter()
        .map(|(name, entry)| {
          let entry = match entry {
            DirectoryTreeEntry::File(file) => Entry::file(file.hash, file.size),
            DirectoryTreeEntry::Directory(directory) => self.pack_directory(directory),
          };

          (name.clone(), entry)
        })
        .collect(),
    );

    self.directory(&directory)
  }
}
