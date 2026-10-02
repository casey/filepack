use super::*;

pub(crate) struct Reader<'a> {
  bytes: &'a [u8],
  offset: usize,
}

impl<'a> Reader<'a> {
  fn array<const N: usize>(&mut self) -> Result<[u8; N], Mp4Error> {
    Ok(self.bytes(N)?.try_into().unwrap())
  }

  pub(crate) fn bytes(&mut self, len: usize) -> Result<&'a [u8], Mp4Error> {
    let bytes = self
      .bytes
      .get(self.offset..self.offset + len)
      .context(mp4_error::Truncated)?;

    self.offset += len;

    Ok(bytes)
  }

  pub(crate) fn container(self) -> Result<Container<'a>, Mp4Error> {
    Container::parse(self.rest())
  }

  pub(crate) fn fourcc(&mut self) -> Result<Fourcc, Mp4Error> {
    Ok(Fourcc(self.array()?))
  }

  pub(crate) fn i32(&mut self) -> Result<i32, Mp4Error> {
    Ok(i32::from_be_bytes(self.array()?))
  }

  pub(crate) fn new(bytes: &'a [u8]) -> Self {
    Self { bytes, offset: 0 }
  }

  pub(crate) fn rest(self) -> &'a [u8] {
    &self.bytes[self.offset..]
  }

  pub(crate) fn skip(&mut self, len: usize) -> Result<(), Mp4Error> {
    self.bytes(len)?;
    Ok(())
  }

  pub(crate) fn u16(&mut self) -> Result<u16, Mp4Error> {
    Ok(u16::from_be_bytes(self.array()?))
  }

  pub(crate) fn u32(&mut self) -> Result<u32, Mp4Error> {
    Ok(u32::from_be_bytes(self.array()?))
  }

  pub(crate) fn u64(&mut self) -> Result<u64, Mp4Error> {
    Ok(u64::from_be_bytes(self.array()?))
  }

  pub(crate) fn u8(&mut self) -> Result<u8, Mp4Error> {
    Ok(u8::from_be_bytes(self.array()?))
  }

  pub(crate) fn version(&mut self) -> Result<u8, Mp4Error> {
    let version = self.u8()?;
    self.skip(3)?;
    Ok(version)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fields() {
    let mut reader = Reader::new(&[
      1, 0, 2, 0, 0, 0, 3, 0xff, 0xff, 0xff, 0xff, b'a', b'b', b'c', b'd', 9,
    ]);
    assert_eq!(reader.u8().unwrap(), 1);
    assert_eq!(reader.u16().unwrap(), 2);
    assert_eq!(reader.u32().unwrap(), 3);
    assert_eq!(reader.i32().unwrap(), -1);
    assert_eq!(reader.fourcc().unwrap(), Fourcc(*b"abcd"));
    assert_eq!(reader.rest(), &[9]);
  }

  #[test]
  fn truncated() {
    let mut reader = Reader::new(&[1, 2, 3]);
    assert_matches!(reader.u32().unwrap_err(), Mp4Error::Truncated);
    assert_eq!(reader.rest(), &[1, 2, 3]);
  }

  #[test]
  fn version() {
    let mut reader = Reader::new(&[1, 2, 3, 4, 5]);
    assert_eq!(reader.version().unwrap(), 1);
    assert_eq!(reader.rest(), &[5]);
  }
}
