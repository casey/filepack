use super::*;

pub trait Decode: Sized {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError>;

  fn decode_from_slice(buffer: &[u8]) -> Result<Self, DecodeError> {
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

impl Decode for bool {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.boolean()
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

impl<T: Decode> Decode for Vec<T> {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut array = decoder.array()?;

    let mut vec = Vec::new();
    while let Some(item) = array.next::<T>()? {
      vec.push(item);
    }

    array.finish()?;

    Ok(vec)
  }
}

impl Decode for i32 {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder
      .signed_integer()?
      .try_into()
      .context(decode_error::IntegerRange)
  }
}

impl Decode for i64 {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.signed_integer()
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

impl<const N: usize> Decode for [u8; N] {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.byte_array()
  }
}

impl<const N: usize, const M: usize> Decode for [[u8; N]; M] {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
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
}
