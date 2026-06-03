use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub enum Version {
  #[default]
  #[n(0)]
  Zero,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(Version::Zero, "00");
  }
}
