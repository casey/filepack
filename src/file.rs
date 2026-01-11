use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct File {
  pub hash: Hash,
  pub size: u64,
}

impl File {
  pub(crate) fn fingerprint(&self) -> Hash {
    let mut hasher = FingerprintHasher::new(Context::File);
    hasher.field(0, Hash::bytes(&self.size.to_le_bytes()));
    hasher.field(1, self.hash);
    hasher.finalize()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unknown_fields_are_rejected() {
    assert!(
      serde_json::from_str::<File>(&r#"{"hash": "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262", "size": 0, "foo": null}"#)
        .unwrap_err()
        .to_string()
        .starts_with("unknown field `foo`")
    );
  }
}
