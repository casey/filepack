use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Package {
  pub(crate) creator: Option<Component>,
  pub(crate) creator_tag: Option<Tag>,
  pub(crate) date: Option<DateTime>,
  pub(crate) description: Option<String>,
  pub(crate) homepage: Option<Url>,
  pub(crate) nfo: Option<filename::Nfo>,
}

impl Decode for Package {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut map = decoder.map::<u8>()?;

    let creator = map.optional_key(0)?;
    let creator_tag = map.optional_key(1)?;
    let date = map.optional_key(2)?;
    let description = map.optional_key(3)?;
    let homepage = map.optional_key(4)?;
    let nfo = map.optional_key(5)?;

    map.finish()?;

    Ok(Self {
      creator,
      creator_tag,
      date,
      description,
      homepage,
      nfo,
    })
  }
}

impl Encode for Package {
  fn encode(&self, encoder: &mut Encoder) {
    let length = count_some!(
      self.creator,
      self.creator_tag,
      self.date,
      self.description,
      self.homepage,
      self.nfo,
    );

    let mut map = encoder.map::<u8>(length);

    map.optional_item(0, self.creator.as_ref());
    map.optional_item(1, self.creator_tag.as_ref());
    map.optional_item(2, self.date.as_ref());
    map.optional_item(3, self.description.as_ref());
    map.optional_item(4, self.homepage.as_ref());
    map.optional_item(5, self.nfo.as_ref());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Package {
      creator: Some("foo".parse().unwrap()),
      creator_tag: Some("A0".parse().unwrap()),
      date: Some("2024-01-01".parse().unwrap()),
      description: Some("bar".into()),
      homepage: Some("http://example.com".parse().unwrap()),
      nfo: Some("info.nfo".parse().unwrap()),
    });
  }
}
