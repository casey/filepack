use super::*;

pub struct Loader {
  archive: Archive,
  options: DecodeOptions,
  path: Utf8PathBuf,
}

impl Loader {
  pub(crate) fn archive(&self) -> &Archive {
    &self.archive
  }

  pub fn fingerprint(&self) -> Result<Fingerprint> {
    self
      .archive
      .fingerprint_with_options(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }

  pub fn load(path: Option<&Utf8Path>) -> Result<Self> {
    Self::load_with_options(DecodeOptions::new(), path)
  }

  pub(crate) fn load_with_options(options: DecodeOptions, path: Option<&Utf8Path>) -> Result<Self> {
    let path = if let Some(path) = path {
      if path.is_dir() {
        path.join(Manifest::FILENAME)
      } else {
        path.into()
      }
    } else {
      Manifest::FILENAME.into()
    };

    let deco = filesystem::read_opt(&path)?
      .ok_or_else(|| error::ManifestNotFound { path: &path }.build())?;

    let archive = Archive::decode_magic_bytes_with_options(options, &deco)
      .context(error::DecodeManifest { path: &path })?;

    Ok(Self {
      archive,
      options,
      path,
    })
  }

  pub(crate) fn package(&self) -> Result<Entry> {
    self
      .archive
      .package(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }

  pub(crate) fn path(&self) -> &Utf8Path {
    &self.path
  }

  pub(crate) fn unpack(&self) -> Result<Manifest> {
    self
      .archive
      .unpack_with_options(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }

  pub(crate) fn unpack_with_totals(&self) -> Result<(Manifest, Totals)> {
    self
      .archive
      .unpack_with_totals(self.options)
      .context(error::UnarchiveManifest { path: &self.path })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn directory(builder: &mut ArchiveBuilder, directory: &Directory, unknown: bool) -> Entry {
    let bytes = if unknown {
      with_unknown_field(directory)
    } else {
      directory.encode_to_vec()
    };
    let file = builder.file(bytes);
    Entry::directory(file.hash(), file.size(), directory.totals().unwrap())
  }

  #[test]
  fn unknown_directory_fields() {
    for unknown in [
      None,
      Some("root"),
      Some("package"),
      Some("foo"),
      Some("signatures"),
    ] {
      let mut builder = ArchiveBuilder::new();

      let foo = directory(&mut builder, &Directory::new(), unknown == Some("foo"));

      let mut package = Directory::new();
      package.insert_entry("foo", foo);
      let totals = package.totals().unwrap();
      let package = directory(&mut builder, &package, unknown == Some("package"));
      let fingerprint = Fingerprint(package.hash());

      let signature = test::PRIVATE_KEY
        .parse::<PrivateKey>()
        .unwrap()
        .sign(Statement {
          version: Version::Zero,
          fingerprint,
          timestamp: None,
        });

      let signature_file = builder.file(signature.encode_to_vec());
      let signatures = directory(
        &mut builder,
        Directory::new().insert_entry("0", signature_file),
        unknown == Some("signatures"),
      );

      let root = directory(
        &mut builder,
        Directory::new()
          .insert_entry("package", package.clone())
          .insert_entry("signatures", signatures),
        unknown == Some("root"),
      );

      let archive = builder.build(root.hash());
      let (_tempdir, path) = tempdir();
      let path = path.join(Manifest::FILENAME);
      fs::write(&path, archive.encode_magic_bytes()).unwrap();

      let mut tree = DirectoryTree::new();
      tree.create_directory(&"foo".parse().unwrap()).unwrap();
      let manifest = Manifest {
        embedded: BTreeMap::new(),
        package: tree,
        signatures: BTreeSet::from([signature]),
      };

      assert_eq!(
        Loader::load(Some(&path)).unwrap().unpack().unwrap(),
        manifest
      );

      let loader = Loader::load_with_options(DecodeOptions::strict(), Some(&path)).unwrap();

      let reject = |error| {
        assert_matches!(
          error,
          Error::UnarchiveManifest {
            backtrace: _,
            path: actual,
            source: ArchiveError::DirectoryDecode {
              source: DecodeError::UnknownField { key: u64::MAX },
            },
          } if actual.to_string() == path.as_str(),
          "{unknown:?}",
        );
      };

      if unknown.is_none() {
        assert_eq!(loader.unpack().unwrap(), manifest);
        assert_eq!(loader.unpack_with_totals().unwrap(), (manifest, totals));
      } else {
        reject(loader.unpack().unwrap_err());
        reject(loader.unpack_with_totals().unwrap_err());
      }

      if unknown == Some("root") {
        reject(loader.package().unwrap_err());
        reject(loader.fingerprint().unwrap_err());
      } else {
        assert_eq!(loader.package().unwrap(), package);
        assert_eq!(loader.fingerprint().unwrap(), fingerprint);
      }
    }
  }
}
