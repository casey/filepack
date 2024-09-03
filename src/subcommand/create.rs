use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let mut files = HashMap::new();

  let mut dirs = Vec::new();

  for entry in WalkDir::new(root) {
    let entry = entry?;

    let path = entry.path();

    let path = Utf8Path::from_path(path).context(error::PathUnicode { path })?;

    while let Some(dir) = dirs.last() {
      if path.starts_with(dir) {
        dirs.pop();
      } else {
        break;
      }
    }

    if entry.file_type().is_dir() {
      if path != root {
        dirs.push(path.to_owned());
      }
      continue;
    }

    if entry.file_type().is_symlink() {
      return Err(error::Symlink { path }.build());
    }

    let file = File::open(path).context(error::Io { path })?;

    let mut hasher = Hasher::new();

    hasher.update_reader(file).context(error::Io { path })?;

    let relative = path.strip_prefix(root).unwrap();

    let relative = RelativePath::try_from(relative).context(error::Path { path: relative })?;

    relative.check_portability().context(error::PathLint {
      path: relative.clone(),
    })?;

    files.insert(relative, hasher.finalize().into());
  }

  if !dirs.is_empty() {
    return Err(Error::EmptyDirectory {
      paths: dirs
        .into_iter()
        .map(|dir| dir.strip_prefix(root).unwrap().to_owned())
        .collect(),
    });
  }

  let destination = root.join(Manifest::FILENAME);

  if destination
    .try_exists()
    .context(error::Io { path: &destination })?
  {
    return Err(
      error::ManifestAlreadyExists {
        path: Manifest::FILENAME,
      }
      .build(),
    );
  }

  let manifest = Manifest { files };

  let json = serde_json::to_string(&manifest).unwrap();

  fs::write(&destination, &json).context(error::Io { path: destination })?;

  Ok(())
}
