use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Metadata {
  pub(crate) title: String,
}

impl Metadata {
  pub(crate) const FILENAME: &'static str = "metadata.yaml";

  pub(crate) fn deserialize(path: &Utf8Path, yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml).context(error::DeserializeMetadata { path })
  }

  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    Self::deserialize(path, &filesystem::read_to_string(path)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unknown_fields_are_rejected() {
    assert_eq!(
      serde_yaml::from_str::<Metadata>("title: foo\nbar: 1")
        .unwrap_err()
        .to_string(),
      "unknown field `bar`, expected `title` at line 2 column 1",
    );
  }
}
