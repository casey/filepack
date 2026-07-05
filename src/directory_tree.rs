use super::*;

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, transparent)]
pub struct DirectoryTree {
  #[serde_as(as = "MapPreventDuplicates<_, _>")]
  pub(crate) entries: BTreeMap<ComponentBuf, DirectoryTreeEntry>,
}

impl DirectoryTree {
  pub(crate) fn create_directory(&mut self, path: &RelativePath) -> Result {
    let mut current = self;
    for component in path.components() {
      current = current.create_directory_entry(component)?;
    }
    Ok(())
  }

  fn create_directory_entry(&mut self, component: &Component) -> Result<&mut DirectoryTree> {
    let entry = self
      .entries
      .entry(component.to_owned())
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
          current.entries.insert(component.to_owned(), DirectoryTreeEntry::File(file)).is_none(),
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

  pub(crate) fn totals(&self) -> Result<Totals, TotalsError> {
    let mut file_size = 0u64;
    let mut files = 0;
    let mut stack = vec![self];
    while let Some(directory) = stack.pop() {
      for entry in directory.entries.values() {
        match entry {
          DirectoryTreeEntry::File(file) => {
            files += 1;
            file_size = file_size
              .checked_add(file.size)
              .context(totals_error::Overflow)?;
          }
          DirectoryTreeEntry::Directory(directory) => stack.push(directory),
        }
      }
    }

    Ok(Totals { files, file_size })
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

  #[test]
  fn total_file_size() {
    #[track_caller]
    fn case(files: &[(&str, u64)], expected: Option<u64>) {
      let mut tree = DirectoryTree::new();

      for (path, size) in files {
        tree
          .create_file(
            &path.parse().unwrap(),
            File {
              hash: Hash::bytes(b"foo"),
              size: *size,
            },
          )
          .unwrap();
      }

      assert_eq!(tree.total_file_size(), expected);
    }

    case(&[], Some(0));
    case(&[("bar", 1), ("baz", 2)], Some(3));
    case(&[("bar/baz", 1), ("foo", 2)], Some(3));
    case(&[("bar", u64::MAX), ("baz", 0)], Some(u64::MAX));
    case(&[("bar", u64::MAX), ("baz", 1)], None);
  }
}
