pub(crate) struct BitReader<'a> {
  bytes: &'a [u8],
  i: usize,
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
    Self { bytes, i: 0 }
  }

  pub(crate) fn ue(&mut self) -> Option<u64> {
    let mut zeros = 0;

    while self.bit()? == 0 {
      zeros += 1;

      if zeros > 31 {
        return None;
      }
    }

    Some((1 << zeros) - 1 + self.bits(zeros)?)
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

  #[test]
  fn ue() {
    #[track_caller]
    fn case(data: &[u8], expected: Option<u64>) {
      assert_eq!(BitReader::new(data).ue(), expected);
    }

    case(&[0b1000_0000], Some(0));
    case(&[0b0100_0000], Some(1));
    case(&[0b0110_0000], Some(2));
    case(&[0b0001_0010], Some(8));
    case(&[0b0000_0000], None);
    case(&[], None);
  }
}
