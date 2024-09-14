use super::*;

#[derive(Parser)]
pub(crate) struct Create {
  #[arg(help = "Deny <LINT_GROUP>", long, value_name = "LINT_GROUP")]
  deny: Option<LintGroup>,
  #[arg(help = "Overwrite manifest if it already exists", long)]
  force: bool,
  #[arg(
    help = "Write manifest to <MANIFEST>, defaults to `<ROOT>/filepack.json`",
    long
  )]
  manifest: Option<Utf8PathBuf>,
  #[arg(help = "Create manifest for files in <ROOT> directory, defaults to current directory")]
  root: Option<Utf8PathBuf>,
}

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    let current_dir = env::current_dir().context(error::CurrentDir)?;

    let root = if let Some(root) = self.root {
      root
    } else {
      Utf8PathBuf::from_path_buf(current_dir.clone())
        .map_err(|path| error::PathUnicode { path }.build())?
    };

    let destination = if let Some(path) = self.manifest {
      path
    } else {
      root.join(Manifest::FILENAME)
    };

    let cleaned_destination = current_dir.join(&destination).lexiclean();

    let mut paths = HashMap::new();

    let mut case_conflicts = HashMap::<RelativePath, Vec<RelativePath>>::new();

    let mut lint_errors = 0u64;

    let mut dirs = Vec::new();

    for entry in WalkDir::new(&root) {
      let entry = entry?;

      let path = entry.path();

      let path = Utf8Path::from_path(path).context(error::PathUnicode { path })?;

      if current_dir.join(path).lexiclean() == cleaned_destination {
        continue;
      }

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

      ensure! {
        !entry.file_type().is_symlink(),
        error::Symlink { path },
      }

      let relative = path.strip_prefix(&root).unwrap();

      let relative = RelativePath::try_from(relative).context(error::Path { path: relative })?;

      match self.deny {
        None => {}
        Some(LintGroup::All) => {
          if let Some(lint) = relative.lint() {
            eprintln!("error: path failed lint: `{relative}`");
            eprintln!("       └─ {lint}");
            lint_errors += 1;
          }

          case_conflicts
            .entry(relative.to_lowercase())
            .or_default()
            .push(relative.clone());
        }
      }

      let metadata = path.metadata().context(error::Io { path })?;

      paths.insert(relative, metadata.len());
    }

    for mut originals in case_conflicts.into_values() {
      if originals.len() > 1 {
        originals.sort();
        eprintln!("error: paths would conflict on case-insensitive filesystem:");
        for (i, original) in originals.iter().enumerate() {
          eprintln!(
            "       {}─ `{original}`",
            if i < originals.len() - 1 {
              '├'
            } else {
              '└'
            }
          );
        }
        lint_errors += 1;
      }
    }

    if lint_errors > 0 {
      return Err(error::Lint { count: lint_errors }.build());
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

    ensure! {
      self.force || !destination.try_exists().context(error::Io { path: &destination })?,
      error::ManifestAlreadyExists {
        path: destination,
      },
    }

    let mut files = HashMap::new();

    let bar = progress_bar::new(paths.values().sum());

    for (path, _size) in paths {
      let entry = options
        .hash_file(&root.join(&path))
        .context(error::Io { path: &path })?;
      files.insert(path, entry);
      bar.inc(entry.size);
    }

    let manifest = Manifest { files };

    let json = serde_json::to_string(&manifest).unwrap();

    fs::write(&destination, &json).context(error::Io { path: destination })?;

    Ok(())
  }
}
