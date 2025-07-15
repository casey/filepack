use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Archive {
  pub(crate) manifest: Hash,
}

impl Archive {
  pub(crate) const EXTENSION: &str = "filepack";
  pub(crate) const FILE_SIGNATURE: &[u8] = b"FILEPACK";

  pub(crate) fn load(path: &Utf8Path) -> Result<Self, ArchiveError> {
    let mut reader = BufReader::new(File::open(path).context(archive_error::FilesystemIo)?);

    let mut signature = [0u8; Self::FILE_SIGNATURE.len()];

    match reader.read_exact(&mut signature) {
      Ok(()) => {}
      Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
        return Err(archive_error::FileSignature.build());
      }
      Err(error) => {
        return Err(archive_error::FilesystemIo.into_error(error));
      }
    }

    if signature != Self::FILE_SIGNATURE {
      return Err(archive_error::FileSignature.build());
    }

    let mut buffer = [0u8; Hash::LEN];

    match reader.read_exact(&mut buffer) {
      Ok(()) => {}
      Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
        return Err(archive_error::Truncated.build());
      }
      Err(error) => {
        return Err(archive_error::FilesystemIo.into_error(error));
      }
    }

    Ok(Self {
      manifest: buffer.into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn file_signature_truncated_error() {
    let tempdir = TempDir::new().unwrap();

    tempdir.child("foo.archive").write_binary(b"foo").unwrap();

    let path = decode_path(tempdir.path()).unwrap();

    assert_matches! {
      Archive::load(&path.join("foo.archive")),
      Err(ArchiveError::FileSignature { .. }),
    }
  }

  #[test]
  fn file_signature_mismatch_error() {
    let tempdir = TempDir::new().unwrap();

    tempdir
      .child("foo.archive")
      .write_binary(b"aaaaaaaa")
      .unwrap();

    let path = decode_path(tempdir.path()).unwrap();

    assert_matches! {
      Archive::load(&path.join("foo.archive")),
      Err(ArchiveError::FileSignature { .. }),
    }
  }

  #[test]
  fn truncated_error() {
    let tempdir = TempDir::new().unwrap();

    tempdir
      .child("foo.archive")
      .write_binary(b"FILEPACK")
      .unwrap();

    let path = decode_path(tempdir.path()).unwrap();

    assert_matches! {
      Archive::load(&path.join("foo.archive")),
      Err(ArchiveError::Truncated { .. }),
    }
  }

  #[test]
  fn success() {
    let tempdir = TempDir::new().unwrap();

    let manifest = Hash::bytes(&[]);

    let archive = Archive::FILE_SIGNATURE
      .iter()
      .chain(manifest.as_bytes())
      .copied()
      .collect::<Vec<u8>>();

    tempdir.child("foo.archive").write_binary(&archive).unwrap();

    let path = decode_path(tempdir.path()).unwrap();

    let archive = Archive::load(&path.join("foo.archive")).unwrap();

    assert_eq!(archive, Archive { manifest });
  }
}
