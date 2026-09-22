use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct WithMagicBytes<T>(pub T);

impl<T: Encode + MagicBytes> Encode for WithMagicBytes<T> {
  fn encode(&self, encoder: &mut Encoder) {
    self.0.encode(encoder);
    encoder.bytes(&T::MAGIC_BYTES);
  }
}

impl<T: DecodeOwned + MagicBytes> Decode<'_> for WithMagicBytes<T> {
  fn decode(decoder: &mut Decoder) -> DecodeResult<Self> {
    let actual = decoder.bytes()?;
    ensure!(
      actual == T::MAGIC_BYTES,
      decode_error::MagicBytes {
        actual: &actual[..actual.len().min(T::MAGIC_BYTES.len() + 1)],
        expected: T::MAGIC_BYTES,
      },
    );
    Ok(Self(T::decode(decoder)?))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Decode, Encode, PartialEq)]
  struct Foo {
    #[n(0)]
    bar: u64,
  }

  impl MagicBytes for Foo {
    const MAGIC_BYTES: MagicByteArray = *b"foo\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
  }

  #[test]
  fn decode_error_after_magic_bytes() {
    let mut encoder = Encoder::new();
    BTreeMap::<u64, u64>::new().encode(&mut encoder);
    encoder.bytes(&Foo::MAGIC_BYTES);
    assert_eq!(
      Foo::decode_magic_bytes(&encoder.finish())
        .unwrap_err()
        .to_string(),
      "missing required field: 0",
    );
  }

  #[test]
  fn encoding() {
    assert_deco(
      WithMagicBytes(Foo { bar: 1 }),
      "92666f6f000000000000000000000000000000820001",
    );
  }

  #[test]
  fn magic_bytes() {
    #[track_caller]
    fn case<T: MagicBytes>() {
      let magic_bytes = T::MAGIC_BYTES.encode_to_vec();
      assert!(str::from_utf8(&magic_bytes).is_err());
      assert!(magic_bytes.starts_with(&[0x92]));
      assert!(magic_bytes.ends_with(&[0]));
      assert!(magic_bytes[1..].starts_with(b"filepack-"));
    }
    case::<Archive>();
    case::<Metadata>();
  }

  #[test]
  fn mismatch() {
    #[track_caller]
    fn case(magic: &[u8], found: &str) {
      let mut encoder = Encoder::new();
      Foo { bar: 1 }.encode(&mut encoder);
      encoder.bytes(magic);
      assert_eq!(
        Foo::decode_magic_bytes(&encoder.finish())
          .unwrap_err()
          .to_string(),
        format!(
          "unexpected magic bytes, expected \
          `foo\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00` \
          but found `{found}`"
        ),
      );
    }

    case(b"bar", "bar");
    case(b"barbarbarbarbarbarbar", "barbarbarbarbarbar…");
  }
}
