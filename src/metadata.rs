use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Encode, Decode, PartialEq, Serialize)]
pub(crate) struct Metadata {
  #[n(0)]
  pub(crate) artwork: Option<filename::Png>,
  #[n(1)]
  pub(crate) creator: Option<ComponentBuf>,
  #[n(2)]
  pub(crate) date: Option<DateTime>,
  #[n(3)]
  pub(crate) description: Option<String>,
  #[n(4)]
  pub(crate) homepage: Option<Url>,
  #[n(5)]
  pub(crate) language: Option<Language>,
  #[n(6)]
  pub(crate) package: Option<Package>,
  #[n(7)]
  pub(crate) readme: Option<filename::Md>,
  #[n(8)]
  pub(crate) title: Option<ComponentBuf>,
}

impl Metadata {
  pub(crate) const CBOR_FILENAME: &'static str = "metadata.filepack";
  pub(crate) const YAML_FILENAME: &'static str = "metadata.yaml";

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

    if let Some(package) = &self.package
      && let Some(nfo) = &package.nfo
    {
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
  fn deserialize_allows_missing_optional_fields() {
    Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), "title: Foo").unwrap();
  }

  #[test]
  fn deserializer_allows_unknown_fields() {
    Metadata::deserialize(Metadata::YAML_FILENAME.as_ref(), UNKNOWN_FIELD).unwrap();
  }

  #[test]
  fn encoding() {
    assert_encoding(Metadata {
      artwork: Some("cover.png".parse().unwrap()),
      creator: Some("foo".parse().unwrap()),
      date: Some("2024".parse().unwrap()),
      description: Some("bar".into()),
      homepage: Some("http://example.com".parse().unwrap()),
      language: Some("en".parse().unwrap()),
      package: Some(Package {
        creator: Some("baz".parse().unwrap()),
        creator_tag: Some("A0".parse().unwrap()),
        date: Some("2024-01-01".parse().unwrap()),
        description: Some("qux".into()),
        homepage: Some("http://example.com/foo".parse().unwrap()),
        nfo: Some("info.nfo".parse().unwrap()),
        title: Some("foo-bar".parse().unwrap()),
      }),
      readme: Some("README.md".parse().unwrap()),
      title: Some("foo".parse().unwrap()),
    });
  }

  #[test]
  fn filepack_metadata_is_valid() {
    Metadata::load_strict(Metadata::YAML_FILENAME.as_ref()).unwrap();
  }

  #[test]
  fn metadata_in_readme_is_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```yaml(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      let metadata = Metadata::deserialize_strict("README.md".as_ref(), &capture[1]).unwrap();

      let Metadata {
        artwork,
        creator,
        date,
        description,
        homepage,
        language,
        package,
        readme,
        title,
      } = metadata;

      if title
        .as_ref()
        .is_none_or(|title| *title != "Tobin's Spirit Guide")
      {
        continue;
      }

      assert!(artwork.is_some());
      assert!(creator.is_some());
      assert!(date.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(language.is_some());
      assert!(readme.is_some());
      assert!(title.is_some());

      let Package {
        creator,
        creator_tag,
        date,
        description,
        homepage,
        nfo,
        title,
      } = package.unwrap();

      assert!(creator.is_some());
      assert!(creator_tag.is_some());
      assert!(date.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(nfo.is_some());
      assert!(title.is_some());
    }
  }

  #[test]
  fn strict_deserialize_rejects_unknown_fields() {
    assert_eq!(
      Metadata::deserialize_strict(Metadata::YAML_FILENAME.as_ref(), UNKNOWN_FIELD)
        .unwrap_err()
        .to_string(),
      "unknown fields in metadata at `metadata.yaml`: `bar`",
    );
  }
}
