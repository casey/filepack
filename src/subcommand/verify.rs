use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[arg(help = "Print manifest if verification is successful", long)]
  print: bool,
  #[arg(
    help = "Verify files in <ROOT> directory against manifest, defaulting to current directory"
  )]
  root: Option<Utf8PathBuf>,
}

impl Verify {
  pub(crate) fn run(self, options: Options) -> Result {
    let root = if let Some(root) = self.root {
      root
    } else {
      let path = env::current_dir().context(error::CurrentDir)?;
      Utf8PathBuf::from_path_buf(path).map_err(|path| error::PathUnicode { path }.build())?
    };

    let source = root.join(Manifest::FILENAME);

    let json = match fs::read_to_string(&source) {
      Err(err) if err.kind() == io::ErrorKind::NotFound => {
        return Err(
          error::ManifestNotFound {
            path: Manifest::FILENAME,
          }
          .build(),
        );
      }
      result => result.context(error::Io {
        path: Manifest::FILENAME,
      })?,
    };

    let manifest = serde_json::from_str::<Manifest>(&json).context(error::Deserialize {
      path: Manifest::FILENAME,
    })?;

    let bar = progress_bar::new(manifest.files.values().map(|entry| entry.size).sum());

    for (path, entry) in &manifest.files {
      let size = match root.join(path).metadata() {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
          return Err(error::MissingFile { path }.build());
        }
        result => result.context(error::Io { path })?.len(),
      };

      if size != entry.size {
        return Err(
          error::SizeMismatch {
            actual: size,
            expected: entry.size,
            path,
          }
          .build(),
        );
      }

      let hash = options.hash_file(&root.join(path))?;

      if hash != entry.hash {
        return Err(
          error::HashMismatch {
            actual: hash,
            expected: entry.hash,
            path,
          }
          .build(),
        );
      }

      bar.inc(size);
    }

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

      let path = path.strip_prefix(&root).unwrap();

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      if path == Manifest::FILENAME {
        continue;
      }

      if !manifest.files.contains_key(&path) {
        return Err(error::ExtraneousFile { path }.build());
      }
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

    if self.print {
      print!("{json}");
    }

    Ok(())
  }
}
