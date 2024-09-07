use super::*;

#[derive(Parser)]
pub(crate) struct Create {
  #[arg(help = "Create manifest for files in <ROOT> directory")]
  root: Utf8PathBuf,
}

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    let mut paths = HashSet::new();

    let mut dirs = Vec::new();

    for entry in WalkDir::new(&self.root) {
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
        if path != self.root {
          dirs.push(path.to_owned());
        }
        continue;
      }

      if entry.file_type().is_symlink() {
        return Err(error::Symlink { path }.build());
      }

      let relative = path.strip_prefix(&self.root).unwrap();

      let relative = RelativePath::try_from(relative).context(error::Path { path: relative })?;

      relative.check_portability().context(error::PathLint {
        path: relative.clone(),
      })?;

      paths.insert(relative);
    }

    if !dirs.is_empty() {
      dirs.sort();
      return Err(Error::EmptyDirectory {
        paths: dirs
          .into_iter()
          .map(|dir| dir.strip_prefix(&self.root).unwrap().to_owned())
          .collect(),
      });
    }

    let destination = self.root.join(Manifest::FILENAME);

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

    let mut files = HashMap::new();

    for path in paths {
      let hash = options.hash_file(&self.root.join(&path))?;
      files.insert(path, hash);
    }

    let manifest = Manifest { files };

    let json = serde_json::to_string(&manifest).unwrap();

    fs::write(&destination, &json).context(error::Io { path: destination })?;

    Ok(())
  }
}
