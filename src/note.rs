use super::*;

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Note {
  #[serde_as(as = "MapPreventDuplicates<_, _>")]
  pub signatures: BTreeMap<PublicKey, Signature>,
}

impl Note {
  #[allow(clippy::unused_self)]
  pub(crate) fn digest(&self, fingerprint: Hash) -> Digest {
    Message { fingerprint }.digest()
  }

  pub(crate) fn from_message(
    _message: Message,
    public_key: PublicKey,
    signature: Signature,
  ) -> Self {
    Self {
      signatures: [(public_key, signature)].into(),
    }
  }

  pub(crate) fn has_signature(&self, public_key: PublicKey) -> bool {
    self.signatures.contains_key(&public_key)
  }

  pub(crate) fn verify(&self, fingerprint: Hash) -> Result<u64> {
    let digest = self.digest(fingerprint);
    for (public_key, signature) in &self.signatures {
      public_key.verify(digest, signature)?;
    }
    Ok(self.signatures.len().into_u64())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duplicate_signatures_are_rejected() {
    assert_eq!(
      serde_json::from_str::<Note>(r#"{"signatures":{"a":"a","a":"a"}}"#)
        .unwrap_err()
        .to_string(),
      "invalid entry: found duplicate key at line 1 column 15",
    );
  }
}
