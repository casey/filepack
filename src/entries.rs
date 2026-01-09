use super::*;

pub(crate) struct Entries<'a> {
  stack: Vec<(Vec<&'a Component>, &'a Entry)>,
}

impl<'a> From<&'a Manifest> for Entries<'a> {
  fn from(manifest: &'a Manifest) -> Self {
    let mut stack = Vec::new();
    for (component, entry) in &manifest.files.entries {
      stack.push((vec![component], entry));
    }
    Self { stack }
  }
}

impl<'a> Iterator for Entries<'a> {
  type Item = (RelativePath, &'a Entry);

  fn next(&mut self) -> Option<Self::Item> {
    let (components, entry) = self.stack.pop()?;

    if let Entry::Directory(directory) = entry {
      for (component, child) in &directory.entries {
        self.stack.push((
          components
            .iter()
            .copied()
            .chain(iter::once(component))
            .collect(),
          child,
        ));
      }
    }

    Some((components.as_slice().try_into().unwrap(), entry))
  }
}
