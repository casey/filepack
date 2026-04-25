use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, FromRepr, PartialEq)]
#[repr(u8)]
pub(crate) enum Version {
  #[default]
  Zero = 0,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(Version::Zero, &[0x00]);
  }
}
