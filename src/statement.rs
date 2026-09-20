use super::*;

#[derive(Clone, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Statement {
  #[n(0)]
  pub fingerprint: Fingerprint,
  #[n(1)]
  pub timestamp: Option<u64>,
}

impl Message for Statement {
  const CONTEXT: Context = Context::Statement;
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
  fn fingerprint_mismatch() {
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();
    let other = Fingerprint::from_bytes(default());
    assert_matches!(
      test::SIGNATURE
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
