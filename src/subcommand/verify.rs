use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[arg(help = "Verify manifest fingerprint is <FINGERPRINT>", long)]
  fingerprint: Option<Fingerprint>,
  #[arg(help = "Ignore missing files", long)]
  ignore_missing: bool,
  #[arg(
    help = "Verify that manifest has been signed by <KEY>",
    long = "key",
    value_name = "KEY"
  )]
  keys: Vec<KeyIdentifier>,
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
    #[derive(Default)]
    struct Verified {
      bytes: u128,
      files: u64,
      notes: u64,
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

    let mut verified = Verified::default();

    let fingerprint = manifest.verify_notes()?;

    verified.signatures += manifest
      .notes
      .iter()
      .map(|note| note.signatures.len().into_u64())
      .sum::<u64>();
    verified.notes += manifest.notes.len().into_u64();

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

    let bar = progress_bar::new(
      &options,
      u64::try_from(manifest.total_size()).unwrap_or(u64::MAX),
    );

    let mut mismatches = BTreeMap::new();

    let files = manifest.files();

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

    let mut empty = Vec::new();

    for entry in WalkDir::new(&root) {
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

      if let Some(yaml) = filesystem::read_to_string_opt(&path)? {
        Metadata::deserialize(&path, &yaml)?;
      }
    }

    for (key, identifier) in keys {
      ensure! {
        manifest.notes.iter().any(|note| note.has_signature(key)),
        error::SignatureMissing { identifier: identifier.clone() },
      }
    }

    if self.print {
      print!("{json}");
    }

    eprintln!(
      "successfully verified {} totaling {} with {} across {}",
      Count(verified.files, "file"),
      Count(verified.bytes, "byte"),
      Count(verified.signatures, "signature"),
      Count(verified.notes, "note"),
    );

    Ok(())
  }
}
