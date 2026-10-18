use super::*;

#[derive(Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Statement {
  /// Fingerprint of the signed package.
  #[n(1)]
  pub fingerprint: Fingerprint,
  /// Unix timestamp of signing in seconds.
  #[n(2)]
  pub timestamp: Option<u64>,
}

impl Message for Statement {
  const CONTEXT: Context = Context::Statement;
  const EXAMPLE: &'static str = concat!(
    "signature101a03b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2902a401a0af1349",
    "b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262020003c0458b42064a66a78926efd7066f",
    "62098750af27dee51d4381da7f55007464a56db26eb0f6e4af961aa91c4c58be51ae250e938f0b405c1f4fa679e0",
    "dd4009870d",
  );
  const NAME: &'static str = "Statement";
  const TAG: Tag = Tag::Signature;
  type Error = Error;
  type Policy<'a> = Fingerprint;

  fn check(&self, _signer: PublicKey, fingerprint: Fingerprint) -> Result {
    ensure! {
      self.fingerprint == fingerprint,
      error::SignatureFingerprintMismatch {
        package: fingerprint,
        signature: self.fingerprint,
      },
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn example() {
    let private_key = PrivateKey::from([0; PrivateKey::LEN]);

    let statement = Statement {
      fingerprint: Fingerprint(Hash::bytes(b"")),
      timestamp: Some(0),
    };

    assert_eq!(
      private_key.sign(statement).to_string(),
      Attestation::EXAMPLE
    );
  }

  #[test]
  fn fingerprint_mismatch() {
    let fingerprint = Fingerprint::EXAMPLE.parse::<Fingerprint>().unwrap();
    let other = Fingerprint::from_bytes(default());
    assert_matches!(
      Attestation::EXAMPLE
        .parse::<Attestation>()
        .unwrap()
        .verify(other)
        .unwrap_err(),
      Error::SignatureFingerprintMismatch {
        package,
        signature,
        ..
      } if package == other && signature == fingerprint,
    );
  }
}
