use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[arg(
    help = "Read manifest from <MANIFEST>, defaults to `<ROOT>/filepack.json`",
    long
  )]
  manifest: Option<Utf8PathBuf>,
  #[arg(help = "Print manifest if verification is successful", long)]
  print: bool,
  #[arg(help = "Verify files in <ROOT> directory against manifest, defaults to current directory")]
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

    let source = if let Some(ref manifest) = self.manifest {
      manifest.clone()
    } else {
      root.join(Manifest::FILENAME)
    };

    let json = match fs::read_to_string(&source) {
      Err(err) if err.kind() == io::ErrorKind::NotFound => {
        return Err(
          error::ManifestNotFound {
            path: self
              .manifest
              .as_deref()
              .unwrap_or(Utf8Path::new(Manifest::FILENAME)),
          }
          .build(),
        );
      }
      result => result.context(error::Io { path: &source })?,
    };

    let manifest = serde_json::from_str::<Manifest>(&json).context(error::Deserialize {
      path: Manifest::FILENAME,
    })?;

    let bar = progress_bar::new(manifest.files.values().map(|entry| entry.size).sum());

    let mut mismatches = BTreeMap::new();

    for (path, &expected) in &manifest.files {
      let actual = match options.hash_file(&root.join(path)) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
          return Err(error::MissingFile { path }.build())
        }
        result => result.context(error::Io { path })?,
      };

      if actual != expected {
        mismatches.insert(path, (actual, expected));
      }

      bar.inc(expected.size);
    }

    if !mismatches.is_empty() {
      for (path, (actual, expected)) in &mismatches {
        eprintln!(
          "\
mismatched file: `{path}`
       manifest: {} ({} bytes)
           file: {} ({} bytes)",
          expected.hash, expected.size, actual.hash, actual.size,
        );
      }

      return Err(
        error::EntryMismatch {
          count: mismatches.len(),
        }
        .build(),
      );
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

      if path == source {
        continue;
      }

      let path = path.strip_prefix(&root).unwrap();

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      ensure! {
        manifest.files.contains_key(&path),
        error::ExtraneousFile { path },
      }
    }

    if !dirs.is_empty() {
      dirs.sort();
      return Err(Error::EmptyDirectory {
        paths: dirs
          .into_iter()
          .map(|dir| dir.strip_prefix(&root).unwrap().to_owned().into())
          .collect(),
      });
    }

    if self.print {
      print!("{json}");
    }

    Ok(())
  }
}
