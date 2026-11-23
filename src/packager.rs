use super::*;

pub(crate) use self::operation::Operation;

mod operation;

#[derive(Parser)]
pub(crate) struct Packager {
  #[arg(
    add = ArgValueCandidates::new(LintSelector::candidates),
    help = "Allow <LINT>",
    long,
    value_name = "LINT",
  )]
  allow: Vec<LintSelector>,
  #[arg(
    add = ArgValueCandidates::new(LintSelector::candidates),
    help = "Deny <LINT>",
    long,
    value_name = "LINT",
  )]
  deny: Vec<LintSelector>,
  #[arg(
    add = ArgValueCandidates::new(Lint::candidates),
    help = "Expect <LINT>",
    long,
    value_name = "LINT",
  )]
  expect: Vec<Lint>,
  #[arg(help = "Ignore <PATH>", long, value_name = "PATH")]
  ignore: Vec<RelativePath>,
  #[arg(default_value_t = KeyName::DEFAULT, help = "Sign with <KEY>", long, requires = "sign")]
  key: KeyName,
  #[arg(
    help = "Use manifest at <MANIFEST>, defaults to `<ROOT>/manifest.filepack`",
    long,
    value_hint = ValueHint::FilePath,
  )]
  manifest: Option<Utf8PathBuf>,
  #[arg(help = "Don't generate derived assets", long)]
  no_generate: bool,
  #[arg(
    help = "Package files in <ROOT> directory, defaults to current directory",
    value_hint = ValueHint::DirPath,
  )]
  root: Option<Utf8PathBuf>,
  #[arg(help = "Sign manifest", long)]
  sign: bool,
}

impl Packager {
  pub(crate) fn package(self, options: Options, operation: Operation) -> Result {
    let root = self.root.unwrap_or_else(|| ".".into());

    ensure! {
      filesystem::metadata(&root)?.is_dir(),
      error::PackageRootDirectory { path: root },
    }

    check_case(
      &root,
      &[
        Manifest::FILENAME,
        Metadata::DECO_FILENAME,
        Metadata::YAML_FILENAME,
        State::DIR,
      ],
    )?;

    let manifest_path = if let Some(path) = self.manifest {
      path
    } else {
      root.join(Manifest::FILENAME)
    };

    let (overwrite, previous) = match operation {
      Operation::Create { force } => {
        ensure! {
          force || !filesystem::exists(&manifest_path)?,
          error::ManifestAlreadyExists {
            path: manifest_path,
          },
        }

        (force, None)
      }
      Operation::Update => {
        let loader = Loader::load_with_options(DecodeOptions::strict(), Some(&manifest_path))?;

        let manifest = loader.unpack()?;

        ensure! {
          manifest.unknown.is_empty(),
          error::UnknownEntries {
            names: manifest
              .unknown
              .entries
              .keys()
              .cloned()
              .collect::<BTreeSet<ComponentBuf>>(),
            path: loader.path(),
          },
        }

        let revision_hash = loader.revision_hash()?;

        let parents = if State::load(&root)?.is_published(revision_hash) {
          Some(NonEmptyOrderedSet::singleton(revision_hash))
        } else {
          manifest.parents.clone()
        };

        (true, Some((manifest, parents, revision_hash)))
      }
    };

    let path = root.join(Metadata::YAML_FILENAME);

    let metadata = if let Some(yaml) = filesystem::read_to_string_opt(&path)? {
      let metadata = yaml::Metadata::deserialize(&path, &yaml)?;

      if let Some(yaml::Media::Web) = metadata.media {
        options.unstable(UnstableFeature::WebPackages)?;
      }

      let path = root.join(Metadata::DECO_FILENAME);

      ensure! {
        overwrite || !filesystem::exists(&path)?,
        error::MetadataAlreadyExists {
          path,
        },
      }

      Some(metadata)
    } else {
      let path = root.join(Metadata::DECO_FILENAME);

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

    linter.expect(
      self.expect.iter().copied().chain(
        metadata
          .as_ref()
          .and_then(|metadata| metadata.lints.as_ref())
          .and_then(|lints| lints.expect.as_ref())
          .into_iter()
          .flat_map(|expect| expect.as_slice().iter().copied()),
      ),
    )?;

    linter.lint_metadata(metadata.as_ref(), !self.no_generate);

    linter.check()?;

    let metadata = if let Some(metadata) = metadata {
      let mut metadata = metadata.load(&root, options.quiet)?;

      metadata.validate(&root)?;

      if !self.no_generate {
        metadata.generate(&root, overwrite, options.quiet)?;
      }

      let deco = metadata.encode_to_vec();

      Some((metadata, deco))
    } else {
      None
    };

    let mut paths = HashMap::new();

    let Walk {
      empty,
      entries,
      subpackages,
    } = Walk::new(&root, &manifest_path, &self.ignore)?;

    for (path, ty) in entries {
      ensure! {
        path.is_nfc(),
        error::PathNormalization { path },
      }

      linter.lint_path(&path);

      if ty == EntryType::File {
        let size = filesystem::metadata(&root.join(&path))?.len();
        paths.insert(path, size);
      }
    }

    linter.lint_case_conflicts();

    linter.check()?;

    linter.lint_content(
      &root,
      &options,
      metadata.as_ref().map(|(metadata, _deco)| metadata),
    )?;

    linter.done()?;

    if let Some((metadata, _deco)) = &metadata {
      let files = paths.keys().cloned().collect::<HashSet<RelativePath>>();

      metadata.check_files(&files)?;

      if metadata.media.is_some() {
        metadata.check_extras(&files, &empty)?;
      }
    }

    if let Some((_metadata, deco)) = &metadata {
      filesystem::write(&root.join(Metadata::DECO_FILENAME), deco)?;

      paths.insert(
        Metadata::DECO_FILENAME.parse().unwrap(),
        deco.len().into_u64(),
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
        .context(filesystem_error::Io { path: &path })?;
      package.create_file(&path, file)?;
      bar.inc(file.size);
    }

    for path in &subpackages {
      verify_subpackage(&root, path, package.subdirectory(path).unwrap())?;
    }

    check_metadata(&root, &package)?;

    let embedded = if let Some((_metadata, deco)) = metadata {
      BTreeMap::from([(Hash::bytes(&deco), deco.into())])
    } else {
      BTreeMap::new()
    };

    let (parents, signatures, signers) = match previous {
      Some((previous, parents, revision_hash)) => {
        if package == previous.package {
          (previous.parents, previous.signatures, BTreeSet::new())
        } else {
          let count = previous.unknown_signatures();

          ensure! {
            count == 0,
            error::ResignUnknownSignatures { count },
          }

          let signers = previous
            .signatures()
            .map(|signature| {
              signature.verify(revision_hash)?;
              Ok(signature.public_key())
            })
            .collect::<Result<BTreeSet<PublicKey>>>()?;

          (parents, BTreeSet::new(), signers)
        }
      }
      None => (None, BTreeSet::new(), BTreeSet::new()),
    };

    let mut manifest = Manifest {
      embedded,
      package,
      parents,
      signatures,
      unknown: DirectoryTree::new(),
    };

    let resigned = if self.sign || !signers.is_empty() {
      let revision_hash = Archive::pack(&manifest)
        .unwrap()
        .revision_hash(DecodeOptions::default())
        .unwrap();

      let keychain = Keychain::load(&options)?;

      let resigned = signers
        .into_iter()
        .map(|key| {
          keychain
            .name(key)
            .cloned()
            .context(error::ResignKeyMissing { key })
        })
        .collect::<Result<Vec<KeyName>>>()?;

      for name in &resigned {
        manifest.sign(&keychain, name, revision_hash)?;
      }

      if self.sign {
        manifest.sign(&keychain, &self.key, revision_hash)?;
      }

      resigned
    } else {
      Vec::new()
    };

    manifest.save(&manifest_path)?;

    if !options.quiet {
      for name in resigned {
        eprintln!("re-signed with key `{name}`");
      }
    }

    Ok(())
  }
}
