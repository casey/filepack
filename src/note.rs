use super::*;

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Note {
  #[serde_as(as = "SetPreventDuplicates<_>")]
  pub signatures: BTreeSet<Signature>,
  #[serde(default, skip_serializing_if = "is_default")]
  pub time: Option<u128>,
}

impl Note {
  pub(crate) fn from_message(message: Message, signature: Signature) -> Self {
    Self {
      signatures: [signature].into(),
      time: message.time,
    }
  }

  pub fn has_signature(&self, public_key: PublicKey) -> bool {
    self
      .signatures
      .iter()
      .any(|signature| signature.public_key() == public_key)
  }

  pub(crate) fn message(&self, fingerprint: Fingerprint) -> Message {
    Message {
      fingerprint,
      time: self.time,
    }
  }

  pub(crate) fn verify(&self, fingerprint: Fingerprint) -> Result<u64> {
    let serialized = self.message(fingerprint).serialize();
    for signature in &self.signatures {
      signature.verify(&serialized)?;
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
      serde_json::from_str::<Note>(r#"{"signatures":[],"signatures":[]}"#)
        .unwrap_err()
        .to_string(),
      "duplicate field `signatures` at line 1 column 29",
    );
  }

  #[test]
  fn duplicate_signatures_are_rejected() {
    let json = format!(
      r#"{{"signatures":["{}","{}"]}}"#,
      test::SIGNATURE,
      test::SIGNATURE,
    );

    assert_matches_regex! {
      serde_json::from_str::<Note>(&json).unwrap_err().to_string(),
      r"invalid entry: found duplicate value at line 1 column \d+",
    }
  }

  #[test]
  fn optional_fields_are_not_serialized() {
    assert_eq!(
      serde_json::to_string(&Note {
        signatures: BTreeSet::new(),
        time: None,
      })
      .unwrap(),
      r#"{"signatures":[]}"#,
    );
  }
}
