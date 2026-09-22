use super::*;

pub trait MagicBytes: Encode {
  const MAGIC_BYTES: MagicByteArray;

  fn decode_magic_bytes(buffer: &[u8]) -> DecodeResult<Self>
  where
    Self: DecodeOwned,
  {
    Self::decode_magic_bytes_with_options(DecodeOptions::new(), buffer)
  }

  fn decode_magic_bytes_with_options(options: DecodeOptions, buffer: &[u8]) -> DecodeResult<Self>
  where
    Self: DecodeOwned,
  {
    Ok(WithMagicBytes::decode_from_slice_with_options(options, buffer)?.0)
  }

  fn encode_magic_bytes(&self) -> Vec<u8> {
    WithMagicBytes(self).encode_to_vec()
  }
}

impl<T: MagicBytes + ?Sized> MagicBytes for &T {
  const MAGIC_BYTES: MagicByteArray = T::MAGIC_BYTES;
}
