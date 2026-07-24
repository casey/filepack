pub(crate) struct BitReader<'a> {
  i: usize,
  bytes: &'a [u8],
}

impl<'a> BitReader<'a> {
  pub(crate) fn bit(&mut self) -> Option<u64> {
    let byte = self.bytes.get(self.i / 8)?;
    let bit = u64::from(byte >> (7 - self.i % 8) & 1);
    self.i += 1;
    Some(bit)
  }

  pub(crate) fn bits(&mut self, n: u32) -> Option<u64> {
    let mut value = 0;

    for _ in 0..n {
      value = value << 1 | self.bit()?;
    }

    Some(value)
  }

  pub(crate) fn new(bytes: &'a [u8]) -> Self {
    Self { i: 0, bytes }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bits() {
    let mut reader = BitReader::new(&[0b1010_0110, 0b1100_0000]);

    assert_eq!(reader.bit(), Some(1));
    assert_eq!(reader.bits(3), Some(0b010));
    assert_eq!(reader.bits(6), Some(0b01_1011));
    assert_eq!(reader.bits(7), None);
  }
}
