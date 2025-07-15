use super::*;

#[derive(Clone)]
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
