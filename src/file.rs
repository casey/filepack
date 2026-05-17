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

  pub(crate) fn eprint_mismatch(actual: Self, expected: Self, path: &str) {
    let style = Style::stderr();

    let hash_style = if expected.hash == actual.hash {
      style.good()
    } else {
      style.bad()
    };

    let size_style = if expected.size == actual.size {
      style.good()
    } else {
      style.bad()
    };

    eprintln!(
      "\
mismatched file: `{path}`
       manifest: {} ({} bytes)
           file: {} ({} bytes)",
      expected.hash.style(style.good()),
      expected.size.style(style.good()),
      actual.hash.style(hash_style),
      actual.size.style(size_style),
    );
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
