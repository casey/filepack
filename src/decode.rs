use super::*;

pub trait Decode<'a>: Sized {
  fn decode(decoder: &mut Decoder<'a>) -> DecodeResult<Self>;

  fn decode_from_slice(buffer: &'a [u8]) -> DecodeResult<Self> {
    Self::decode_from_slice_with_options(DecodeOptions::new(), buffer)
  }

  fn decode_from_slice_with_options(
    options: DecodeOptions,
    buffer: &'a [u8],
  ) -> DecodeResult<Self> {
    let mut decoder = Decoder::with_options(options, buffer);
    let value = Self::decode(&mut decoder)?;
    decoder.finish()?;
    Ok(value)
  }

  fn decode_optional(decoder: &mut Decoder<'a>) -> DecodeResult<Option<Self>> {
    Self::decode(decoder).map(Some)
  }
}

impl<'a, K, V> Decode<'a> for BTreeMap<K, V>
where
  K: Clone + Decode<'a> + Debug + Ord + PartialOrd,
  V: Decode<'a>,
{
  fn decode(decoder: &mut Decoder<'a>) -> DecodeResult<Self> {
    let mut decoder = decoder.map::<K>()?;

    let mut map = BTreeMap::new();
    while let Some((key, value)) = decoder.next::<V>()? {
      map.insert(key, value);
    }

    decoder.finish()?;

    Ok(map)
  }
}

impl Decode<'_> for bool {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    decoder.boolean()
  }
}

impl Decode<'_> for String {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    Ok(decoder.text()?.to_owned())
  }
}

impl Decode<'_> for Vec<u8> {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    Ok(decoder.bytes()?.to_vec())
  }
}

impl<'a, T: Decode<'a>> Decode<'a> for Vec<T> {
  fn decode(decoder: &mut Decoder<'a>) -> DecodeResult<Self> {
    let mut array = decoder.array()?;

    let mut vec = Vec::new();
    while let Some(item) = array.next::<T>()? {
      vec.push(item);
    }

    array.finish()?;

    Ok(vec)
  }
}

impl Decode<'_> for i32 {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    decoder
      .signed_integer()?
      .try_into()
      .context(decode_error::IntegerRange)
  }
}

impl Decode<'_> for i64 {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    decoder.signed_integer()
  }
}

impl Decode<'_> for u64 {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    decoder.integer()
  }
}

impl Decode<'_> for usize {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    decoder
      .integer()?
      .try_into()
      .context(decode_error::IntegerRange)
  }
}

impl<'a> Decode<'a> for &'a [u8] {
  fn decode(decoder: &mut Decoder<'a>) -> DecodeResult<Self> {
    decoder.bytes()
  }
}

impl<'a> Decode<'a> for &'a str {
  fn decode(decoder: &mut Decoder<'a>) -> DecodeResult<Self> {
    decoder.text()
  }
}

impl<const N: usize> Decode<'_> for [u8; N] {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    decoder.byte_array()
  }
}

impl<const N: usize, const M: usize> Decode<'_> for [[u8; N]; M] {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    let bytes = decoder.bytes()?;

    ensure! {
      bytes.len() == M * N,
      decode_error::ArrayLength {
        actual: bytes.len(),
        expected: M * N,
      }
    }

    let (chunks, remainder) = bytes.as_chunks::<N>();
    assert!(remainder.is_empty());

    Ok(chunks.try_into().unwrap())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn borrowed_bytes() {
    assert_eq!(
      <&[u8]>::decode_from_slice(&[0x82, 0x01, 0x02]).unwrap(),
      [0x01, 0x02],
    );
  }

  #[test]
  fn borrowed_str() {
    assert_eq!(
      <&str>::decode_from_slice(&[0x83, 0x66, 0x6f, 0x6f]).unwrap(),
      "foo",
    );
  }

  #[test]
  fn decode_from_slice_errors_on_trailing_bytes() {
    assert_matches!(
      u64::decode_from_slice(&[0x00, 0x00]),
      Err(DecodeError::TrailingBytes),
    );
  }

  #[test]
  fn nested_byte_array_length_mismatch() {
    assert_matches!(
      <[[u8; 2]; 2]>::decode_from_slice(&[0x83, 1, 2, 3]),
      Err(DecodeError::ArrayLength {
        actual: 3,
        expected: 4,
      }),
    );
  }

  #[test]
  fn unknown_fields() {
    #[derive(Debug, Decode, Encode, PartialEq)]
    struct Foo {
      #[n(0)]
      foo: u64,
    }

    let value = Foo { foo: 1 };

    let bytes = with_unknown_field(&value);

    assert_eq!(Foo::decode_from_slice(&bytes).unwrap(), value);

    assert_matches!(
      Foo::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
      Err(DecodeError::UnknownField { key: u64::MAX }),
    );
  }

  #[test]
  fn unknown_fields_array() {
    #[derive(Debug, Decode, Encode, PartialEq)]
    struct Foo {
      #[n(0)]
      foo: u64,
    }

    let foo = vec![Foo { foo: 1 }];

    assert_eq!(
      Vec::<Foo>::decode_from_slice_with_options(DecodeOptions::strict(), &foo.encode_to_vec())
        .unwrap(),
      foo,
    );

    let bytes = vec![BTreeMap::from([(0u64, 1u64), (1, 2)])].encode_to_vec();

    assert_eq!(Vec::<Foo>::decode_from_slice(&bytes).unwrap(), foo);

    assert_matches!(
      Vec::<Foo>::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
      Err(DecodeError::UnknownField { key: 1 }),
    );
  }

  #[test]
  fn unknown_fields_nested() {
    #[derive(Debug, Decode, Encode, PartialEq)]
    struct Foo {
      #[n(0)]
      foo: u64,
    }

    #[derive(Debug, Decode, Encode, PartialEq)]
    struct Bar {
      #[n(0)]
      bar: Foo,
    }

    let bar = Bar {
      bar: Foo { foo: 1 },
    };

    assert_eq!(
      Bar::decode_from_slice_with_options(DecodeOptions::strict(), &bar.encode_to_vec()).unwrap(),
      bar,
    );

    let bytes = BTreeMap::from([(0u64, BTreeMap::from([(0u64, 1u64), (1, 2)]))]).encode_to_vec();

    assert_eq!(Bar::decode_from_slice(&bytes).unwrap(), bar);

    assert_matches!(
      Bar::decode_from_slice_with_options(DecodeOptions::strict(), &bytes),
      Err(DecodeError::UnknownField { key: 1 }),
    );
  }
}
