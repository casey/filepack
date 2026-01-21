use super::*;

#[derive(Debug)]
pub(crate) struct Bech32mPayload<const PREFIX: usize, const DATA: usize, T> {
  pub(crate) data: [u8; DATA],
  pub(crate) prefix: [Fe32; PREFIX],
  pub(crate) suffix: T,
}

impl<const DATA: usize> Bech32mPayload<0, DATA, ()> {
  pub(crate) fn from_data(data: [u8; DATA]) -> Self {
    Self {
      data,
      prefix: [],
      suffix: (),
    }
  }

  pub(crate) fn into_data(self) -> [u8; DATA] {
    self.data
  }
}
