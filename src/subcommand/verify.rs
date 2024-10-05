use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[arg(
    help = "Verify that manifest has been signed by <PUBKEY>",
    long,
    value_name = "PUBKEY"
  )]
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
    let current_dir = Utf8PathBuf::from_path_buf(env::current_dir().context(error::CurrentDir)?)
      .map_err(|path| error::PathUnicode { path }.build())?;

    let root = if let Some(root) = self.root {
      root
    } else {
      current_dir.clone()
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

    let manifest = serde_json::from_str::<Manifest>(&json).context(error::DeserializeManifest {
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

    let mut signatures = BTreeSet::new();

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

      if path == SIGNATURES {
        continue;
      }

      if path.starts_with(SIGNATURES) {
        ensure! {
          path.extension() == Some("signature"),
          error::SignatureFilename { path },
        }

        let pubkey = path
          .file_stem()
          .context(error::SignatureFilename { path })?
          .parse::<PublicKey>()
          .context(error::SignaturePublicKey { path })?;

        let signature = fs::read_to_string(entry.path()).context(error::Io { path })?;

        let signature = signature
          .parse::<Signature>()
          .context(error::SignatureMalformed { path })?;

        pubkey
          .verify(json.as_bytes(), &signature)
          .context(error::SignatureInvalid { path })?;

        signatures.insert(pubkey);

        continue;
      }

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      ensure! {
        manifest.files.contains_key(&path),
        error::ExtraneousFile { path },
      }
    }

    if let Some(key) = self.key {
      ensure! {
        signatures.contains(&key),
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
