use super::*;

#[derive(Parser)]
pub(crate) struct Create {
  #[arg(help = "Allow <LINT>", long, value_name = "LINT")]
  allow: Vec<LintSelector>,
  #[arg(help = "Deny <LINT>", long, value_name = "LINT")]
  deny: Vec<LintSelector>,
  #[arg(
    help = "Overwrite manifest and generated assets if they already exist",
    long
  )]
  force: bool,
  #[arg(help = "Generate derived assets", long)]
  generate: bool,
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

    ensure! {
      filesystem::metadata(&root)?.is_dir(),
      error::PackageRootDirectory { path: root },
    }

    let manifest_path = if let Some(path) = self.manifest {
      path
    } else {
      root.join(Manifest::FILENAME)
    };

    let path = root.join(Metadata::YAML_FILENAME);

    let metadata = if let Some(yaml) = filesystem::read_to_string_opt(&path)? {
      let metadata = yaml::Metadata::deserialize(&path, &yaml)?;

      let path = root.join(Metadata::CBOR_FILENAME);

      ensure! {
        self.force || !filesystem::exists(&path)?,
        error::MetadataAlreadyExists {
          path,
        },
      }

      Some(metadata)
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

    let mut linter = Linter::new();

    linter.deny(&self.deny);

    linter.allow(&self.allow);

    linter.lint_metadata(metadata.as_ref(), self.generate);

    linter.check()?;

    let metadata = if let Some(metadata) = metadata {
      let mut metadata = metadata.load(&root, options.quiet)?;

      if self.generate {
        metadata.generate(&root, self.force, options.quiet)?;
      }

      metadata.validate(&root)?;

      let cbor = metadata.encode_to_vec();

      Some((metadata, cbor))
    } else {
      None
    };

    let cleaned_manifest = current_dir.join(&manifest_path).lexiclean();

    let mut paths = HashMap::new();

    let mut empty = Vec::new();

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

      linter.lint_path(&relative);

      empty.pop_if(|dir| relative.starts_with(dir));

      if entry.file_type().is_dir() {
        empty.push(relative);
        continue;
      }

      paths.insert(relative, metadata.len());
    }

    linter.lint_case_conflicts();

    linter.check()?;

    linter.lint_content(
      &root,
      &options,
      metadata.as_ref().map(|(metadata, _cbor)| metadata),
    )?;

    linter.done()?;

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

    if let Some((_metadata, cbor)) = &metadata {
      filesystem::write(&root.join(Metadata::CBOR_FILENAME), cbor)?;

      paths.insert(
        Metadata::CBOR_FILENAME.parse().unwrap(),
        cbor.len().into_u64(),
      );
    }

    let mut total_file_size = 0u64;
    for size in paths.values() {
      total_file_size = total_file_size
        .checked_add(*size)
        .context(error::TotalFileSizeOverflow)?;
    }

    let bar = ProgressBar::bytes(&options, total_file_size);

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
