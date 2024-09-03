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

    let relative = relative
      .as_str()
      .parse::<RelativePath>()
      .context(error::Path { path: relative })?;

    relative.check_portability().context(error::PathLint {
      path: relative.clone(),
    })?;

    files.insert(relative, hasher.finalize().into());
  }

  let destination = root.join(Manifest::FILENAME);

  if destination
    .try_exists()
    .context(error::Io { path: &destination })?
  {
    return Err(error::ManifestAlreadyExists { path: destination }.build());
  }

  let manifest = Manifest { files };

  let json = serde_json::to_string(&manifest).unwrap();

  fs::write(&destination, &json).context(error::Io { path: destination })?;

  Ok(())
}
