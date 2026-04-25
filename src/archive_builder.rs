use super::*;

pub(crate) struct ArchiveBuilder {
  pub(crate) files: BTreeMap<Hash, Vec<u8>>,
}

impl ArchiveBuilder {
  pub(crate) fn build(self, root: Hash) -> Archive {
    Archive {
      application: Application::Filepack,
      context: Context::Manifest,
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
