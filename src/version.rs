use super::*;

#[derive(Clone, Copy, Debug, Default, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum Version {
  #[default]
  #[n(0)]
  Zero,
}

impl Decode<'_> for Version {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    let mut array = decoder.array()?;
    let version = match array.element::<u64>()? {
      0 => Self::Zero,
      version => return Err(DecodeError::UnsupportedVersion { version }),
    };
    array.finish()?;
    Ok(version)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Version::Zero);
  }

  #[test]
  fn unsupported() {
    let error = Version::decode_from_slice(&[0x01]).unwrap_err();
    assert_matches!(error, DecodeError::UnsupportedVersion { version: 1 });
  }
}
