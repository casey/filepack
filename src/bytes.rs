use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Bytes(Vec<u8>);

impl Deref for Bytes {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Display for Bytes {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", hex::encode(&self.0))
  }
}

impl From<&Bytes> for Vec<u8> {
  fn from(bytes: &Bytes) -> Self {
    bytes.to_vec()
  }
}

impl From<&[u8]> for Bytes {
  fn from(bytes: &[u8]) -> Self {
    bytes.to_vec().into()
  }
}

impl From<Vec<u8>> for Bytes {
  fn from(bytes: Vec<u8>) -> Self {
    Self(bytes)
  }
}

impl FromStr for Bytes {
  type Err = BytesError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let digits = s
      .chars()
      .map(|digit| match digit {
        'a'..='f' => Ok(u8::try_from(digit).unwrap() - b'a' + 10),
        '0'..='9' => Ok(u8::try_from(digit).unwrap() - b'0'),
        _ => Err(BytesError::Digit { digit }),
      })
      .collect::<Result<Vec<u8>, BytesError>>()?;

    let (chunks, remainder) = digits.as_chunks::<2>();

    ensure! {
      remainder.is_empty(),
      bytes_error::OddLength { len: digits.len() },
    }

    Ok(Self(
      chunks.iter().map(|[high, low]| high << 4 | low).collect(),
    ))
  }
}

impl json::Json for Bytes {
  fn json_schema(registry: &mut json::Registry) -> String {
    json::Schema::Bytes {
      description: "Byte string, encoded as hexadecimal.".into(),
    }
    .register("Bytes", registry)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn errors() {
    assert_eq!(
      "g".parse::<Bytes>().unwrap_err(),
      BytesError::Digit { digit: 'g' },
    );

    assert_eq!(
      "a".parse::<Bytes>().unwrap_err(),
      BytesError::OddLength { len: 1 },
    );
  }

  #[test]
  fn round_trip() {
    let bytes = Bytes::from(b"foo".to_vec());

    let json = serde_json::to_string(&bytes).unwrap();

    assert_eq!(json, r#""666f6f""#);

    assert_eq!(serde_json::from_str::<Bytes>(&json).unwrap(), bytes);
  }
}
