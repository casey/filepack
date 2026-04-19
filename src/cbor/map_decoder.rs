use super::*;

pub(crate) struct MapDecoder<'a, K> {
  decoder: &'a mut Decoder,
  last: Option<K>,
  remaining: u64,
}

impl<'a, K> MapDecoder<'a, K> {
  pub(crate) fn new(decoder: &'a mut Decoder, len: u64) -> Self {
    Self {
      decoder,
      last: None,
      remaining: len,
    }
  }
}

impl<K: Clone + Decode + Debug + PartialOrd> MapDecoder<'_, K> {
  pub(crate) fn finish(&mut self) -> Result<(), DecodeError> {
    ensure!(self.remaining == 0, decode_error::UnconsumedEntries);
    Ok(())
  }

  pub(crate) fn key<V: Decode>(&mut self, key: K) -> Result<Option<V>, DecodeError> {
    let Some((k, value)) = self.next()? else {
      return Ok(None);
    };

    ensure!(k == key, decode_error::UnexpectedKey);

    Ok(Some(value))
  }

  pub(crate) fn next<V: Decode>(&mut self) -> Result<Option<(K, V)>, DecodeError> {
    if self.remaining == 0 {
      return Ok(None);
    }

    self.remaining -= 1;

    let key = K::decode(self.decoder)?;

    if let Some(last) = &self.last {
      ensure!(key > *last, decode_error::KeyOrder);
    }

    self.last = Some(key.clone());

    let value = V::decode(self.decoder)?;

    Ok(Some((key, value)))
  }

  pub(crate) fn optional_key<V: Decode>(&mut self, key: K) -> Result<Option<V>, DecodeError>
  where
    K: Eq,
  {
    if self.remaining == 0 {
      return Ok(None);
    }

    self.decoder.push_position();
    let next = K::decode(self.decoder)?;

    if next != key {
      self.decoder.pop_position();
      return Ok(None);
    }

    if let Some(last) = &self.last {
      ensure!(next > *last, decode_error::KeyOrder);
    }

    self.remaining -= 1;
    self.last = Some(next);

    Ok(Some(V::decode(self.decoder)?))
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
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn key_mismatch() {
    let mut decoder = Decoder::new(vec![0xA1, 0x01, 0x00]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_eq!(map.key::<u64>(0), Err(DecodeError::UnexpectedKey));
  }

  #[test]
  fn out_of_order() {
    let mut decoder = Decoder::new(vec![0xA2, 0x02, 0x00, 0x01, 0x00]);
    let mut map = decoder.map::<u64>().unwrap();
    map.next::<u64>().unwrap();
    assert_eq!(map.next::<u64>(), Err(DecodeError::KeyOrder));
  }

  #[test]
  fn unconsumed_entries() {
    let mut decoder = Decoder::new(vec![0xA2, 0x00, 0x00, 0x01, 0x01]);
    let mut map = decoder.map::<u64>().unwrap();
    map.next::<u64>().unwrap();
    assert_eq!(map.finish(), Err(DecodeError::UnconsumedEntries));
  }
}
