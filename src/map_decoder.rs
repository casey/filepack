use super::*;

pub(crate) struct MapDecoder<'a, K> {
  decoder: Decoder<'a>,
  last: Option<K>,
}

impl<'a, K> MapDecoder<'a, K> {
  pub(crate) fn new(decoder: Decoder<'a>) -> Self {
    Self {
      decoder,
      last: None,
    }
  }
}

impl<K: Clone + Decode + Debug + PartialOrd> MapDecoder<'_, K> {
  pub(crate) fn finish(&mut self) -> Result<(), DecodeError> {
    ensure!(self.decoder.is_empty(), decode_error::UnconsumedEntries);
    Ok(())
  }

  pub(crate) fn key<V: Decode>(&mut self, key: K) -> Result<Option<V>, DecodeError> {
    self.key_with(key, V::decode)
  }

  pub(crate) fn key_with<V>(
    &mut self,
    key: K,
    decode: impl FnOnce(&mut Decoder) -> Result<V, DecodeError>,
  ) -> Result<Option<V>, DecodeError> {
    let Some((k, value)) = self.next_with(decode)? else {
      return Ok(None);
    };

    ensure!(k == key, decode_error::UnexpectedKey);

    Ok(Some(value))
  }

  pub(crate) fn next<V: Decode>(&mut self) -> Result<Option<(K, V)>, DecodeError> {
    self.next_with(V::decode)
  }

  pub(crate) fn next_with<V>(
    &mut self,
    decode: impl FnOnce(&mut Decoder) -> Result<V, DecodeError>,
  ) -> Result<Option<(K, V)>, DecodeError> {
    if self.decoder.is_empty() {
      return Ok(None);
    }

    let key = K::decode(&mut self.decoder)?;

    if let Some(last) = &self.last {
      ensure!(key > *last, decode_error::KeyOrder);
    }

    self.last = Some(key.clone());

    let value = decode(&mut self.decoder)?;

    Ok(Some((key, value)))
  }

  pub(crate) fn optional_key<V: Decode>(&mut self, key: K) -> Result<Option<V>, DecodeError>
  where
    K: Eq,
  {
    self.optional_key_with(key, V::decode)
  }

  pub(crate) fn optional_key_with<V>(
    &mut self,
    key: K,
    decode: impl FnOnce(&mut Decoder) -> Result<V, DecodeError>,
  ) -> Result<Option<V>, DecodeError>
  where
    K: Eq,
  {
    if self.decoder.is_empty() {
      return Ok(None);
    }

    let position = self.decoder.position();
    let next = K::decode(&mut self.decoder)?;

    if next != key {
      self.decoder.set_position(position);
      return Ok(None);
    }

    if let Some(last) = &self.last {
      ensure!(next > *last, decode_error::KeyOrder);
    }

    self.last = Some(next);

    Ok(Some(decode(&mut self.decoder)?))
  }

  pub(crate) fn required_key<V: Decode>(&mut self, key: K) -> Result<V, DecodeError>
  where
    K: Clone + Display,
  {
    self
      .key(key.clone())?
      .with_context(|| decode_error::MissingField {
        key: key.to_string(),
      })
  }

  pub(crate) fn required_key_with<V>(
    &mut self,
    key: K,
    decode: impl FnOnce(&mut Decoder) -> Result<V, DecodeError>,
  ) -> Result<V, DecodeError>
  where
    K: Clone + Display,
  {
    self
      .key_with(key.clone(), decode)?
      .with_context(|| decode_error::MissingField {
        key: key.to_string(),
      })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn decode_offset(decoder: &mut Decoder) -> Result<u64, DecodeError> {
    Ok(decoder.integer()? + 1)
  }

  #[test]
  fn key_mismatch() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x00]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.key::<u64>(0), Err(DecodeError::UnexpectedKey));
  }

  #[test]
  fn key_with() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.key_with(0, decode_offset), Ok(Some(43)));
    map.finish().unwrap();
  }

  #[test]
  fn malformed_entries() {
    #[track_caller]
    fn case(bytes: &[u8], expected: &str) {
      assert_eq!(
        BTreeMap::<u64, u64>::decode_from_slice(bytes)
          .unwrap_err()
          .to_string(),
        expected
      );
    }

    case(&[0x84, 0, 1, 0, 2], "map keys out of order");
    case(&[0x84, 1, 0, 0, 2], "map keys out of order");
    case(&[0], "truncated");
  }

  #[test]
  fn missing_field() {
    let mut decoder = Decoder::new(&[0x80]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.required_key::<u64>(0), Err(DecodeError::MissingField { key }) if key == "0");
  }

  #[test]
  fn next_with() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.next_with(decode_offset), Ok(Some((0, 43))));
    map.finish().unwrap();
  }

  #[test]
  fn optional_key_missing() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.optional_key::<u64>(0), Ok(None));
    map.next::<u64>().unwrap();
    map.finish().unwrap();
  }

  #[test]
  fn optional_key_present() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.optional_key::<u64>(0), Ok(Some(42)));
    map.finish().unwrap();
  }

  #[test]
  fn optional_key_with_missing() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.optional_key_with(0, decode_offset), Ok(None));
    map.next::<u64>().unwrap();
    map.finish().unwrap();
  }

  #[test]
  fn optional_key_with_present() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.optional_key_with(0, decode_offset), Ok(Some(43)));
    map.finish().unwrap();
  }

  #[test]
  fn optional_keys_preserve_position() {
    let mut decoder = Decoder::new(&[0x84, 1, 2, 3, 4]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_eq!(map.optional_key::<u64>(0).unwrap(), None);
    assert_eq!(map.optional_key::<u64>(1).unwrap(), Some(2));
    assert_eq!(map.optional_key::<u64>(2).unwrap(), None);
    assert_eq!(map.optional_key::<u64>(3).unwrap(), Some(4));
    assert_eq!(map.optional_key::<u64>(4).unwrap(), None);
    map.finish().unwrap();
    decoder.finish().unwrap();
  }

  #[test]
  fn out_of_order() {
    let mut decoder = Decoder::new(&[0x84, 0x02, 0x00, 0x01, 0x00]);
    let mut map = decoder.map::<u64>().unwrap();
    map.next::<u64>().unwrap();
    assert_matches!(map.next::<u64>(), Err(DecodeError::KeyOrder));
  }

  #[test]
  fn required_key_with() {
    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.required_key_with(0, decode_offset), Ok(43));
    map.finish().unwrap();
  }

  #[test]
  fn unconsumed_entries() {
    let mut decoder = Decoder::new(&[0x84, 0x00, 0x00, 0x01, 0x01]);
    let mut map = decoder.map::<u64>().unwrap();
    map.next::<u64>().unwrap();
    assert_matches!(map.finish(), Err(DecodeError::UnconsumedEntries));
  }
}
