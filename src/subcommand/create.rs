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

    let manifest_path = if let Some(path) = self.manifest {
      path
    } else {
      root.join(Manifest::FILENAME)
    };

    let denied = self
      .deny
      .into_iter()
      .flat_map(LintSelector::lints)
      .collect::<BTreeSet<Lint>>();

    let allowed = self
      .allow
      .into_iter()
      .flat_map(LintSelector::lints)
      .collect::<BTreeSet<Lint>>();

    let mut linter = Linter::new(&denied - &allowed);

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

    linter.lint_metadata(metadata.as_ref(), self.generate)?;

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

    let mut case_conflicts = HashMap::<RelativePath, Vec<RelativePath>>::new();

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

      if let Some(lint) = relative.lint(linter.active()) {
        linter.error_path(lint, &relative);
      }

      if linter.is_active(Lint::CaseConflict) {
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
        linter.error_paths(LintError::CaseConflict, &originals);
      }
    }

    if (linter.is_active(Lint::AudioEmbeddedArtworkMissing)
      || linter.is_active(Lint::AudioEmbeddedArtworkAspectRatio))
      && let Some((metadata, _cbor)) = &metadata
      && let Some(Media::Audio { items }) = &metadata.media
    {
      let failures = {
        let bar = ProgressBar::count(options.quiet, items.len().into_u64(), "files");

        let mut failures = Vec::new();

        for audio in items {
          let covers = audio.content.cover_art(&root)?;

          if linter.is_active(Lint::AudioEmbeddedArtworkMissing) && covers.is_empty() {
            failures.push((audio, LintError::AudioEmbeddedArtworkMissing));
          }

          if linter.is_active(Lint::AudioEmbeddedArtworkAspectRatio) {
            for cover in covers {
              let dimensions = cover.dimensions().context(error::Audio {
                path: root.join(audio.path()),
              })?;

              if dimensions.width != dimensions.height {
                failures.push((
                  audio,
                  LintError::AudioEmbeddedArtworkAspectRatio { dimensions },
                ));
              }
            }
          }

          bar.inc(1);
        }

        failures
      };

      for (audio, lint) in failures {
        linter.error_path(lint, audio.path());
      }
    }

    if linter.is_active(Lint::VideoPlaceholderDimensions)
      && let Some((metadata, _cbor)) = &metadata
      && let Some(Media::Video { items }) = &metadata.media
    {
      for item in items {
        if let Some(placeholder) = &item.content.placeholder
          && let Some(video) = item.content.oriented_dimensions()
          && let placeholder = placeholder.oriented_dimensions()
          && placeholder != video
        {
          linter.error_path(
            LintError::VideoPlaceholderDimensions { placeholder, video },
            item.path(),
          );
        }
      }
    }

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
