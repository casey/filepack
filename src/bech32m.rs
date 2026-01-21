use super::*;

const VERSION: Fe32 = Fe32::A;

type Bytes<'a> = FesToBytes<AsciiToFe32Iter<'a>>;

pub(crate) trait Suffix: Sized {
  fn from_bytes(ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError>;
}

impl Suffix for () {
  fn from_bytes(ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError> {
    let actual = bytes.count();

    ensure! {
      actual == 0,
      bech32m_error::SuffixLength {
        actual,
        expected: 0usize,
        ty,
      },
    }

    Ok(())
  }
}

impl Suffix for Vec<u8> {
  fn from_bytes(_ty: &'static str, bytes: Bytes) -> Result<Self, Bech32mError> {
    Ok(bytes.collect())
  }
}

pub(crate) trait Bech32m<const PREFIX: usize, const DATA: usize> {
  const HRP: Hrp;
  const TYPE: &'static str;

  type Suffix: Suffix;

  fn decode_bech32m(s: &str) -> Result<([Fe32; PREFIX], [u8; DATA], Self::Suffix), Bech32mError> {
    let hrp_string = CheckedHrpstring::new::<bech32::Bech32m>(s)
      .context(bech32m_error::Decode { ty: Self::TYPE })?;

    let actual = hrp_string.hrp();

    ensure! {
      actual == Self::HRP,
      bech32m_error::Hrp { expected: Self::HRP, actual },
    }

    let mut fe32s = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

    let version = fe32s
      .next()
      .context(bech32m_error::VersionMissing { ty: Self::TYPE })?;

    ensure! {
      version == VERSION,
      bech32m_error::UnsupportedVersion { ty: Self::TYPE, version },
    }

    Self::validate_padding(&hrp_string).context(bech32m_error::Padding { ty: Self::TYPE })?;

    let mut prefix = [Fe32::Q; PREFIX];

    for (actual, fe32) in prefix.iter_mut().enumerate() {
      *fe32 = fe32s.next().context(bech32m_error::PrefixLength {
        ty: Self::TYPE,
        expected: PREFIX,
        actual,
      })?;
    }

    let mut data = [0; DATA];

    let mut bytes = fe32s.fes_to_bytes();

    {
      let mut actual = 0usize;
      for byte in &mut data {
        *byte = bytes.next().context(bech32m_error::DataLength {
          actual,
          expected: DATA,
          ty: Self::TYPE,
        })?;
        actual += 1;
      }
    }

    let suffix = Self::Suffix::from_bytes(Self::TYPE, bytes)?;

    Ok((prefix, data, suffix))
  }

  fn encode_bech32m(f: &mut Formatter, prefix: [Fe32; PREFIX], data: [u8; DATA]) -> fmt::Result {
    let chars = prefix
      .into_iter()
      .chain(data.iter().copied().bytes_to_fes())
      .with_checksum::<bech32::Bech32m>(&Self::HRP)
      .with_witness_version(VERSION)
      .chars();

    for c in chars {
      f.write_char(c)?;
    }

    Ok(())
  }

  fn validate_padding(hrp_string: &CheckedHrpstring) -> Result<(), PaddingError> {
    let mut fe32s = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

    fe32s.next().unwrap();

    let Some((i, last)) = fe32s.enumerate().last() else {
      return Ok(());
    };

    let padding_len = (i + 1) * 5 % 8;

    if padding_len > 4 {
      return Err(PaddingError::TooMuch);
    }

    if u64::from(last.to_u8().trailing_zeros()) < padding_len.into_u64() {
      Err(PaddingError::NonZero)
    } else {
      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct EmptyPublicKey;

  impl Bech32m<0, 0> for EmptyPublicKey {
    const HRP: Hrp = Hrp::parse_unchecked("public");
    const TYPE: &'static str = "public key";
    type Suffix = ();
  }

  impl Display for EmptyPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      Self::encode_bech32m(f, [], [])
    }
  }

  struct LongPublicKey;

  impl Bech32m<0, 33> for LongPublicKey {
    const HRP: Hrp = Hrp::parse_unchecked("public");
    const TYPE: &'static str = "public key";
    type Suffix = ();
  }

  impl Display for LongPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      Self::encode_bech32m(f, [], [0; 33])
    }
  }

  #[test]
  fn implementations() {
    fn case<const PREFIX: usize, const DATA: usize, T: Bech32m<PREFIX, DATA>>(hrp: &str, ty: &str) {
      use bech32::Checksum;

      let max = (bech32::Bech32m::CODE_LENGTH
        - T::HRP.as_str().len()
        - 1
        - bech32::Bech32m::CHECKSUM_LENGTH
        - 1
        - PREFIX)
        * 5
        / 8;

      assert!(DATA <= max);

      assert_eq!(T::HRP.as_str(), hrp);

      assert_eq!(T::TYPE, ty);
    }

    case::<0, { Fingerprint::LEN }, Fingerprint>("package", "package fingerprint");
    case::<0, { PrivateKey::LEN }, PrivateKey>("private", "private key");
    case::<0, { PublicKey::LEN }, PublicKey>("public", "public key");
    case::<1, { Signature::LEN }, Signature>("signature", "signature");
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(
        PublicKey::decode_bech32m(s).unwrap_err().to_string(),
        expected,
      );
    }

    case("foo", "failed to decode bech32m public key");

    case(
      test::PRIVATE_KEY,
      "expected bech32m human-readable part `public1...` but found `private1...`",
    );

    case(
      &EmptyPublicKey.to_string(),
      "expected bech32m public key to have 32 data bytes but found 0",
    );

    case(
      &LongPublicKey.to_string(),
      "expected bech32m public key to have 0 suffix bytes but found 1",
    );

    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();

    let bech32 =
      bech32::encode::<bech32::Bech32>(PublicKey::HRP, public_key.inner().as_bytes()).unwrap();

    case(&bech32, "failed to decode bech32m public key");
  }

  #[test]
  fn no_version() {
    let mut s = String::new();
    for c in []
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(&PublicKey::HRP)
      .chars()
    {
      s.write_char(c).unwrap();
    }

    assert_eq!(
      PublicKey::decode_bech32m(&s).unwrap_err().to_string(),
      "bech32m public key missing version character",
    );
  }

  #[test]
  fn non_zero_padding_rejected() {
    let bech32m = iter::repeat_n(Fe32::Q, 51)
      .chain(iter::once(Fe32::P))
      .with_checksum::<bech32::Bech32m>(&PublicKey::HRP)
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    assert_eq!(
      PublicKey::decode_bech32m(&bech32m).unwrap_err().to_string(),
      "bech32m public key has invalid padding",
    );
  }

  #[test]
  fn prefix_length() {
    struct PrefixedType;

    impl Bech32m<2, 0> for PrefixedType {
      const HRP: Hrp = Hrp::parse_unchecked("test");
      const TYPE: &'static str = "test";
      type Suffix = ();
    }

    let bech32m = []
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(&PrefixedType::HRP)
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    assert_eq!(
      PrefixedType::decode_bech32m(&bech32m)
        .unwrap_err()
        .to_string(),
      "expected bech32m test to have 2 prefix characters but found 0",
    );
  }

  #[test]
  fn unsupported_version() {
    let bytes = [0u8; 32];
    let mut s = String::new();
    for c in bytes
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(&PublicKey::HRP)
      .with_witness_version(Fe32::P)
      .chars()
    {
      s.write_char(c).unwrap();
    }

    assert_eq!(
      PublicKey::decode_bech32m(&s).unwrap_err().to_string(),
      "bech32m public key version `p` is not supported",
    );
  }
}
