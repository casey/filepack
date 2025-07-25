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
  #[arg(help = "Include metadata from YAML document <METADATA>`", long)]
  metadata: Option<Utf8PathBuf>,
  #[arg(help = "Create manifest for files in <ROOT> directory, defaults to current directory")]
  root: Option<Utf8PathBuf>,
  #[arg(help = "Sign manifest with master key", long)]
  sign: bool,
}

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    let current_dir = current_dir()?;

    let root = self.root.unwrap_or_else(|| current_dir.clone());

    let manifest_path = if let Some(path) = self.manifest {
      path
    } else {
      root.join(Manifest::FILENAME)
    };

    if let Some(path) = &self.metadata {
      let yaml = filesystem::read_to_string(path)?;
      let template = serde_yaml::from_str::<Template>(&yaml)
        .context(error::DeserializeMetadataTemplate { path })?;
      let path = root.join(Metadata::FILENAME);
      ensure! {
        self.force || !filesystem::exists(&path)?,
        error::MetadataAlreadyExists { path: &path },
      }
      Metadata::from(template).store(&path)?;
    }

    let cleaned_manifest = current_dir.join(&manifest_path).lexiclean();

    let cleaned_metadata = self.metadata.map(|path| current_dir.join(path).lexiclean());

    let mut paths = HashMap::new();

    let mut case_conflicts = HashMap::<RelativePath, Vec<RelativePath>>::new();

    let mut lint_errors = 0u64;

    let mut dirs = Vec::new();

    for entry in WalkDir::new(&root) {
      let entry = entry?;

      let path = entry.path();

      let path = decode_path(path)?;

      let cleaned_path = current_dir.join(path).lexiclean();

      if cleaned_path == cleaned_manifest {
        continue;
      }

      if cleaned_metadata
        .as_ref()
        .is_some_and(|path| cleaned_path == *path)
      {
        return Err(error::MetadataTemplateIncluded { path }.build());
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

      let metadata = filesystem::metadata(path)?;

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
      self.force || !manifest_path.try_exists().context(error::FilesystemIo { path: &manifest_path })?,
      error::ManifestAlreadyExists {
        path: manifest_path,
      },
    }

    let mut files = BTreeMap::new();

    let bar = progress_bar::new(&options, paths.values().sum());

    for (path, _size) in paths {
      let entry = options
        .hash_file(&root.join(&path))
        .context(error::FilesystemIo { path: &path })?;
      files.insert(path, entry);
      bar.inc(entry.size);
    }

    let mut manifest = Manifest {
      files,
      signatures: BTreeMap::new(),
    };

    if self.sign {
      let private_key_path = options.key_dir()?.join(MASTER_PRIVATE_KEY);

      let (public_key, signature) =
        PrivateKey::load_and_sign(&private_key_path, manifest.fingerprint().as_bytes())?;

      manifest.signatures.insert(public_key, signature);
    }

    manifest.store(&manifest_path)?;

    Ok(())
  }
}
