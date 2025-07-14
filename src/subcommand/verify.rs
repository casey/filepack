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

    if let Some(expected) = self.fingerprint {
      if fingerprint != expected {
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
    }

    let bar = progress_bar::new(manifest.files.values().map(|entry| entry.size).sum());

    let mut mismatches = BTreeMap::new();

    for (path, &expected) in &manifest.files {
      let actual = match options.hash_file(&root.join(path)) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
          ensure! {
            self.ignore_missing,
            error::MissingFile { path },
          }
          continue;
        }
        result => result.context(error::FilesystemIo { path })?,
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

      if current_dir.join(path) == current_dir.join(&source) {
        continue;
      }

      let path = path.strip_prefix(&root).unwrap();

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      ensure! {
        manifest.files.contains_key(&path),
        error::ExtraneousFile { path },
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
