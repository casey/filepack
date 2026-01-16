use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct File {
  pub hash: Hash,
  pub size: u64,
}

impl File {
  pub(crate) fn fingerprint(&self) -> Fingerprint {
    let mut hasher = FingerprintHasher::new(FingerprintPrefix::File);
    hasher.field(0, self.hash.as_bytes());
    hasher.field(1, &self.size.to_le_bytes());
    hasher.finalize()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unknown_fields_are_rejected() {
    assert!(
      serde_json::from_str::<File>(&format!(
        r#"{{"hash": "{}", "size": 0, "foo": null}}"#,
        test::HASH,
      ))
      .unwrap_err()
      .to_string()
      .starts_with("unknown field `foo`")
    );
  }
}
