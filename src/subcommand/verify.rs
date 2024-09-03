use super::*;

pub(crate) fn run(root: &Utf8Path) -> Result {
  let source = root.join(Manifest::FILENAME);

  let json = fs::read_to_string(&source).context(error::Io {
    path: Manifest::FILENAME,
  })?;

  let manifest = serde_json::from_str::<Manifest>(&json).context(error::Deserialize {
    path: Manifest::FILENAME,
  })?;

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

    let path = path.strip_prefix(root).unwrap();

    let path = RelativePath::try_from(path).context(error::Path { path })?;

    if path == Manifest::FILENAME {
      continue;
    }

    if !manifest.files.contains_key(&path) {
      return Err(error::ExtraneousFile { path }.build());
    }
  }

  if !dirs.is_empty() {
    return Err(Error::EmptyDirectory {
      paths: dirs
        .into_iter()
        .map(|dir| dir.strip_prefix(root).unwrap().to_owned())
        .collect(),
    });
  }

  Ok(())
}
