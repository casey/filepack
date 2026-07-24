use super::*;

#[derive(Parser)]
pub(crate) struct Create {
  #[arg(help = "Deny <LINT_GROUP>", long, value_name = "LINT_GROUP")]
  deny: Option<LintGroup>,
  #[arg(help = "Overwrite manifest if it already exists", long)]
  force: bool,
  #[arg(help = "Ignore <PATH>", long, value_name = "PATH")]
  ignore: Vec<RelativePath>,
  #[arg(default_value_t = KeyName::DEFAULT, help = "Sign with <KEY>", long, requires = "sign")]
  key: KeyName,
  #[arg(
    help = "Write manifest to <MANIFEST>, defaults to `<ROOT>/manifest.filepack`",
    long
  )]
  manifest: Option<Utf8PathBuf>,
  #[arg(help = "Create manifest for files in <ROOT> directory, defaults to current directory")]
  root: Option<Utf8PathBuf>,
  #[arg(help = "Sign manifest", long)]
  sign: bool,
  #[arg(help = TIMESTAMP_HELP, long)]
  timestamp: bool,
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

    let path = root.join(Metadata::YAML_FILENAME);

    let metadata = if let Some(yaml) = filesystem::read_to_string_opt(&path)? {
      let mut metadata = Metadata::deserialize(&path, &yaml)?;

      metadata.populate(&root)?;

      metadata.validate(&root)?;

      let cbor = metadata.encode_to_vec();

      let path = root.join(Metadata::CBOR_FILENAME);

      ensure! {
        self.force || !filesystem::exists(&path)?,
        error::MetadataAlreadyExists {
          path,
        },
      }

      filesystem::write(&path, &cbor)?;

      Some((metadata, cbor))
    } else {
      let path = root.join(Metadata::CBOR_FILENAME);

      ensure! {
        !filesystem::exists(&path)?,
        error::StaleMetadata {
          path,
        },
      }

      None
    };

    let cleaned_manifest = current_dir.join(&manifest_path).lexiclean();

    let mut paths = HashMap::new();

    let mut case_conflicts = HashMap::<RelativePath, Vec<RelativePath>>::new();

    let mut lint_errors = 0u64;

    let mut empty = Vec::new();

    let lints = self.deny.map(LintGroup::lints).unwrap_or_default();

    for entry in WalkDir::new(&root).sort_by_file_name() {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let path = decode_path(entry.path())?;

      let cleaned_path = current_dir.join(path).lexiclean();

      if cleaned_path == cleaned_manifest {
        continue;
      }

      let relative = path.strip_prefix(&root).unwrap();

      let relative = RelativePath::try_from(relative).context(error::Path { path: relative })?;

      if self
        .ignore
        .iter()
        .any(|ignore| relative.starts_with(ignore))
      {
        continue;
      }

      ensure! {
        !entry.file_type().is_symlink(),
        error::Symlink { path },
      }

      let metadata = filesystem::metadata(path)?;

      if let Some(lint) = relative.lint(&lints) {
        eprintln!("error: path failed lint: `{relative}`");
        eprintln!("       └─ {lint}");
        lint_errors += 1;
      }

      if lints.contains(&Lint::CaseConflict) {
        case_conflicts
          .entry(relative.to_lowercase())
          .or_default()
          .push(relative.clone());
      }

      empty.pop_if(|dir| relative.starts_with(dir));

      if entry.file_type().is_dir() {
        empty.push(relative);
        continue;
      }

      paths.insert(relative, metadata.len());
    }

    for mut originals in case_conflicts.into_values() {
      if originals.len() > 1 {
        originals.sort();
        eprintln!("error: {}", LintError::CaseConflict);
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

    if let Some((metadata, _cbor)) = &metadata {
      let files = paths.keys().cloned().collect::<HashSet<RelativePath>>();

      metadata.check_files(&files)?;

      if metadata.media.is_some() {
        metadata.check_extras(&files, &empty)?;
      }
    }

    ensure! {
      self.force || !manifest_path.try_exists().context(error::FilesystemIo { path: &manifest_path })?,
      error::ManifestAlreadyExists {
        path: manifest_path,
      },
    }

    let mut total_file_size = 0u64;
    for size in paths.values() {
      total_file_size = total_file_size
        .checked_add(*size)
        .context(error::TotalFileSizeOverflow)?;
    }

    let bar = progress_bar::new(&options, total_file_size);

    let mut package = DirectoryTree::new();

    for path in empty {
      package.create_directory(&path)?;
    }

    for (path, _size) in paths {
      let file = options
        .hash_file(&root.join(&path))
        .context(error::FilesystemIo { path: &path })?;
      package.create_file(&path, file)?;
      bar.inc(file.size);
    }

    let embedded = if let Some((_metadata, cbor)) = metadata {
      BTreeMap::from([(Hash::bytes(&cbor), cbor)])
    } else {
      BTreeMap::new()
    };

    let mut manifest = Manifest {
      embedded,
      package,
      signatures: BTreeSet::new(),
    };

    if self.sign {
      let keychain = Keychain::load(&options)?;
      manifest.sign(
        SignOptions {
          timestamp: self.timestamp,
        },
        &keychain,
        &self.key,
      )?;
    }

    manifest.save(&manifest_path)?;

    Ok(())
  }
}
