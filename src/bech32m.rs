use super::*;

const VERSION: Fe32 = Fe32::A;

pub(crate) trait Bech32m<const LEN: usize> {
  const HRP: Hrp;
  const TYPE: &'static str;

  fn decode_bech32m(s: &str) -> Result<[u8; LEN], Bech32mError> {
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

    let mut bytes = fe32s.fes_to_bytes();

    let mut array = [0; LEN];

    let mut actual = 0;
    for byte in &mut array {
      *byte = bytes.next().context(bech32m_error::Length {
        actual,
        expected: LEN,
      })?;
      actual += 1;
    }

    actual += bytes.count();

    ensure! {
      actual == LEN,
      bech32m_error::Length {
        actual,
        expected: LEN,
      },
    }

    Ok(array)
  }

  fn encode_bech32m(f: &mut Formatter, bytes: [u8; LEN]) -> fmt::Result {
    let chars = bytes
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(&Self::HRP)
      .with_witness_version(VERSION)
      .chars();

    for c in chars {
      f.write_char(c)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct EmptyPublicKey;

  impl Bech32m<0> for EmptyPublicKey {
    const HRP: Hrp = Hrp::parse_unchecked("public");
    const TYPE: &'static str = "public key";
  }

  impl Display for EmptyPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      Self::encode_bech32m(f, [])
    }
  }

  struct LongPublicKey;

  impl Bech32m<33> for LongPublicKey {
    const HRP: Hrp = Hrp::parse_unchecked("public");
    const TYPE: &'static str = "public key";
  }

  impl Display for LongPublicKey {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      Self::encode_bech32m(f, [0; 33])
    }
  }

  #[test]
  fn implementations() {
    fn case<const LEN: usize, T: Bech32m<LEN>>(hrp: &str, ty: &str) {
      use bech32::Checksum;

      let max = (bech32::Bech32m::CODE_LENGTH
        - T::HRP.as_str().len()
        - 1
        - bech32::Bech32m::CHECKSUM_LENGTH
        - 1)
        * 5
        / 8;

      assert!(LEN <= max);

      assert_eq!(T::HRP.as_str(), hrp);

      assert_eq!(T::TYPE, ty);
    }

    case::<{ Fingerprint::LEN }, Fingerprint>("package", "package fingerprint");
    case::<{ PrivateKey::LEN }, PrivateKey>("private", "private key");
    case::<{ PublicKey::LEN }, PublicKey>("public", "public key");
    case::<{ Signature::LEN }, Signature>("signature", "signature");
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

    case(&EmptyPublicKey.to_string(), "expected 32 bytes but found 0");

    case(&LongPublicKey.to_string(), "expected 32 bytes but found 33");

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

  #[test]
  fn non_zero_padding_rejected() {
    let bytes = [0u8; 32];
    let mut fe32s = bytes.iter().copied().bytes_to_fes().collect::<Vec<_>>();

    let last = fe32s.last_mut().unwrap();
    *last = Fe32::try_from(last.to_u8() | 0x01).unwrap();

    let mut s = String::new();
    for c in fe32s
      .into_iter()
      .with_checksum::<bech32::Bech32m>(&PublicKey::HRP)
      .with_witness_version(VERSION)
      .chars()
    {
      s.write_char(c).unwrap();
    }

    assert_eq!(
      PublicKey::decode_bech32m(&s).unwrap_err().to_string(),
      "bech32m public key has invalid padding",
    );
  }
}
