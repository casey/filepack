use super::*;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", transparent)]
pub struct Directory {
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

  pub(crate) fn fingerprint(&self) -> Hash {
    let mut hasher = ContextHasher::new(Context::Directory);

    hasher.array(0, self.entries.len().into_u64());

    for (component, entry) in &self.entries {
      hasher.element(entry.fingerprint(component));
    }

    hasher.finalize()
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }
}
