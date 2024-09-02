use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let source = root.join(Manifest::FILENAME);

  let json = fs::read_to_string(&source).context(error::Io { path: &source })?;

  let manifest =
    serde_json::from_str::<Manifest>(&json).context(error::Deserialize { path: &source })?;

  for (path, &expected) in &manifest.files {
    for component in path.components() {
      if !matches!(component, Utf8Component::Normal(_)) {
        return Err(
          error::PathComponent {
            component: component.to_string(),
            path,
          }
          .build(),
        );
      }
    }

    if path.as_str().ends_with('/') {
      return Err(error::PathTrailingSlash { path }.build());
    }

    if path.as_str().contains("//") {
      return Err(error::PathDoubleSlash { path }.build());
    }

    if path.as_str().contains('\\') {
      return Err(error::PathBackslash { path }.build());
    }

    let file = File::open(root.join(path)).context(error::Io { path })?;

    let mut hasher = Hasher::new();

    hasher.update_reader(file).context(error::Io { path })?;

    let actual = Hash::from(hasher.finalize());

    if actual != expected {
      return Err(
        error::HashMismatch {
          path,
          actual,
          expected,
        }
        .build(),
      );
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

    if relative == Manifest::FILENAME {
      continue;
    }

    if !manifest.files.contains_key(relative) {
      return Err(error::ExtraneousFile { path }.build());
    }
  }

  Ok(())
}
