use super::*;

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, transparent)]
pub struct DirectoryTree {
  #[serde_as(as = "MapPreventDuplicates<_, _>")]
  pub(crate) entries: BTreeMap<Component, DirectoryTreeEntry>,
}

impl DirectoryTree {
  pub(crate) fn create_directory(&mut self, path: &RelativePath) -> Result {
    let mut current = self;
    for component in path.components() {
      current = current.create_directory_entry(component)?;
    }
    Ok(())
  }

  fn create_directory_entry(&mut self, component: Component) -> Result<&mut DirectoryTree> {
    let entry = self
      .entries
      .entry(component)
      .or_insert(DirectoryTreeEntry::Directory(DirectoryTree::new()));

    match entry {
      DirectoryTreeEntry::Directory(directory) => Ok(directory),
      DirectoryTreeEntry::File(_file) => Err(
        error::Internal {
          message: "entry `{component}` already contains file",
        }
        .build(),
      ),
    }
  }

  pub(crate) fn create_file(&mut self, path: &RelativePath, file: File) -> Result {
    let mut components = path.components().peekable();

    let mut current = self;
    while let Some(component) = components.next() {
      if components.peek().is_none() {
        ensure! {
          current.entries.insert(component, DirectoryTreeEntry::File(file)).is_none(),
          error::Internal {
            message: "entry `{component}` already contains file",
          }
        }
        return Ok(());
      }

      current = current.create_directory_entry(component)?;
    }

    Ok(())
  }

  pub fn new() -> Self {
    Self::default()
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
