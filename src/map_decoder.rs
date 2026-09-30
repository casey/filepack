use super::*;

pub(crate) struct MapDecoder<'a, K> {
  decoder: Decoder<'a>,
  last: Option<K>,
}

impl<'a, K> MapDecoder<'a, K> {
  pub(crate) fn is_empty(&self) -> bool {
    self.decoder.is_empty()
  }

  pub(crate) fn new(decoder: Decoder<'a>) -> Self {
    Self {
      decoder,
      last: None,
    }
  }
}

impl<'a, K: Clone + Decode<'a> + Debug + PartialOrd> MapDecoder<'a, K> {
  pub(crate) fn finish(self) -> DecodeResult {
    ensure!(self.decoder.is_empty(), malformed_error::UnconsumedEntries);
    Ok(())
  }

  pub(crate) fn next<V: Decode<'a>>(&mut self) -> DecodeResult<Option<(K, V)>> {
    if self.decoder.is_empty() {
      return Ok(None);
    }

    let key = K::decode(&mut self.decoder)?;

    if let Some(last) = &self.last {
      ensure!(key > *last, malformed_error::KeyOrder);
    }

    self.last = Some(key.clone());

    let value = V::decode(&mut self.decoder)?;

    Ok(Some((key, value)))
  }

  pub(crate) fn optional_key<V: Decode<'a>>(&mut self, key: K) -> DecodeResult<Option<V>>
  where
    K: Eq,
  {
    if self.decoder.is_empty() {
      return Ok(None);
    }

    let mut decoder = self.decoder.clone();

    let next = K::decode(&mut decoder)?;

    if next != key {
      return Ok(None);
    }

    self.decoder = decoder;

    if let Some(last) = &self.last {
      ensure!(next > *last, malformed_error::KeyOrder);
    }

    self.last = Some(next);

    match V::decode(&mut self.decoder) {
      Ok(value) => Ok(Some(value)),
      Err(DecodeError::Unknown { strict: false, .. }) => Ok(None),
      Err(error) => Err(error),
    }
  }

  pub(crate) fn required_key<V: Decode<'a>>(&mut self, key: K) -> DecodeResult<V>
  where
    K: Clone + Display,
  {
    let Some((k, value)) = self.next()? else {
      return Err(
        malformed_error::MissingField {
          key: key.to_string(),
        }
        .build()
        .into(),
      );
    };

    ensure!(k == key, malformed_error::UnexpectedKey);

    Ok(value)
  }
}

impl MapDecoder<'_, u64> {
  pub(crate) fn decode_unknown(mut self) -> DecodeResult {
    while let Some((key, _value)) = self.next::<&[u8]>()? {
      ensure!(
        !self.decoder.strict(),
        malformed_error::UnknownField { key }
      );
    }
    Ok(())
  }

  pub(crate) fn version(&mut self, name: &'static str) -> DecodeResult {
    if self.decoder.peek() != Some(0) {
      return Ok(());
    }
    self.decoder.integer()?;
    let version = self.decoder.integer()?;
    Err(
      self
        .decoder
        .unknown(unknown_error::Version { name, version }.build()),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_unknown() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x2a]);
    decoder.map::<u64>().unwrap().decode_unknown().unwrap();

    let mut decoder = Decoder::new(&[0x82, 0x01, 0x2a]);
    assert_matches!(
      decoder.strict_map::<u64>().unwrap().decode_unknown(),
      Err(DecodeError::Malformed(MalformedError::UnknownField {
        key: 1
      })),
    );
  }

  #[test]
  fn key_mismatch() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x00]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(
      map.required_key::<u64>(0),
      Err(DecodeError::Malformed(MalformedError::UnexpectedKey))
    );
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
    assert_matches!(map.required_key::<u64>(0), Err(DecodeError::Malformed(MalformedError::MissingField { key })) if key == "0");
  }

  #[test]
  fn optional_key_malformed() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0xf8]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(
      map.optional_key::<Language>(1),
      Err(DecodeError::Malformed(MalformedError::Reserved {
        value: 0xf8
      })),
    );
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
  fn optional_key_strict() {
    let bytes = BTreeMap::from([(1u64, "xx")]).encode_to_vec();
    let mut decoder = Decoder::with_options(DecodeOptions::strict(), &bytes);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(
      map.optional_key::<Language>(1),
      Err(DecodeError::Unknown {
        source: UnknownError::Language {
          source: LanguageError::Code { code },
        },
        strict: true,
      }) if code == "xx",
    );
  }

  #[test]
  fn optional_key_unknown() {
    let bytes = BTreeMap::from([(1u64, "xx"), (2, "en")]).encode_to_vec();
    let mut decoder = Decoder::new(&bytes);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(map.optional_key::<Language>(1), Ok(None));
    assert_eq!(
      map.optional_key::<Language>(2).unwrap(),
      Some("en".parse().unwrap()),
    );
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
    assert_matches!(
      map.next::<u64>(),
      Err(DecodeError::Malformed(MalformedError::KeyOrder))
    );
  }

  #[test]
  fn unconsumed_entries() {
    let mut decoder = Decoder::new(&[0x84, 0x00, 0x00, 0x01, 0x01]);
    let mut map = decoder.map::<u64>().unwrap();
    map.next::<u64>().unwrap();
    assert_matches!(
      map.finish(),
      Err(DecodeError::Malformed(MalformedError::UnconsumedEntries))
    );
  }

  #[test]
  fn version() {
    let mut decoder = Decoder::new(&[0x82, 0x01, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    map.version("foo").unwrap();
    assert_matches!(map.next::<u64>(), Ok(Some((1, 42))));
    map.finish().unwrap();

    let mut decoder = Decoder::new(&[0x80]);
    decoder.map::<u64>().unwrap().version("foo").unwrap();

    let mut decoder = Decoder::new(&[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(
      map.version("foo"),
      Err(DecodeError::Unknown {
        source: UnknownError::Version {
          name: "foo",
          version: 42,
        },
        strict: false
      }),
    );

    let mut decoder = Decoder::with_options(DecodeOptions::strict(), &[0x82, 0x00, 0x2a]);
    let mut map = decoder.map::<u64>().unwrap();
    assert_matches!(
      map.version("foo"),
      Err(DecodeError::Unknown {
        source: UnknownError::Version {
          name: "foo",
          version: 42,
        },
        strict: true
      }),
    );
  }
}
