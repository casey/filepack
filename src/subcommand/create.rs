use super::*;

#[derive(Parser)]
pub(crate) struct Create {
  #[arg(help = "Create manifest for files in <ROOT> directory, defaulting to current directory")]
  root: Option<Utf8PathBuf>,
}

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    let root = if let Some(root) = self.root {
      root
    } else {
      let path = env::current_dir().context(error::CurrentDir)?;
      Utf8PathBuf::from_path_buf(path).map_err(|path| error::PathUnicode { path }.build())?
    };

    let mut paths = HashMap::new();

    let mut dirs = Vec::new();

    for entry in WalkDir::new(&root) {
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

      let relative = path.strip_prefix(&root).unwrap();

      let relative = RelativePath::try_from(relative).context(error::Path { path: relative })?;

      relative
        .check_portability()
        .context(error::PathLint { path: &relative })?;

      let metadata = path.metadata().context(error::Io { path })?;

      paths.insert(relative, metadata.len());
    }

    if !dirs.is_empty() {
      dirs.sort();
      return Err(Error::EmptyDirectory {
        paths: dirs
          .into_iter()
          .map(|dir| dir.strip_prefix(&root).unwrap().to_owned())
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

    let mut files = HashMap::new();

    let bar = progress_bar::new(paths.values().sum());

    for (path, _size) in paths {
      let entry = options.hash_file(&root.join(&path))?;
      files.insert(path, entry);
      bar.inc(entry.size);
    }

    let manifest = Manifest { files };

    let json = serde_json::to_string(&manifest).unwrap();

    fs::write(&destination, &json).context(error::Io { path: destination })?;

    Ok(())
  }
}
