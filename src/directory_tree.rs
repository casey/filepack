use super::*;

/// Directory tree of subdirectories and file content.
#[serde_as]
#[derive(Debug, Default, Deserialize, Json, PartialEq, Serialize)]
#[serde(transparent)]
pub struct DirectoryTree {
  #[serde_as(as = "MapPreventDuplicates<_, _>")]
  pub(crate) entries: BTreeMap<ComponentBuf, DirectoryTreeEntry>,
}

impl DirectoryTree {
  pub(crate) fn create_directory(&mut self, path: &RelativePath) -> Result<(), ManifestError> {
    let mut current = self;
    for component in path.components() {
      current = current.create_directory_entry(component)?;
    }
    Ok(())
  }

  fn create_directory_entry(
    &mut self,
    component: &Component,
  ) -> Result<&mut DirectoryTree, ManifestError> {
    let entry = self
      .entries
      .entry(component.to_owned())
      .or_insert(DirectoryTreeEntry::Directory(DirectoryTree::new()));

    match entry {
      DirectoryTreeEntry::Directory(directory) => Ok(directory),
      DirectoryTreeEntry::File(_file) => Err(
        manifest_error::Internal {
          message: "entry `{component}` already contains file",
        }
        .build(),
      ),
    }
  }

  pub(crate) fn create_file(
    &mut self,
    path: &RelativePath,
    file: File,
  ) -> Result<(), ManifestError> {
    let mut components = path.components().peekable();

    let mut current = self;
    while let Some(component) = components.next() {
      if components.peek().is_none() {
        ensure! {
          current.entries.insert(component.to_owned(), DirectoryTreeEntry::File(file)).is_none(),
          manifest_error::Internal {
            message: "entry `{component}` already contains file",
          }
        }
        return Ok(());
      }

      current = current.create_directory_entry(component)?;
    }

    Ok(())
  }

  pub(crate) fn descendants(&self) -> Descendants {
    Descendants::new(self)
  }

  pub(crate) fn files(&self) -> BTreeMap<RelativePath, File> {
    let mut files = BTreeMap::new();

    for (path, entry) in self.descendants() {
      if let DirectoryTreeEntry::File(file) = entry {
        let old = files.insert(path, *file);
        assert!(old.is_none());
      }
    }

    files
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn subdirectory(&self, path: &RelativePath) -> Option<&DirectoryTree> {
    let mut current = self;
    for component in path.components() {
      let DirectoryTreeEntry::Directory(directory) = current.entries.get(component)? else {
        return None;
      };
      current = directory;
    }
    Some(current)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duplicate_entries_are_rejected() {
    assert_eq!(
      serde_json::from_str::<DirectoryTree>(r#"{"a":{},"a":{}}"#)
        .unwrap_err()
        .to_string(),
      "invalid entry: found duplicate key at line 1 column 15",
    );
  }
}
