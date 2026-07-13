use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[arg(help = "Verify package fingerprint is <FINGERPRINT>", long)]
  fingerprint: Option<Fingerprint>,
  #[arg(help = "Ignore <PATH>", long, value_name = "PATH")]
  ignore: Vec<RelativePath>,
  #[arg(help = "Ignore missing files", long)]
  ignore_missing: bool,
  #[arg(
    help = "Verify that manifest has been signed by <KEY>",
    long = "key",
    value_name = "KEY"
  )]
  keys: Vec<KeyIdentifier>,
  #[arg(
    help = "Read manifest from <MANIFEST>, defaults to `<ROOT>/manifest.filepack`",
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
    #[derive(Default)]
    struct Verified {
      bytes: u128,
      files: u64,
      signatures: u64,
    }

    let keychain = Keychain::load(&options)?;

    let mut keys = BTreeMap::new();
    for second in &self.keys {
      let key = keychain.identifier_public_key(second)?;
      if let Some(first) = keys.insert(key, second) {
        return Err(
          error::DuplicateKey {
            first: first.clone(),
            second: second.clone(),
          }
          .build(),
        );
      }
    }

    let current_dir = current_dir()?;

    let root = self.root.clone().unwrap_or_else(|| current_dir.clone());

    let source = if let Some(manifest) = &self.manifest {
      ensure! {
        !filesystem::exists(&root.join(Manifest::FILENAME))?,
        error::ManifestInPackage {
          path: Manifest::FILENAME,
        },
      }

      manifest.clone()
    } else if let Some(root) = &self.root {
      root.join(Manifest::FILENAME)
    } else {
      Manifest::FILENAME.into()
    };

    let archive = Archive::load(&source)?;

    let (manifest, totals) = archive
      .unpack_with_totals()
      .context(error::UnarchiveManifest { path: &source })?;

    manifest.verify_signatures()?;

    let mut verified = Verified::default();

    verified.signatures += manifest.signatures.len().into_u64();

    let fingerprint = manifest.fingerprint();

    if let Some(expected) = self.fingerprint
      && fingerprint != expected
    {
      let style = Style::stderr();
      eprintln!(
        "\
fingerprint mismatch: `{source}`
            expected: {}
              actual: {}",
        expected.style(style.good()),
        fingerprint.style(style.bad()),
      );
      return Err(error::FingerprintMismatch.build());
    }

    let bar = progress_bar::new(&options, totals.file_size);

    let mut mismatches = BTreeMap::new();

    let files = manifest.files();

    let manifest_empty = manifest.empty_directories();

    for ignore in &self.ignore {
      ensure! {
        !files.keys().chain(&manifest_empty).any(|path| path.starts_with(ignore)),
        error::IgnoredPath { path: ignore.clone() },
      }
    }

    for (path, expected) in &files {
      let actual = match options.hash_file(&root.join(path)) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
          ensure! {
            self.ignore_missing,
            error::MissingFile { path },
          }
          continue;
        }
        result => result.context(error::FilesystemIo { path: &path })?,
      };

      if actual == *expected {
        verified.files += 1;
        verified.bytes += u128::from(expected.size);
      } else {
        mismatches.insert(path, (actual, expected));
      }

      bar.inc(expected.size);
    }

    if !mismatches.is_empty() {
      for (path, (actual, expected)) in &mismatches {
        File::eprint_mismatch(*actual, **expected, path.as_ref());
      }

      return Err(
        error::EntryMismatch {
          count: mismatches.len(),
        }
        .build(),
      );
    }

    let mut empty = Vec::new();

    for entry in WalkDir::new(&root).sort_by_file_name() {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let path = decode_path(entry.path())?;

      if current_dir.join(path) == current_dir.join(&source) {
        continue;
      }

      let path = path.strip_prefix(&root).unwrap();

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      if self.ignore.iter().any(|ignore| path.starts_with(ignore)) {
        continue;
      }

      empty.pop_if(|dir| path.starts_with(dir));

      if entry.file_type().is_dir() {
        empty.push(path);
        continue;
      }

      ensure! {
        files.contains_key(&path),
        error::ExtraneousFile { path },
      }
    }

    for path in &manifest_empty {
      ensure! {
        empty.iter().any(|dir| dir.starts_with(path)),
        error::MissingDirectory { path },
      }
    }

    for path in &empty {
      ensure! {
        manifest_empty.contains(path),
        error::ExtraneousDirectory { path },
      }
    }

    {
      let path = root.join(Metadata::CBOR_FILENAME);

      if let Some(cbor) = filesystem::read_opt(&path)? {
        Metadata::decode_from_slice(&cbor)
          .context(error::DecodeMetadataCbor { path })?
          .check_files(&files.keys().cloned().collect())?;
      }
    }

    for (key, identifier) in keys {
      ensure! {
        manifest.signatures.iter().any(|signature| signature.public_key() == key),
        error::SignatureMissing { identifier: identifier.clone() },
      }
    }

    if self.print {
      println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
    }

    eprint!(
      "successfully verified {}",
      Count::new(verified.files, "file")
    );

    if verified.files > 0 {
      eprint!(" totaling {}", Count::new(verified.bytes, "byte"));
    }

    if verified.signatures > 0 {
      eprint!(" with {}", Count::new(verified.signatures, "signature"));
    }

    eprintln!();

    Ok(())
  }
}
