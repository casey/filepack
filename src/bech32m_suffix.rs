use super::*;

type Bytes<'a> = FesToBytes<AsciiToFe32Iter<'a>>;

pub(crate) trait Bech32mSuffix: Sized {
  fn as_bytes(&self) -> &[u8];

  fn from_bytes(ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError>;
}

impl Bech32mSuffix for () {
  fn as_bytes(&self) -> &[u8] {
    &[]
  }

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
}

impl Bech32mSuffix for Vec<u8> {
  fn as_bytes(&self) -> &[u8] {
    self
  }

  fn from_bytes(_ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError> {
    Ok(bytes.collect())
  }
}
