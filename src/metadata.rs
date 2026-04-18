use super::*;

// todo:
// - this seems fine
// - metadata command is good since JSON is easier to manipulate
// - cbor manifest is probably good
// - cbor is efficient, canonical, and simple
// - do the annoying thing first, if users hate it i can relent

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Metadata {
  pub(crate) artwork: Option<filename::Png>,
  pub(crate) creator: Option<Component>,
  pub(crate) date: Option<DateTime>,
  pub(crate) description: Option<String>,
  pub(crate) homepage: Option<Url>,
  pub(crate) language: Option<Language>,
  pub(crate) package: Option<Package>,
  pub(crate) readme: Option<filename::Md>,
  pub(crate) title: Component,
}

impl Decode for Metadata {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut map = decoder.map::<u8>()?;

    let artwork = map.optional_key(0)?;
    let creator = map.optional_key(1)?;
    let date = map.optional_key(2)?;
    let description = map.optional_key(3)?;
    let homepage = map.optional_key(4)?;
    let language = map.optional_key(5)?;
    let package = map.optional_key(6)?;
    let readme = map.optional_key(7)?;
    let title = map.required_key(8)?;

    map.finish()?;

    Ok(Self {
      artwork,
      creator,
      date,
      description,
      homepage,
      language,
      package,
      readme,
      title,
    })
  }
}

impl Encode for Metadata {
  fn encode(&self, encoder: &mut Encoder) {
    let length = 1
      + count_some!(
        self.artwork,
        self.creator,
        self.date,
        self.description,
        self.homepage,
        self.language,
        self.package,
        self.readme,
      );

    let mut map = encoder.map::<u8>(length);

    map.optional_item(0, self.artwork.as_ref());
    map.optional_item(1, self.creator.as_ref());
    map.optional_item(2, self.date.as_ref());
    map.optional_item(3, self.description.as_ref());
    map.optional_item(4, self.homepage.as_ref());
    map.optional_item(5, self.language.as_ref());
    map.optional_item(6, self.package.as_ref());
    map.optional_item(7, self.readme.as_ref());
    map.item(8, &self.title);
  }
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
        creator,
        date,
        description,
        homepage,
        language,
        package,
        readme,
        title,
      } = metadata;

      assert!(!title.as_str().is_empty());

      if title != "Tobin's Spirit Guide" {
        continue;
      }

      assert!(artwork.is_some());
      assert!(creator.is_some());
      assert!(date.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(language.is_some());
      assert!(readme.is_some());

      let Package {
        creator,
        creator_tag,
        date,
        description,
        homepage,
        nfo,
      } = package.unwrap();

      assert!(creator.is_some());
      assert!(creator_tag.is_some());
      assert!(date.is_some());
      assert!(description.is_some());
      assert!(homepage.is_some());
      assert!(nfo.is_some());
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
