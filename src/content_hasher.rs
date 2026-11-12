use super::*;

/// Incremental BLAKE3 hasher over file content, producing the same [`Hash`]
/// and size a manifest records for the file.
#[derive(Clone, Default)]
pub struct ContentHasher {
  hasher: Hasher,
}

impl ContentHasher {
  #[must_use]
  pub fn finalize(&self) -> Hash {
    self.hasher.finalize().into()
  }

  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[must_use]
  pub fn size(&self) -> u64 {
    self.hasher.count()
  }

  pub fn update(&mut self, bytes: &[u8]) -> &mut Self {
    self.hasher.update(bytes);
    self
  }
}

impl Write for ContentHasher {
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }

  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.hasher.update(buf);
    Ok(buf.len())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn matches_hash_bytes() {
    let mut hasher = ContentHasher::new();
    hasher.update(b"foo").update(b"bar");
    assert_eq!(hasher.finalize(), Hash::bytes(b"foobar"));
    assert_eq!(hasher.size(), 6);
  }

  #[test]
  fn writer() {
    let mut hasher = ContentHasher::new();
    io::copy(&mut &b"foobar"[..], &mut hasher).unwrap();
    assert_eq!(hasher.finalize(), Hash::bytes(b"foobar"));
  }
}
