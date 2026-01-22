use super::*;

#[derive(Clone, DeserializeFromStr, PartialEq, SerializeDisplay)]
pub struct Signature {
  inner: ed25519_dalek::Signature,
  scheme: SignatureScheme,
}

impl Signature {
  pub(crate) const LEN: usize = ed25519_dalek::Signature::BYTE_SIZE;

  pub(crate) fn new(scheme: SignatureScheme, inner: ed25519_dalek::Signature) -> Self {
    Self { inner, scheme }
  }

  pub fn verify(&self, message: &SerializedMessage, public_key: PublicKey) -> Result {
    let signed_data = self.scheme.signed_data(message);

    public_key
      .inner()
      .verify_strict(&signed_data, &self.inner)
      .map_err(DalekSignatureError)
      .context(error::SignatureInvalid { public_key })
  }
}

impl Bech32m<2, { Signature::LEN }> for Signature {
  const HRP: Hrp = Hrp::parse_unchecked("signature");
  const TYPE: &'static str = "signature";
  type Suffix = Vec<u8>;
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Self::encode_bech32m(f, self.scheme.payload(self.inner))
  }
}

impl fmt::Debug for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.inner, f)
  }
}

impl FromStr for Signature {
  type Err = SignatureError;

  fn from_str(signature: &str) -> Result<Self, Self::Err> {
    let Bech32mPayload {
      prefix: [scheme, version],
      data,
      suffix,
    } = Self::decode_bech32m(signature)?;

    Ok(Self {
      inner: ed25519_dalek::Signature::from_bytes(&data),
      scheme: SignatureScheme::new(scheme, version, suffix)?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    let message = Message {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      time: None,
    };
    let signature = PrivateKey::generate().sign(&message.serialize());
    assert_eq!(
      signature.to_string().parse::<Signature>().unwrap(),
      signature
    );
  }

  #[test]
  fn prefix_length() {
    let bech32m = []
      .iter()
      .copied()
      .bytes_to_fes()
      .with_checksum::<bech32::Bech32m>(&Signature::HRP)
      .with_witness_version(Fe32::A)
      .chars()
      .collect::<String>();

    assert_eq!(
      bech32m.parse::<Signature>().unwrap_err().to_string(),
      "expected bech32m signature to have 2 prefix characters but found 0",
    );
  }

  #[test]
  fn error_display() {
    #[track_caller]
    fn case(bech32m: &str, expected: &str) {
      assert_eq!(
        bech32m
          .replace('%', &"q".repeat(103))
          .parse::<Signature>()
          .unwrap_err()
          .to_string(),
        expected
      );
    }

    case("foo", "failed to decode bech32m signature");

    case(
      "signature1aq0%dcnjdk",
      "signature scheme `q` is not supported",
    );

    case(
      "signature1afp%fcu5ju",
      "signature scheme `filepack` version `p` is not supported, expected `0`",
    );

    case(
      "signature1af0%qqqqqqqqk7md3j",
      "found unexpected suffix for signature scheme `filepack`",
    );
  }

  #[test]
  fn round_trip() {
    #[track_caller]
    fn case(bech32m: &str, expected: SignatureSchemeType) {
      let bech32m = bech32m.replace('%', &"q".repeat(103));
      let signature = bech32m.parse::<Signature>().unwrap();
      assert_eq!(signature.scheme.discriminant(), expected);
      assert_eq!(signature.to_string(), bech32m);
    }
    case("signature1af0%ldnl7s", SignatureSchemeType::Filepack);
    case("signature1ap4%qqypqxpqk2fwrl", SignatureSchemeType::Pgp);
    case("signature1as0%yxnqs4", SignatureSchemeType::Ssh);
  }
}
