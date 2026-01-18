use {super::*, bech32::primitives::decode::CheckedHrpstring};

pub(crate) trait Bech32m<const LEN: usize> {
  const HRP: Hrp;

  fn decode_bech32m(s: &str) -> Result<[u8; LEN], Bech32mError> {
    let hrp_string = CheckedHrpstring::new::<bech32::Bech32m>(s).context(bech32m_error::Decode)?;

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
      }
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
  fn implementation() {
    fn case<const LEN: usize, T: Bech32m<LEN>>(hrp: &str) {
      use bech32::Checksum;

      let max = (bech32::Bech32m::CODE_LENGTH
        - T::HRP.as_str().len()
        - 1
        - bech32::Bech32m::CHECKSUM_LENGTH)
        * 5
        / 8;

      assert!(LEN <= max);

      assert_eq!(T::HRP.as_str(), hrp);
    }

    case::<{ PrivateKey::LEN }, PrivateKey>("private");
    case::<{ PublicKey::LEN }, PublicKey>("public");
    case::<{ Signature::LEN }, Signature>("signature");
  }

  #[test]
  fn weak_encodings() {
    let weak_pub = [0u8; PublicKey::LEN];
    let weak_sig = [0u8; Signature::LEN];

    let pub_encoded = bech32::encode::<bech32::Bech32m>(PublicKey::HRP, &weak_pub).unwrap();
    let sig_encoded = bech32::encode::<bech32::Bech32m>(Signature::HRP, &weak_sig).unwrap();

    assert_eq!(
      pub_encoded,
      "public1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9xa2lj"
    );
    assert_eq!(
      sig_encoded,
      "signature1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq83s9ss"
    );
  }
}
