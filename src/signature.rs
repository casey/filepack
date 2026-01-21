use super::*;

#[derive(Clone, Copy, DeserializeFromStr, PartialEq, SerializeDisplay)]
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
    let signed_data = match self.scheme {
      SignatureScheme::Filepack => Cow::Borrowed(message.filepack_signed_data()),
      SignatureScheme::Pgp => Cow::Owned(message.pgp_signed_data()),
      SignatureScheme::Ssh => Cow::Owned(message.ssh_signed_data()),
    };

    public_key
      .inner()
      .verify_strict(&signed_data, &self.inner)
      .map_err(DalekSignatureError)
      .context(error::SignatureInvalid { public_key })
  }
}

impl Bech32m<1, { Signature::LEN }> for Signature {
  const HRP: Hrp = Hrp::parse_unchecked("signature");
  const TYPE: &'static str = "signature";
  type Suffix = Vec<u8>;
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let payload = bech32m::Payload {
      prefix: [self.scheme.into()],
      data: self.inner.to_bytes(),
      suffix: Vec::new(),
    };

    Self::encode_bech32m(f, payload)
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
    let bech32m::Payload {
      prefix: [scheme],
      data,
      suffix,
    } = Self::decode_bech32m(signature)?;

    Ok(Self {
      inner: ed25519_dalek::Signature::from_bytes(&data),
      scheme: scheme.try_into()?,
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
      "expected bech32m signature to have 1 prefix character but found 0",
    );
  }

  #[test]
  fn unsupported_scheme() {
    let bech32m = iter::once(Fe32::Q)
      .chain([0u8; Signature::LEN].iter().copied().bytes_to_fes())
      .with_checksum::<bech32::Bech32m>(&Signature::HRP)
      .with_witness_version(Fe32::A)
      .chars()
      .collect::<String>();

    assert_eq!(
      bech32m.parse::<Signature>().unwrap_err().to_string(),
      "bech32m signature scheme `q` is not supported",
    );
  }
}
