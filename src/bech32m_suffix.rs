use super::*;

type Bytes<'a> = FesToBytes<AsciiToFe32Iter<'a>>;

pub(crate) trait Bech32mSuffix: Sized {
  fn from_bytes(ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError>;

  fn into_bytes(self) -> Vec<u8>;
}

impl Bech32mSuffix for () {
  fn from_bytes(ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError> {
    let actual = bytes.count();

    ensure! {
      actual == 0,
      bech32m_error::SuffixLength {
        actual,
        expected: 0usize,
        ty,
      },
    }

    Ok(())
  }

  fn into_bytes(self) -> Vec<u8> {
    Vec::new()
  }
}

impl Bech32mSuffix for Vec<u8> {
  fn from_bytes(_ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError> {
    Ok(bytes.collect())
  }

  fn into_bytes(self) -> Vec<u8> {
    self
  }
}
