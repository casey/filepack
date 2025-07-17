use super::*;

use std::io::Seek;
use std::io::SeekFrom;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Archive {
  pub(crate) hash: Hash,
  pub(crate) manifest: Manifest,
}

struct Listing {
  hash: Hash,
  offset: u64,
  size: u64,
}

struct ArchiveReader(BufReader<File>);

impl Seek for ArchiveReader {
  fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
    self.0.seek(pos)
  }
}

impl ArchiveReader {
  fn read_signature(&mut self) -> Result<[u8; Archive::FILE_SIGNATURE.len()], ArchiveError> {
    let mut signature = [0u8; Archive::FILE_SIGNATURE.len()];

    match self.0.read_exact(&mut signature) {
      Ok(()) => {}
      Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
        return Err(archive_error::FileSignature.build());
      }
      Err(error) => {
        return Err(archive_error::FilesystemIo.into_error(error));
      }
    }

    Ok(signature)
  }

  fn read(&mut self, buffer: &mut [u8]) -> Result<(), ArchiveError> {
    match self.0.read_exact(buffer) {
      Ok(()) => Ok(()),
      Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
        Err(archive_error::Truncated.build())
      }
      Err(error) => Err(archive_error::FilesystemIo.into_error(error)),
    }
  }

  fn read_hash(&mut self) -> Result<Hash, ArchiveError> {
    let mut hash = [0u8; Hash::LEN];
    self.read(&mut hash)?;
    Ok(hash.into())
  }

  fn read_u64(&mut self) -> Result<u64, ArchiveError> {
    let mut n = [0u8; 8];
    self.read(&mut n)?;
    Ok(u64::from_le_bytes(n))
  }
}

impl Archive {
  pub(crate) const EXTENSION: &str = "filepack";
  pub(crate) const FILE_SIGNATURE: &[u8] = b"FILEPACK";

  pub(crate) fn load(path: &Utf8Path) -> Result<Self, ArchiveError> {
    let mut reader = ArchiveReader(BufReader::new(
      File::open(path).context(archive_error::FilesystemIo)?,
    ));

    let signature = reader.read_signature()?;

    if signature != Self::FILE_SIGNATURE {
      return Err(archive_error::FileSignature.build());
    }

    let manifest_hash = reader.read_hash()?;

    let count = reader.read_u64()?;

    let mut listings = Vec::new();

    for _ in 0..count {
      let hash = reader.read_hash()?;
      let offset = reader.read_u64()?;
      let size = reader.read_u64()?;
      listings.push(Listing { hash, offset, size });
    }

    let manifest = listings
      .iter()
      .find(|listing| listing.hash == manifest_hash.into())
      .context(archive_error::ManifestMissing)?;

    reader
      .seek(SeekFrom::Current(manifest.offset as i64))
      .unwrap();

    let mut content = vec![0; manifest.size as usize];
    reader.read(&mut content)?;

    Ok(Self {
      manifest: serde_json::from_slice::<Manifest>(&content).unwrap(),
      hash: manifest_hash.into(),
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

    let hash = Hash::bytes(&[]);

    let manifest = Manifest::default();

    let manifest_content = serde_json::to_string(&manifest).unwrap();

    let archive = Archive::FILE_SIGNATURE
      .iter()
      .chain(hash.as_bytes())
      .chain(&1u64.to_le_bytes())
      .chain(hash.as_bytes())
      .chain(&0u64.to_le_bytes())
      .chain(&manifest_content.len().to_le_bytes())
      .chain(manifest_content.as_bytes())
      .copied()
      .collect::<Vec<u8>>();

    tempdir.child("foo.archive").write_binary(&archive).unwrap();

    let path = decode_path(tempdir.path()).unwrap();

    let archive = Archive::load(&path.join("foo.archive")).unwrap();

    assert_eq!(archive, Archive { hash, manifest });
  }
}
