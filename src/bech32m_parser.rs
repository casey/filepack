use super::*;

const FE32_BITS: usize = 5;

pub(crate) struct Bech32mDecoder<'a> {
  data: &'a [u8],
  i: usize,
  ty: Bech32mType,
}

impl<'a> Bech32mDecoder<'a> {
  pub(crate) fn new(ty: Bech32mType, s: &'a str) -> Result<Self, Bech32mError> {
    let hrp_string =
      CheckedHrpstring::new::<bech32::Bech32m>(s).context(bech32m_error::Decode { ty })?;

    let actual = hrp_string.hrp();

    ensure! {
      actual == *ty.hrp(),
      bech32m_error::Hrp { ty, actual },
    }

    let mut fes = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

    let version = fes.next().context(bech32m_error::VersionMissing { ty })?;

    ensure! {
      version == BECH32M_VERSION,
      bech32m_error::UnsupportedVersion { ty, version },
    }

    Ok(Self {
      data: hrp_string.data_part_ascii_no_checksum(),
      i: 1,
      ty,
    })
  }

  pub(crate) fn fe(&mut self) -> Result<Fe32, Bech32mError> {
    let next = self
      .data
      .get(self.i)
      .map(|c| Fe32::from_char_unchecked(*c))
      .context(bech32m_error::Truncated { ty: self.ty })?;

    self.i += 1;

    Ok(next)
  }

  pub(crate) fn done(self) -> Result<(), Bech32mError> {
    let excess = self.data.len() - self.i;

    ensure! {
      excess == 0,
      bech32m_error::Overlong { excess, ty: self.ty },
    }

    Ok(())
  }

  pub(crate) fn into_bytes(mut self) -> Result<Vec<u8>, Bech32mError> {
    let fes = self.data.len() - self.i;
    Ok(self.bytes(fes * 5 / 8)?.collect())
  }

  fn bytes(&mut self, len: usize) -> Result<impl Iterator<Item = u8>, Bech32mError> {
    let fe_len = (len * 8).div_ceil(FE32_BITS);

    let fes = self.fes(fe_len)?;

    if let Some((i, last)) = fes.clone().enumerate().last() {
      let padding_len = (i + 1) * 5 % 8;

      if padding_len > 4 {
        return Err(Bech32mError::Padding {
          source: PaddingError::TooMuch,
          ty: self.ty,
        });
      }

      if u64::from(last.to_u8().trailing_zeros()) < padding_len.into_u64() {
        return Err(Bech32mError::Padding {
          source: PaddingError::NonZero,
          ty: self.ty,
        });
      }
    }

    Ok(fes.fes_to_bytes())
  }

  fn fes(
    &mut self,
    len: usize,
  ) -> Result<impl Iterator<Item = Fe32> + Clone + use<'a>, Bech32mError> {
    let end = self.i + len;

    if end > self.data.len() {
      return Err(Bech32mError::Truncated { ty: self.ty });
    }

    let fes = &self.data[self.i..end];

    self.i = end;

    Ok(fes.iter().map(|c| Fe32::from_char_unchecked(*c)))
  }

  pub(crate) fn byte_array<const LEN: usize>(&mut self) -> Result<[u8; LEN], Bech32mError> {
    let mut array = [0; LEN];

    for (slot, byte) in array.iter_mut().zip(self.bytes(LEN)?) {
      *slot = byte;
    }

    Ok(array)
  }

  pub(crate) fn fe_array<const LEN: usize>(&mut self) -> Result<[Fe32; LEN], Bech32mError> {
    let mut array = [Fe32::Q; LEN];

    for slot in array.iter_mut() {
      *slot = self.fe()?;
    }

    Ok(array)
  }
}
