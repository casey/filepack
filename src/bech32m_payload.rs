use super::*;

pub(crate) struct Bech32mPayload<const PREFIX: usize, const DATA: usize> {
  pub(crate) prefix: [Fe32; PREFIX],
  pub(crate) data: [u8; DATA],
}

impl<const DATA: usize> Bech32mPayload<0, DATA> {
  pub(crate) fn into_data(self) -> [u8; DATA] {
    self.data
  }
}

impl<const PREFIX: usize, const DATA: usize> Bech32mPayload<PREFIX, DATA> {
  pub(crate) fn into_prefix_and_data(self) -> ([Fe32; PREFIX], [u8; DATA]) {
    (self.prefix, self.data)
  }
}
