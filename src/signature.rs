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

impl Bech32m<3, { Signature::LEN }> for Signature {
  const TYPE: Bech32mType = Bech32mType::Signature;
  type Suffix = Vec<u8>;
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut encoder = Bech32mEncoder::new(Bech32mType::Signature);

    let payload = self.scheme.payload(self.inner);

    encoder.fes(&payload.prefix);

    encoder.bytes(&payload.body);

    encoder.bytes(&payload.suffix);

    write!(f, "{encoder}")
  }
}

impl fmt::Debug for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    <dyn std::fmt::Debug>::fmt(&self.inner, f)
  }
}

impl FromStr for Signature {
  type Err = SignatureError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut decoder = Bech32mDecoder::new(Bech32mType::Signature, s)?;

    let prefix = decoder.fe_array()?;

    let inner = decoder.byte_array()?;

    let suffix = decoder.into_bytes()?;

    let scheme = SignatureScheme::new(prefix, suffix)?;

    Ok(Self {
      inner: ed25519_dalek::Signature::from_bytes(&inner),
      scheme,
    })
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bech32::primitives::decode::{ChecksumError, CodeLengthError},
  };

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
      "signature1aq0q%62zsmd",
      "signature scheme `q` not supported",
    );

    case(
      "signature1afpq%40xf7h",
      "signature version `p` not supported with filepack signatures, expected version `0`",
    );

    case(
      "signature1af0p%eeas80",
      "signature hash algorithm `p` not supported with filepack signatures, expected hash algorithm `q`",
    );

    case(
      "signature1af0q%qqqqqqqqeyyw6u",
      "found unexpected 5 byte suffix on filepack signature",
    );
  }

  #[test]
  fn overlong_pgp_suffix() {
    let bech32m = [Fe32::P, Fe32::_4, Fe32::P]
      .into_iter()
      .chain([0u8; 64].into_iter().chain(vec![0u8; 65536]).bytes_to_fes())
      .with_checksum::<bech32::Bech32m>(Signature::TYPE.hrp())
      .with_witness_version(Fe32::A)
      .chars()
      .collect::<String>();

    let SignatureError::Bech32m { source } = bech32m.parse::<Signature>().unwrap_err() else {
      panic!("expected bech32m error");
    };

    let Bech32mError::Decode { ty, source } = source else {
      panic!("expected decode error");
    };

    assert_eq!(ty, Bech32mType::Signature);

    let CheckedHrpstringError::Checksum(err) = source else {
      panic!("expected checksum error");
    };

    assert_matches!(
      err,
      ChecksumError::CodeLength(CodeLengthError {
        encoded_length: 104_980,
        code_length: 1023,
        ..
      })
    );
  }

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
      .with_checksum::<bech32::Bech32m>(Signature::TYPE.hrp())
      .with_witness_version(Fe32::A)
      .chars()
      .collect::<String>();

    assert_eq!(
      bech32m.parse::<Signature>().unwrap_err().to_string(),
      "bech32m signature truncated",
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
    case("signature1af0q%fwxcmn", SignatureSchemeType::Filepack);
    case("signature1ap4p%qqypqxpq4vxtfa", SignatureSchemeType::Pgp);
    case("signature1as0p%hlmu87", SignatureSchemeType::Ssh);
  }
}
