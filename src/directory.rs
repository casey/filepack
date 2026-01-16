use super::*;

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", transparent)]
pub struct Directory {
  #[serde_as(as = "MapPreventDuplicates<_, _>")]
  pub(crate) entries: BTreeMap<Component, Entry>,
}

impl Directory {
  pub(crate) fn create_directory(&mut self, path: &RelativePath) -> Result {
    let mut current = self;
    for component in path.components() {
      current = current.create_directory_entry(component)?;
    }
    Ok(())
  }

  fn create_directory_entry(&mut self, component: Component) -> Result<&mut Directory> {
    let entry = self
      .entries
      .entry(component)
      .or_insert(Entry::Directory(Directory::new()));

    match entry {
      Entry::Directory(directory) => Ok(directory),
      Entry::File(_file) => Err(
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
          current.entries.insert(component, Entry::File(file)).is_none(),
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

  pub(crate) fn fingerprint(&self) -> Fingerprint {
    let mut hasher = FingerprintHasher::new(FingerprintPrefix::Directory);

    for (component, entry) in &self.entries {
      hasher.field(0, entry.fingerprint(component).as_bytes());
    }

    hasher.finalize()
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duplicate_entries_are_rejected() {
    assert_eq!(
      serde_json::from_str::<Directory>(r#"{"a":{},"a":{}}"#)
        .unwrap_err()
        .to_string(),
      "invalid entry: found duplicate key at line 1 column 15",
    );
  }
}
