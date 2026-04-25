use super::*;

pub(crate) struct Entries<'a> {
  stack: Vec<(Vec<&'a Component>, &'a DirectoryTreeEntry)>,
}

impl<'a> From<&'a Manifest> for Entries<'a> {
  fn from(manifest: &'a Manifest) -> Self {
    let mut stack = Vec::new();
    for (component, entry) in &manifest.package.entries {
      stack.push((vec![component.borrow()], entry));
    }
    Self { stack }
  }
}

impl<'a> Iterator for Entries<'a> {
  type Item = (RelativePath, &'a DirectoryTreeEntry);

  fn next(&mut self) -> Option<Self::Item> {
    let (components, entry) = self.stack.pop()?;

    if let DirectoryTreeEntry::Directory(directory) = entry {
      for (component, child) in &directory.entries {
        self.stack.push((
          components
            .iter()
            .copied()
            .chain(iter::once(&**component))
            .collect(),
          child,
        ));
      }
    }

    Some((components.as_slice().try_into().unwrap(), entry))
  }
}
