use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let mut files = HashMap::new();

  for entry in WalkDir::new(root) {
    let entry = entry?;

    if entry.file_type().is_dir() {
      continue;
    }

    let path = entry.path();

    let path = Utf8Path::from_path(path).context(error::Path { path })?;

    if entry.file_type().is_symlink() {
      return Err(error::Symlink { path }.build());
    }

    let file = File::open(path).context(error::Io { path })?;

    let mut hasher = Hasher::new();

    hasher.update_reader(file).context(error::Io { path })?;

    let relative = path.strip_prefix(root).unwrap();

    files.insert(relative.into(), hasher.finalize().into());
  }

  let destination = root.join(Filepack::FILENAME);

  if destination
    .try_exists()
    .context(error::Io { path: &destination })?
  {
    return Err(Error::FilepackExists { path: destination });
  }

  let filepack = Filepack { files };

  let json = serde_json::to_string(&filepack).unwrap();

  fs::write(&destination, &json).context(error::Io { path: destination })?;

  Ok(())
}
