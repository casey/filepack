use super::*;

/// Statement attesting to a package.
#[derive(Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Statement {
  /// Fingerprint of the signed package.
  #[n(1)]
  pub fingerprint: Fingerprint,
}

impl Message for Statement {
  const CONTEXT: Context = Context::Statement;
  const EXAMPLE: &'static str = concat!(
    "signature101a03b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2902a201a02186aa",
    "6cefc1ae2f427b90e61a3ab91c88573f6f6719ca0188098d681b6c80ca03c02a595da6b7e9d82d21cf63b069839a",
    "dbdec89cfc6147f5ca09881dafa71ca777c99b63be16b579838006f5b15801b62d51e8dd73f0754036bf84e23a01",
    "e52101",
  );
  const EXAMPLE_DESCRIPTION: &'static str =
    "Signature over the fingerprint of the empty package by the all-zero private key.";
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
      fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
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
