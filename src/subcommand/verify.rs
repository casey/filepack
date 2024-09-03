use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let source = root.join(Manifest::FILENAME);

  let json = fs::read_to_string(&source).context(error::Io { path: &source })?;

  let manifest =
    serde_json::from_str::<Manifest>(&json).context(error::Deserialize { path: &source })?;

  for (path, &expected) in &manifest.files {
    let file = File::open(root.join(path)).context(error::Io { path })?;

    let mut hasher = Hasher::new();

    hasher.update_reader(file).context(error::Io { path })?;

    let actual = Hash::from(hasher.finalize());

    if actual != expected {
      return Err(
        error::HashMismatch {
          actual,
          expected,
          path,
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

    // todo: handle error
    let relative = path
      .strip_prefix(root)
      .unwrap()
      .as_str()
      .parse::<RelativePath>()
      .unwrap();

    if relative == Manifest::FILENAME {
      continue;
    }

    if !manifest.files.contains_key(&relative) {
      return Err(error::ExtraneousFile { path }.build());
    }
  }

  Ok(())
}
