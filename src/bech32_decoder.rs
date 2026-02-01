use super::*;

#[derive(Debug)]
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

      if u64::from(last.to_u8().trailing_zeros()) < padding_len.into_u64() {
        return Err(Bech32Error::Padding { ty: self.ty });
      }
    }

    Ok(fes.fes_to_bytes())
  }

  pub(crate) fn decode_byte_array<const LEN: usize>(
    ty: Bech32Type,
    s: &'a str,
  ) -> Result<[u8; LEN], Bech32Error> {
    let mut decoder = Self::new(ty, s)?;
    let array = decoder.byte_array()?;
    decoder.done()?;
    Ok(array)
  }

  pub(crate) fn done(self) -> Result<(), Bech32Error> {
    let excess = self.data.len() - self.i;

    ensure! {
      excess == 0,
      bech32_error::Overlong { excess, ty: self.ty },
    }

    Ok(())
  }

  pub(crate) fn fe(&mut self) -> Option<Fe32> {
    let fe = Fe32::from_char_unchecked(*self.data.get(self.i)?);
    self.i += 1;
    Some(fe)
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

#[cfg(test)]
mod tests {
  use super::*;

  fn checksum(s: &str) -> String {
    let checked_hrpstring = CheckedHrpstring::new::<bech32::NoChecksum>(s).unwrap();
    checked_hrpstring
      .fe32_iter::<std::vec::IntoIter<u8>>()
      .with_checksum::<bech32::Bech32m>(&checked_hrpstring.hrp())
      .chars()
      .collect()
  }

  #[test]
  fn errors() {
    #[track_caller]
    fn case(s: &str, err: &str) {
      assert_eq!(
        Bech32Decoder::decode_byte_array::<1>(Bech32Type::PublicKey, &checksum(s))
          .unwrap_err()
          .to_string(),
        err,
      );
    }

    case(
      "foo1",
      "expected bech32 human-readable part `public1...` but found `foo1...`",
    );

    case("public1c", "bech32 public key version `c` not supported");

    case("public1a", "bech32 public key truncated");

    case("public1aqqq", "bech32 public key overlong by 1 character");

    case("public1aql", "bech32 public key has nonzero padding");
  }

  #[test]
  fn length() {
    #[track_caller]
    fn case(s: &str) {
      use bech32::Checksum;
      assert!(s.len() <= bech32::Bech32m::CODE_LENGTH);
    }

    case(test::FINGERPRINT);
    case(test::PUBLIC_KEY);
    case(test::PRIVATE_KEY);
    case(test::SIGNATURE);
  }

  #[test]
  fn private_key_round_trip() {
    assert_eq!(
      test::PRIVATE_KEY
        .parse::<PrivateKey>()
        .unwrap()
        .display_secret()
        .to_string(),
      test::PRIVATE_KEY,
    );
  }

  #[test]
  fn round_trip() {
    #[track_caller]
    fn case<T: FromStr + ToString>(s: &str)
    where
      T::Err: fmt::Debug,
    {
      assert_eq!(s.parse::<T>().unwrap().to_string(), s);
    }

    case::<Fingerprint>(test::FINGERPRINT);
    case::<PublicKey>(test::PUBLIC_KEY);
    case::<Signature>(test::SIGNATURE);
  }
}
