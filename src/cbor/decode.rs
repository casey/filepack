use super::*;

pub(crate) trait Decode: Sized {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError>;

  fn decode_from_vec(buffer: Vec<u8>) -> Result<Self, DecodeError> {
    let mut decoder = Decoder::new(buffer);
    let value = Self::decode(&mut decoder)?;
    decoder.finish()?;
    Ok(value)
  }
}

impl<K, V> Decode for BTreeMap<K, V>
where
  K: Clone + Decode + Debug + Ord + PartialOrd,
  V: Decode,
{
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.map::<K>()?;

    let mut map = BTreeMap::new();
    while let Some((key, value)) = decoder.next::<V>()? {
      map.insert(key, value);
    }

    decoder.finish()?;

    Ok(map)
  }
}

impl Decode for Url {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.text()?.parse::<Url>().context(decode_error::Url)
  }
}

impl Decode for String {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    Ok(decoder.text()?.to_owned())
  }
}

impl Decode for Vec<u8> {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    Ok(decoder.bytes()?.to_vec())
  }
}

impl Decode for u8 {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder
      .integer()?
      .try_into()
      .context(decode_error::IntegerRange)
  }
}

impl Decode for u64 {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.integer()
  }
}

impl Decode for usize {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder
      .integer()?
      .try_into()
      .context(decode_error::IntegerRange)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn url() {
    assert_encoding(
      "http://example.com/".parse::<Url>().unwrap(),
      &"http://example.com/".encode_to_vec(),
    );
  }
}
