use super::*;

pub(crate) trait Decode: Sized {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError>;
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

impl Decode for String {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    Ok(decoder.text()?.to_owned())
  }
}

impl Decode for u8 {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder
      .integer()?
      .try_into()
      .context(decode_error::IntegerOutOfRange)
  }
}

impl Decode for u64 {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    decoder.integer()
  }
}
