use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Metadata {
  pub(crate) artwork: Option<filename::Png>,
  pub(crate) description: Option<String>,
  pub(crate) language: Option<Language>,
  pub(crate) nfo: Option<filename::Nfo>,
  pub(crate) package_date: Option<DateTime>,
  pub(crate) package_description: Option<String>,
  pub(crate) packager: Option<Component>,
  pub(crate) readme: Option<filename::Md>,
  pub(crate) release_date: Option<DateTime>,
  pub(crate) title: Component,
}

impl Metadata {
  pub(crate) const FILENAME: &'static str = "metadata.yaml";

  pub(crate) fn deserialize(path: &Utf8Path, yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml).context(error::DeserializeMetadata { path })
  }

  pub(crate) fn deserialize_strict(path: &Utf8Path, yaml: &str) -> Result<Self> {
    let deserializer = serde_yaml::Deserializer::from_str(yaml);

    let mut unknown = BTreeSet::new();

    let metadata = serde_ignored::deserialize(deserializer, |path| {
      unknown.insert(path.to_string());
    })
    .context(error::DeserializeMetadata { path })?;

    if !unknown.is_empty() {
      return Err(error::DeserializeMetadataStrict { path, unknown }.build());
    }

    Ok(metadata)
  }

  pub(crate) fn files(&self) -> Vec<RelativePath> {
    let mut files = Vec::new();

    if let Some(artwork) = &self.artwork {
      files.push(artwork.as_path());
    }

    if let Some(nfo) = &self.nfo {
      files.push(nfo.as_path());
    }

    if let Some(readme) = &self.readme {
      files.push(readme.as_path());
    }

    files
  }

  pub(crate) fn load_strict(path: &Utf8Path) -> Result<Self> {
    Self::deserialize_strict(path, &filesystem::read_to_string(path)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const UNKNOWN_FIELD: &str = "title: foo\nbar: 1";

  #[test]
  fn deserializer_allows_unknown_fields() {
    Metadata::deserialize(Metadata::FILENAME.as_ref(), UNKNOWN_FIELD).unwrap();
  }

  #[test]
  fn filepack_metadata_is_valid() {
    Metadata::load_strict(Metadata::FILENAME.as_ref()).unwrap();
  }

  #[test]
  fn metadata_in_readme_is_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```yaml(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let metadata = Metadata::deserialize_strict("README.md".as_ref(), &capture[1]).unwrap();

      let Metadata {
        artwork,
        description,
        language,
        nfo,
        package_date,
        package_description,
        packager,
        readme,
        release_date,
        title: _,
      } = metadata;

      assert!(artwork.is_some());
      assert!(description.is_some());
      assert!(language.is_some());
      assert!(nfo.is_some());
      assert!(package_date.is_some());
      assert!(package_description.is_some());
      assert!(packager.is_some());
      assert!(readme.is_some());
      assert!(release_date.is_some());
    }
  }

  #[test]
  fn strict_deserialize_rejects_unknown_fields() {
    assert_eq!(
      Metadata::deserialize_strict(Metadata::FILENAME.as_ref(), UNKNOWN_FIELD)
        .unwrap_err()
        .to_string(),
      "unknown fields in metadata at `metadata.yaml`: `bar`",
    );
  }
}
