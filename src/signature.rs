use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub enum Error {
  #[snafu(display("signatures must be lowercase hex: `{signature}`"))]
  Case { signature: String },
  #[snafu(display("invalid signature hex: `{signature}`"))]
  Hex {
    signature: String,
    source: hex::FromHexError,
  },
  #[snafu(display("invalid signature byte length {length}: `{signature}`"))]
  Length {
    signature: String,
    length: usize,
    source: TryFromSliceError,
  },
}

#[derive(Clone, Copy, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub struct Signature(ed25519_dalek::Signature);

impl Signature {
  const LEN: usize = ed25519_dalek::Signature::BYTE_SIZE;
}

impl AsRef<ed25519_dalek::Signature> for Signature {
  fn as_ref(&self) -> &ed25519_dalek::Signature {
    &self.0
  }
}

impl From<ed25519_dalek::Signature> for Signature {
  fn from(inner: ed25519_dalek::Signature) -> Self {
    Self(inner)
  }
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for byte in self.0.to_bytes() {
      write!(f, "{byte:02x}")?;
    }
    Ok(())
  }
}

impl fmt::Debug for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.0, f)
  }
}

impl FromStr for Signature {
  type Err = Error;

  fn from_str(signature: &str) -> Result<Self, Self::Err> {
    let bytes = hex::decode(signature).context(HexError { signature })?;

    if !is_lowercase_hex(signature) {
      return Err(CaseError { signature }.build());
    }

    let array: [u8; Self::LEN] = bytes.as_slice().try_into().context(LengthError {
      signature,
      length: bytes.len(),
    })?;

    Ok(Self(ed25519_dalek::Signature::from_bytes(&array)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display_is_lowercase_hex() {
    let s = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
             0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3";
    assert_eq!(s.parse::<Signature>().unwrap().to_string(), s);
  }

  #[test]
  fn must_have_leading_zeros() {
    "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
     0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<Signature>()
      .unwrap();

    "f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
     0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3"
      .parse::<Signature>()
      .unwrap_err();
  }

  #[test]
  fn parse() {
    let signature = PrivateKey::generate().sign(Digest(Hash::bytes(b"hello")));
    assert_eq!(
      signature.to_string().parse::<Signature>().unwrap(),
      signature
    );
  }

  #[test]
  fn uppercase_is_forbidden() {
    let signature = "0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3\
                     0f6d444f09eb336d3cc94d66cc541fea0b70b36be291eb3ecf5b49113f34c8d3";
    signature.parse::<Signature>().unwrap();
    assert_eq!(
      signature
        .to_uppercase()
        .parse::<Signature>()
        .unwrap_err()
        .to_string(),
      "signatures must be lowercase hex: \
       `0F6D444F09EB336D3CC94D66CC541FEA0B70B36BE291EB3ECF5B49113F34C8D3\
        0F6D444F09EB336D3CC94D66CC541FEA0B70B36BE291EB3ECF5B49113F34C8D3`",
    );
  }
}
