use super::*;

#[derive(Debug)]
pub(crate) struct Bech32mPayload<const PREFIX: usize, const BODY: usize, T> {
  pub(crate) body: [u8; BODY],
  pub(crate) prefix: [Fe32; PREFIX],
  pub(crate) suffix: T,
}

impl<const BODY: usize> Bech32mPayload<0, BODY, ()> {
  pub(crate) fn from_body(body: [u8; BODY]) -> Self {
    Self {
      body,
      prefix: [],
      suffix: (),
    }
  }

  pub(crate) fn into_body(self) -> [u8; BODY] {
    self.body
  }
}
