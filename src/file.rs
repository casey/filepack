use super::*;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
  pub(crate) hash: Hash,
  pub(crate) size: u64,
}

impl File {
  #[cfg(test)]
  pub(crate) fn new(bytes: &[u8]) -> Self {
    Self {
      hash: Hash::bytes(bytes),
      size: bytes.len().into_u64(),
    }
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
