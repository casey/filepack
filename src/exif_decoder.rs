use super::*;

pub(crate) struct ExifDecoder<'a> {
  pub(crate) big_endian: bool,
  pub(crate) data: &'a [u8],
}

impl ExifDecoder<'_> {
  fn array<const N: usize>(&self, offset: usize) -> Result<[u8; N], ExifError> {
    Ok(
      self
        .data
        .get(offset..offset + N)
        .context(exif_error::Truncated)?
        .try_into()
        .unwrap(),
    )
  }

  pub(crate) fn u16(&self, offset: usize) -> Result<u16, ExifError> {
    let bytes = self.array(offset)?;

    Ok(if self.big_endian {
      u16::from_be_bytes(bytes)
    } else {
      u16::from_le_bytes(bytes)
    })
  }

  pub(crate) fn u32(&self, offset: usize) -> Result<u32, ExifError> {
    let bytes = self.array(offset)?;

    Ok(if self.big_endian {
      u32::from_be_bytes(bytes)
    } else {
      u32::from_le_bytes(bytes)
    })
  }
}
