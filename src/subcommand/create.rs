use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let mut files = HashMap::new();

  for entry in WalkDir::new(root) {
    let entry = entry?;

    if entry.file_type().is_dir() {
      continue;
    }

    let path = entry.path();

    let path = Utf8Path::from_path(path).context(error::PathUnicode { path })?;

    if entry.file_type().is_symlink() {
      return Err(error::Symlink { path }.build());
    }

    let file = File::open(path).context(error::Io { path })?;

    let mut hasher = Hasher::new();

    hasher.update_reader(file).context(error::Io { path })?;

    let relative = path.strip_prefix(root).unwrap();

    let mut components = Vec::new();

    for component in relative.components() {
      let Utf8Component::Normal(component) = component else {
        return Err(Error::Internal {
          message: format!("unexpected path component `{component}`"),
        });
      };

      if component.contains('\\') {
        return Err(Error::PathBackslash {
          path: relative.into(),
        });
      }

      components.push(component);
    }

    files.insert(components.join("/").into(), hasher.finalize().into());
  }

  let destination = root.join(Filepack::FILENAME);

  if destination
    .try_exists()
    .context(error::Io { path: &destination })?
  {
    return Err(Error::ManifestAlreadyExists { path: destination });
  }

  let filepack = Filepack { files };

  let json = serde_json::to_string(&filepack).unwrap();

  fs::write(&destination, &json).context(error::Io { path: destination })?;

  Ok(())
}
