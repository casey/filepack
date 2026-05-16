use super::*;

pub(crate) struct HashingWriter {
  hasher: Hasher,
  inner: NamedTempFile,
}

impl HashingWriter {
  pub(crate) fn finalize(self) -> (Hash, NamedTempFile) {
    (self.hasher.finalize().into(), self.inner)
  }

  pub(crate) fn new(inner: NamedTempFile) -> Self {
    Self {
      hasher: Hasher::new(),
      inner,
    }
  }
}

impl Write for HashingWriter {
  fn flush(&mut self) -> io::Result<()> {
    self.inner.flush()
  }

  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let n = self.inner.write(buf)?;
    self.hasher.update(&buf[..n]);
    Ok(n)
  }
}
