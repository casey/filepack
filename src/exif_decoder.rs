use super::*;

pub(crate) struct ExifDecoder<'a> {
  pub(crate) big_endian: bool,
  pub(crate) data: &'a [u8],
}

impl ExifDecoder<'_> {
  pub(crate) fn u16(&self, offset: usize) -> Result<u16, ExifError> {
    let bytes = self
      .data
      .get(offset..offset + 2)
      .context(exif_error::Truncated)?
      .try_into()
      .unwrap();

    Ok(if self.big_endian {
      u16::from_be_bytes(bytes)
    } else {
      u16::from_le_bytes(bytes)
    })
  }

  pub(crate) fn u32(&self, offset: usize) -> Result<u32, ExifError> {
    let bytes = self
      .data
      .get(offset..offset + 4)
      .context(exif_error::Truncated)?
      .try_into()
      .unwrap();

    Ok(if self.big_endian {
      u32::from_be_bytes(bytes)
    } else {
      u32::from_le_bytes(bytes)
    })
  }
}
