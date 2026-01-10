use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[arg(help = "Verify manifest fingerprint is <FINGERPRINT>", long)]
  fingerprint: Option<Hash>,
  #[arg(help = "Ignore missing files", long)]
  ignore_missing: bool,
  #[arg(help = "Verify that manifest has been signed by <KEY>", long)]
  key: Option<PublicKey>,
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
    let current_dir = current_dir()?;

    let root = self.root.unwrap_or_else(|| current_dir.clone());

    let source = if let Some(ref manifest) = self.manifest {
      manifest.clone()
    } else {
      root.join(Manifest::FILENAME)
    };

    let json = filesystem::read_to_string_opt(&source)?.ok_or_else(|| {
      error::ManifestNotFound {
        path: self
          .manifest
          .as_deref()
          .unwrap_or(Utf8Path::new(Manifest::FILENAME)),
      }
      .build()
    })?;

    let manifest = serde_json::from_str::<Manifest>(&json).context(error::DeserializeManifest {
      path: Manifest::FILENAME,
    })?;

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

    let bar = progress_bar::new(&options, manifest.total_size().unwrap_or(u64::MAX));

    let mut mismatches = BTreeMap::new();

    for (path, expected) in manifest.files() {
      let actual = match options.hash_file(&root.join(&path)) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
          ensure! {
            self.ignore_missing,
            error::MissingFile { path },
          }
          continue;
        }
        result => result.context(error::FilesystemIo { path: &path })?,
      };

      if actual != expected {
        mismatches.insert(path, (actual, expected));
      }

      bar.inc(expected.size);
    }

    if !mismatches.is_empty() {
      for (path, (actual, expected)) in &mismatches {
        let style = Style::stderr();

        let hash_style = if expected.hash == actual.hash {
          style.good()
        } else {
          style.bad()
        };

        let size_style = if expected.size == actual.size {
          style.good()
        } else {
          style.bad()
        };

        eprintln!(
          "\
mismatched file: `{path}`
       manifest: {} ({} bytes)
           file: {} ({} bytes)",
          expected.hash.style(style.good()),
          expected.size.style(style.good()),
          actual.hash.style(hash_style),
          actual.size.style(size_style),
        );
      }

      return Err(
        error::EntryMismatch {
          count: mismatches.len(),
        }
        .build(),
      );
    }

    let files = manifest.files();

    let mut empty = Vec::<RelativePath>::new();

    for entry in WalkDir::new(&root) {
      let entry = entry?;

      let path = entry.path();

      let path = decode_path(path)?;

      if path == root {
        continue;
      }

      if current_dir.join(path) == current_dir.join(&source) {
        continue;
      }

      let path = path.strip_prefix(&root).unwrap();

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      empty.pop_if(|empty| path.starts_with(&empty));

      if entry.file_type().is_dir() {
        empty.push(path);
        continue;
      }

      ensure! {
        files.contains_key(&path),
        error::ExtraneousFile { path },
      }
    }

    let manifest_empty = manifest.empty_directories();

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
      let path = root.join(Metadata::FILENAME);

      if let Some(json) = filesystem::read_to_string_opt(&path)? {
        serde_json::from_str::<Metadata>(&json)
          .context(error::DeserializeMetadata { path: &path })?;
      }
    }

    for (public_key, signature) in &manifest.signatures {
      public_key.verify(fingerprint.as_bytes(), signature)?;
    }

    if let Some(key) = self.key {
      ensure! {
        manifest.signatures.contains_key(&key),
        error::SignatureMissing { key },
      }
    }

    if self.print {
      print!("{json}");
    }

    Ok(())
  }
}
