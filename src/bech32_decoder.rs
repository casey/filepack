use super::*;

pub(crate) struct Bech32Decoder<'a> {
  data: &'a [u8],
  i: usize,
  ty: Bech32Type,
}

impl<'a> Bech32Decoder<'a> {
  pub(crate) fn byte_array<const LEN: usize>(&mut self) -> Result<[u8; LEN], Bech32Error> {
    let mut array = [0; LEN];

    for (slot, byte) in array.iter_mut().zip(self.bytes(LEN)?) {
      *slot = byte;
    }

    Ok(array)
  }

  fn bytes(&mut self, len: usize) -> Result<impl Iterator<Item = u8>, Bech32Error> {
    let fe_len = (len * 8).div_ceil(5);

    let fes = self.fes(fe_len)?;

    if let Some((i, last)) = fes.clone().enumerate().last() {
      let padding_len = (i + 1) * 5 % 8;

      if padding_len > 4 {
        return Err(Bech32Error::Padding {
          source: PaddingError::TooMuch,
          ty: self.ty,
        });
      }

      if u64::from(last.to_u8().trailing_zeros()) < padding_len.into_u64() {
        return Err(Bech32Error::Padding {
          source: PaddingError::NonZero,
          ty: self.ty,
        });
      }
    }

    Ok(fes.fes_to_bytes())
  }

  pub(crate) fn done(self) -> Result<(), Bech32Error> {
    let excess = self.data.len() - self.i;

    ensure! {
      excess == 0,
      bech32_error::Overlong { excess, ty: self.ty },
    }

    Ok(())
  }

  pub(crate) fn fe(&mut self) -> Result<Fe32, Bech32Error> {
    let next = self
      .data
      .get(self.i)
      .map(|c| Fe32::from_char_unchecked(*c))
      .context(bech32_error::Truncated { ty: self.ty })?;

    self.i += 1;

    Ok(next)
  }

  pub(crate) fn fe_array<const LEN: usize>(&mut self) -> Result<[Fe32; LEN], Bech32Error> {
    let mut array = [Fe32::Q; LEN];

    for slot in &mut array {
      *slot = self.fe()?;
    }

    Ok(array)
  }

  fn fes(
    &mut self,
    len: usize,
  ) -> Result<impl Iterator<Item = Fe32> + Clone + use<'a>, Bech32Error> {
    let end = self.i + len;

    if end > self.data.len() {
      return Err(Bech32Error::Truncated { ty: self.ty });
    }

    let fes = &self.data[self.i..end];

    self.i = end;

    Ok(fes.iter().map(|c| Fe32::from_char_unchecked(*c)))
  }

  pub(crate) fn into_bytes(mut self) -> Result<Vec<u8>, Bech32Error> {
    let fes = self.data.len() - self.i;
    Ok(self.bytes(fes * 5 / 8)?.collect())
  }

  pub(crate) fn new(ty: Bech32Type, s: &'a str) -> Result<Self, Bech32Error> {
    let hrp_string =
      CheckedHrpstring::new::<bech32::Bech32m>(s).context(bech32_error::Decode { ty })?;

    let actual = hrp_string.hrp();

    ensure! {
      actual == *ty.hrp(),
      bech32_error::Hrp { ty, actual },
    }

    let mut fes = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

    let version = fes.next().context(bech32_error::Truncated { ty })?;

    ensure! {
      version == BECH32_VERSION,
      bech32_error::UnsupportedVersion { ty, version },
    }

    Ok(Self {
      data: hrp_string.data_part_ascii_no_checksum(),
      i: 1,
      ty,
    })
  }
}
