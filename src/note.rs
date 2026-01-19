use super::*;

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Note {
  #[serde_as(as = "MapPreventDuplicates<_, _>")]
  pub signatures: BTreeMap<PublicKey, Signature>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub time: Option<u128>,
}

impl Note {
  pub(crate) fn digest(&self, fingerprint: Fingerprint) -> Digest {
    Message {
      fingerprint,
      time: self.time,
    }
    .digest()
  }

  pub(crate) fn from_message(
    message: Message,
    public_key: PublicKey,
    signature: Signature,
  ) -> Self {
    Self {
      signatures: [(public_key, signature)].into(),
      time: message.time,
    }
  }

  pub(crate) fn has_signature(&self, public_key: PublicKey) -> bool {
    self.signatures.contains_key(&public_key)
  }

  pub(crate) fn verify(&self, fingerprint: Fingerprint) -> Result<u64> {
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
  fn duplicate_fields_are_rejected() {
    assert_eq!(
      serde_json::from_str::<Note>(r#"{"signatures":{},"signatures":{}}"#)
        .unwrap_err()
        .to_string(),
      "duplicate field `signatures` at line 1 column 29",
    );
  }

  #[test]
  fn duplicate_signatures_are_rejected() {
    let json = format!(
      r#"{{"signatures":{{"{}":"{}","{}":"{}"}}}}"#,
      test::PUBLIC_KEY,
      test::SIGNATURE,
      test::PUBLIC_KEY,
      test::SIGNATURE,
    );
    assert_eq!(
      serde_json::from_str::<Note>(&json).unwrap_err().to_string(),
      "invalid entry: found duplicate key at line 1 column 399",
    );
  }

  #[test]
  fn optional_fields_are_not_serialized() {
    assert_eq!(
      serde_json::to_string(&Note {
        signatures: BTreeMap::new(),
        time: None,
      })
      .unwrap(),
      r#"{"signatures":{}}"#,
    );
  }
}
