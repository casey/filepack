use super::*;

const VERSION: Fe32 = Fe32::A;

pub(crate) trait Bech32m<const PREFIX: usize, const BODY: usize> {
  const TYPE: Bech32mType;

  type Suffix: Bech32mSuffix;

  #[cfg(test)]
  fn decode_bech32m(s: &str) -> Result<Bech32mPayload<PREFIX, BODY, Self::Suffix>, Bech32mError> {
    let hrp_string = CheckedHrpstring::new::<bech32::Bech32m>(s)
      .context(bech32m_error::Decode { ty: Self::TYPE })?;

    let actual = hrp_string.hrp();

    ensure! {
      actual == *Self::TYPE.hrp(),
      bech32m_error::Hrp { ty: Self::TYPE, actual },
    }

    let mut fe32s = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

    let version = fe32s
      .next()
      .context(bech32m_error::VersionMissing { ty: Self::TYPE })?;

    ensure! {
      version == VERSION,
      bech32m_error::UnsupportedVersion { ty: Self::TYPE, version },
    }

    let mut prefix = [Fe32::Q; PREFIX];

    for (actual, fe32) in prefix.iter_mut().enumerate() {
      *fe32 = fe32s.next().context(bech32m_error::PrefixLength {
        ty: Self::TYPE,
        expected: PREFIX,
        actual,
      })?;
    }

    let mut body = [0; BODY];

    let mut bytes = fe32s.fes_to_bytes();

    for (actual, byte) in body.iter_mut().enumerate() {
      *byte = bytes.next().context(bech32m_error::BodyLength {
        actual,
        expected: BODY,
        ty: Self::TYPE,
      })?;
    }

    let suffix = Self::Suffix::from_bytes(Self::TYPE, bytes)?;

    Self::validate_padding(&hrp_string).context(bech32m_error::Padding { ty: Self::TYPE })?;

    Ok(Bech32mPayload {
      body,
      prefix,
      suffix,
    })
  }

  #[cfg(test)]
  fn encode_bech32m(
    f: &mut Formatter,
    payload: Bech32mPayload<PREFIX, BODY, &Self::Suffix>,
  ) -> fmt::Result {
    let Bech32mPayload {
      body,
      prefix,
      suffix,
    } = payload;

    let mut encoder = Bech32mEncoder::new(Self::TYPE);

    encoder.fes(&prefix);

    encoder.bytes(&body);

    encoder.bytes(suffix.as_bytes());

    write!(f, "{encoder}")?;

    Ok(())
  }

  fn validate_padding(hrp_string: &CheckedHrpstring) -> Result<(), PaddingError> {
    let mut fe32s = hrp_string.fe32_iter::<std::vec::IntoIter<u8>>();

    fe32s.next().unwrap();

    for _ in 0..PREFIX {
      fe32s.next().unwrap();
    }

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
    const TYPE: Bech32mType = Bech32mType::PublicKey;
    type Suffix = ();
  }

  impl Display for EmptyPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      Self::encode_bech32m(f, Bech32mPayload::from_body([]))
    }
  }

  struct LongPublicKey;

  impl Bech32m<0, 33> for LongPublicKey {
    const TYPE: Bech32mType = Bech32mType::PublicKey;
    type Suffix = ();
  }

  impl Display for LongPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      Self::encode_bech32m(f, Bech32mPayload::from_body([0; 33]))
    }
  }

  #[test]
  fn implementations() {
    fn case<const PREFIX: usize, const BODY: usize, T: Bech32m<PREFIX, BODY>>(hrp: &str, ty: &str) {
      use bech32::Checksum;

      let max = (bech32::Bech32m::CODE_LENGTH
        - T::TYPE.hrp().as_str().len()
        - 1
        - bech32::Bech32m::CHECKSUM_LENGTH
        - 1
        - PREFIX)
        * 5
        / 8;

      assert!(BODY <= max);

      assert_eq!(T::TYPE.hrp().as_str(), hrp);

      assert_eq!(T::TYPE.to_string(), ty);
    }

    case::<0, { Fingerprint::LEN }, Fingerprint>("package", "package fingerprint");
    case::<0, { PrivateKey::LEN }, PrivateKey>("private", "private key");
    case::<0, { PublicKey::LEN }, PublicKey>("public", "public key");
    case::<3, { Signature::LEN }, Signature>("signature", "signature");
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
      "expected bech32m public key to have 32 body bytes but found 0",
    );

    case(
      &LongPublicKey.to_string(),
      "expected bech32m public key to have 0 suffix bytes but found 1",
    );

    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();

    let bech32 =
      bech32::encode::<bech32::Bech32>(*PublicKey::TYPE.hrp(), public_key.inner().as_bytes())
        .unwrap();

    case(&bech32, "failed to decode bech32m public key");
  }

  #[test]
  fn no_version() {
    let mut s = String::new();
    for c in []
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(PublicKey::TYPE.hrp())
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
      .with_checksum::<bech32::Bech32m>(PublicKey::TYPE.hrp())
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    let Bech32mError::Padding { ty, source } = PublicKey::decode_bech32m(&bech32m).unwrap_err()
    else {
      panic!("expected padding error");
    };

    assert_eq!(ty, Bech32mType::PublicKey);

    assert_eq!(source, PaddingError::NonZero);
  }

  #[test]
  fn non_zero_padding_rejected_with_prefix() {
    struct PrefixedPublicKey;

    impl Bech32m<2, 32> for PrefixedPublicKey {
      const TYPE: Bech32mType = Bech32mType::PublicKey;
      type Suffix = ();
    }

    let bech32m = iter::repeat_n(Fe32::Q, 2)
      .chain(iter::repeat_n(Fe32::Q, 51))
      .chain(iter::once(Fe32::P))
      .with_checksum::<bech32::Bech32m>(PrefixedPublicKey::TYPE.hrp())
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    let Bech32mError::Padding { ty, source } =
      PrefixedPublicKey::decode_bech32m(&bech32m).unwrap_err()
    else {
      panic!("expected padding error");
    };

    assert_eq!(ty, Bech32mType::PublicKey);

    assert_eq!(source, PaddingError::NonZero);
  }

  #[test]
  fn prefix_length() {
    struct PrefixedType;

    impl Bech32m<2, 0> for PrefixedType {
      const TYPE: Bech32mType = Bech32mType::PublicKey;
      type Suffix = ();
    }

    let bech32m = []
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(Bech32mType::PublicKey.hrp())
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    assert_eq!(
      PrefixedType::decode_bech32m(&bech32m)
        .unwrap_err()
        .to_string(),
      "expected bech32m public key to have 2 prefix characters but found 0",
    );
  }

  #[test]
  fn round_trip() {
    fn case<T>(s: &str)
    where
      T: FromStr + ToString,
      <T as FromStr>::Err: fmt::Debug,
    {
      assert_eq!(s.parse::<T>().unwrap().to_string(), s);
    }

    case::<Fingerprint>(test::FINGERPRINT);
    case::<PublicKey>(test::PUBLIC_KEY);
    case::<Signature>(test::SIGNATURE);

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
  fn too_much_padding_rejected() {
    struct ShortPublicKey;

    impl Bech32m<0, 31> for ShortPublicKey {
      const TYPE: Bech32mType = Bech32mType::PublicKey;
      type Suffix = ();
    }

    let bech32m = iter::repeat_n(Fe32::Q, 51)
      .with_checksum::<bech32::Bech32m>(ShortPublicKey::TYPE.hrp())
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    let Bech32mError::Padding { ty, source } =
      ShortPublicKey::decode_bech32m(&bech32m).unwrap_err()
    else {
      panic!("expected padding error");
    };

    assert_eq!(ty, Bech32mType::PublicKey);

    assert_eq!(source, PaddingError::TooMuch);
  }

  #[test]
  fn too_much_padding_rejected_with_prefix() {
    struct PrefixedShortPublicKey;

    impl Bech32m<2, 31> for PrefixedShortPublicKey {
      const TYPE: Bech32mType = Bech32mType::PublicKey;
      type Suffix = ();
    }

    let bech32m = iter::repeat_n(Fe32::Q, 2)
      .chain(iter::repeat_n(Fe32::Q, 51))
      .with_checksum::<bech32::Bech32m>(PrefixedShortPublicKey::TYPE.hrp())
      .with_witness_version(VERSION)
      .chars()
      .collect::<String>();

    let Bech32mError::Padding { ty, source } =
      PrefixedShortPublicKey::decode_bech32m(&bech32m).unwrap_err()
    else {
      panic!("expected padding error");
    };

    assert_eq!(ty, Bech32mType::PublicKey);

    assert_eq!(source, PaddingError::TooMuch);
  }

  #[test]
  fn unsupported_version() {
    let bytes = [0u8; 32];
    let mut s = String::new();
    for c in bytes
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(PublicKey::TYPE.hrp())
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
