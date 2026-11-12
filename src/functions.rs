use {
  super::*,
  ::image::{DynamicImage, ImageBuffer, Rgb, Rgba},
};

pub(crate) fn check_case(dir: &Utf8Path, names: &[&'static str]) -> FilesystemResult {
  for entry in fs::read_dir(dir).context(filesystem_error::Io { path: dir })? {
    let actual = entry
      .context(filesystem_error::Io { path: dir })?
      .file_name();

    for &expected in names {
      ensure! {
        !actual.as_encoded_bytes().eq_ignore_ascii_case(expected.as_bytes()) || actual == expected,
        filesystem_error::FilenameCase {
          actual: actual.to_str().unwrap(),
          expected,
        }
      }
    }
  }

  Ok(())
}

#[cfg(feature = "fs")]
pub(crate) fn check_metadata(root: &Utf8Path, package: &DirectoryTree) -> Result {
  for path in package.files().into_keys() {
    if path.filename() != Metadata::DECO_FILENAME {
      continue;
    }

    let (root, package) = match path.parent() {
      Some(parent) => (root.join(&parent), package.subdirectory(&parent).unwrap()),
      None => (root.to_owned(), package),
    };

    if let Some(metadata) = Metadata::load(&root)? {
      metadata.check_files(&package.files().into_keys().collect())?;
    }
  }

  Ok(())
}

pub(crate) fn current_dir() -> Result<Utf8PathBuf> {
  Utf8PathBuf::from_path_buf(env::current_dir().context(error::CurrentDir)?)
    .map_err(|path| error::PathUnicode { path }.build())
}

pub(crate) fn decode_path(path: &Path) -> Result<&Utf8Path> {
  Utf8Path::from_path(path).context(error::PathUnicode { path })
}

pub(crate) fn default<T: Default>() -> T {
  Default::default()
}

pub(crate) fn format_size(size: u64) -> SizeFormatter<u64, FormatSizeOptions> {
  SizeFormatter::new(size, FormatSizeOptions::from(BINARY).decimal_places(1))
}

pub fn gradient(width: u32, height: u32) -> DynamicImage {
  ImageBuffer::from_fn(width, height, |x, y| {
    Rgb([
      u8::try_from(x / 8 % 256).unwrap(),
      u8::try_from(y / 8 % 256).unwrap(),
      u8::try_from((x + y) / 16 % 256).unwrap(),
    ])
  })
  .into()
}

pub fn gradient_alpha(width: u32, height: u32, alpha: u8) -> DynamicImage {
  ImageBuffer::from_fn(width, height, |x, y| {
    Rgba([
      u8::try_from(x / 8 % 256).unwrap(),
      u8::try_from(y / 8 % 256).unwrap(),
      u8::try_from((x + y) / 16 % 256).unwrap(),
      alpha,
    ])
  })
  .into()
}

#[cfg(any(feature = "client", feature = "server"))]
pub fn install_default_crypto_provider() -> Result {
  static INSTALLED: LazyLock<bool> = LazyLock::new(|| {
    rustls::crypto::ring::default_provider()
      .install_default()
      .is_ok()
  });

  ensure!(*INSTALLED, error::RustlsProvider);

  Ok(())
}

/// Seconds since the Unix epoch. On wasm32-unknown-unknown, where
/// `SystemTime::now` panics, the clock is read through JavaScript.
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn now() -> Result<u64, SystemTimeError> {
  Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Seconds since the Unix epoch. On wasm32-unknown-unknown, where
/// `SystemTime::now` panics, the clock is read through JavaScript.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn now() -> Result<u64, SystemTimeError> {
  Ok(
    web_time::SystemTime::now()
      .duration_since(web_time::UNIX_EPOCH)
      .expect("system clock is before the unix epoch")
      .as_secs(),
  )
}

pub(crate) fn parse_number<T: FromStr<Err = ParseIntError>>(s: &str) -> Result<T, NumberError> {
  ensure! {
    re::NUMBER.is_match(s),
    number_error::Invalid { number: s },
  }

  Ok(s.parse()?)
}

#[cfg(feature = "fs")]
pub(crate) fn transfer_tempfile(hash: Hash, path: &Utf8Path) -> FilesystemResult<NamedTempFile> {
  tempfile::Builder::new()
    .prefix(&format!("{hash}-"))
    .suffix(".incomplete")
    .tempfile_in(path)
    .context(filesystem_error::Io { path })
}

#[cfg(feature = "fs")]
pub(crate) fn verify_subpackage(
  root: &Utf8Path,
  path: &RelativePath,
  package: &DirectoryTree,
) -> Result {
  let root = root.join(path);

  let loader = Loader::load(Some(&root.join(Manifest::FILENAME)))?;

  let manifest = loader.unpack()?;

  let expected = loader.fingerprint()?;

  let actual = Fingerprint(
    ArchiveBuilder::new()
      .pack_directory(package)
      .context(error::SubpackageTotals { path: path.clone() })?
      .hash,
  );

  ensure! {
    actual == expected,
    error::SubpackageMismatch {
      actual,
      expected,
      path: path.clone(),
    },
  }

  for signature in manifest.signatures() {
    signature.verify(expected)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check_case() {
    #[track_caller]
    fn case(name: &str, expected: Option<&str>) {
      let (_tempdir, root) = tempdir();

      filesystem::write(&root.join(name), "").unwrap();

      assert_eq!(
        super::check_case(&root, &["foo", "bar"])
          .err()
          .map(|err| err.to_string())
          .as_deref(),
        expected,
      );
    }

    case("foo", None);
    case("baz", None);
    case("FOO", Some("expected `foo` but found case mismatch `FOO`"));
    case("Bar", Some("expected `bar` but found case mismatch `Bar`"));
  }
}
