use super::*;

pub(crate) struct FingerprintSerializer<T> {
  inner: T,
  tag: u64,
}

impl<T: io::Write> FingerprintSerializer<T> {
  pub(crate) fn field(&mut self, tag: u64, field: &[u8]) -> io::Result<()> {
    assert!(tag >= self.tag, "unexpected tag {tag}");
    self.tag = tag;
    self.inner.write_all(&tag.to_le_bytes())?;
    self
      .inner
      .write_all(&field.len().into_u64().to_le_bytes())?;
    self.inner.write_all(field)?;
    Ok(())
  }

  pub(crate) fn into_inner(self) -> T {
    self.inner
  }

  pub(crate) fn new(context: FingerprintPrefix, mut inner: T) -> io::Result<Self> {
    let prefix = context.prefix();
    inner.write_all(&prefix.len().into_u64().to_le_bytes())?;
    inner.write_all(prefix.as_bytes())?;
    Ok(Self { inner, tag: 0 })
  }
}
