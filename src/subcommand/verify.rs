use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let source = root.join(Filepack::FILENAME);

  let json = fs::read_to_string(&source).context(error::Io { path: &source })?;

  let filepack =
    serde_json::from_str::<Filepack>(&json).context(error::Deserialize { path: &source })?;

  for (path, &expected) in &filepack.files {
    for component in path.components() {
      if !matches!(component, Utf8Component::Normal(_)) {
        return Err(Error::PathComponent {
          component: component.to_string(),
          path: path.into(),
        });
      }
    }

    if path.as_str().ends_with("/") {
      return Err(Error::PathTrailingSlash { path: path.into() });
    }

    if path.as_str().contains("//") {
      return Err(Error::PathDoubleSlash { path: path.into() });
    }

    if path.as_str().contains('\\') {
      return Err(Error::PathBackslash { path: path.into() });
    }

    let file = File::open(root.join(path)).context(error::Io { path })?;

    let mut hasher = Hasher::new();

    hasher.update_reader(file).context(error::Io { path })?;

    let actual = Hash::from(hasher.finalize());

    if actual != expected {
      return Err(Error::HashMismatch {
        path: path.into(),
        actual,
        expected,
      });
    }
  }

  for entry in WalkDir::new(root) {
    let entry = entry?;

    if entry.file_type().is_dir() {
      continue;
    }

    let path = entry.path();

    if path == root {
      continue;
    }

    let path = Utf8Path::from_path(path).context(error::PathUnicode { path })?;

    let relative = path.strip_prefix(root).unwrap();

    if relative == Filepack::FILENAME {
      continue;
    }

    if !filepack.files.contains_key(relative) {
      return Err(Error::ExtraneousFile { path: path.into() });
    }
  }

  Ok(())
}
