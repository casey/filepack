pub(crate) struct BitReader<'a> {
  bit: usize,
  data: &'a [u8],
}

impl<'a> BitReader<'a> {
  pub(crate) fn bit(&mut self) -> Option<u64> {
    let byte = self.data.get(self.bit / 8)?;
    let bit = u64::from(byte >> (7 - self.bit % 8) & 1);
    self.bit += 1;
    Some(bit)
  }

  pub(crate) fn bits(&mut self, n: u32) -> Option<u64> {
    let mut value = 0;

    for _ in 0..n {
      value = value << 1 | self.bit()?;
    }

    Some(value)
  }

  pub(crate) fn new(data: &'a [u8]) -> Self {
    Self { bit: 0, data }
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
