use super::*;

#[derive(Debug)]
pub(crate) struct Bech32mPayload<const PREFIX: usize, const BODY: usize, T> {
  pub(crate) body: [u8; BODY],
  pub(crate) prefix: [Fe32; PREFIX],
  pub(crate) suffix: T,
}
