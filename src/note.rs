use super::*;

// todo:
// - ensure duplicate map items are forbidden
// - ensure duplicate set items are forbidden
// - duplicate notes are forbidden
// - multiple signatures from the same pubkey are forbidden
// - serialize duration as integer nanoseconds?
// - does duration forbid additional fields?
// - sort notes by digest
// - add time to note
//   --time none
//   --time now
//   --time N.M

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Note {
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
