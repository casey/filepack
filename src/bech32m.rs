use {super::*, bech32::primitives::decode::CheckedHrpstring};

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

    let mut bytes = hrp_string.byte_iter();

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
    bech32::encode_to_fmt::<bech32::Bech32m, Formatter>(f, Self::HRP, &bytes).map_err(|err| {
      if let bech32::EncodeError::Fmt(err) = err {
        err
      } else {
        unreachable!()
      }
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn implementations() {
    fn case<const LEN: usize, T: Bech32m<LEN>>(hrp: &str, ty: &str) {
      use bech32::Checksum;

      let max = (bech32::Bech32m::CODE_LENGTH
        - T::HRP.as_str().len()
        - 1
        - bech32::Bech32m::CHECKSUM_LENGTH)
        * 5
        / 8;

      assert!(LEN <= max);

      assert_eq!(T::HRP.as_str(), hrp);

      assert_eq!(T::TYPE, ty);
    }

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
      "expected bech32m human-readable prefix `public1...` but found `private1...`",
    );

    case("public134jkgz", "expected 32 bytes but found 0");

    case(
      "public1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq6s7wps",
      "expected 32 bytes but found 33",
    );
  }
}
