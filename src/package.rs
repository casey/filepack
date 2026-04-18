use super::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Package {
  pub(crate) creator: Option<Component>,
  pub(crate) creator_tag: Option<Tag>,
  pub(crate) date: Option<DateTime>,
  pub(crate) description: Option<String>,
  pub(crate) homepage: Option<Url>,
  pub(crate) nfo: Option<filename::Nfo>,
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

    map.item_optional(0, self.creator.as_ref());
    map.item_optional(1, self.creator_tag.as_ref());
    map.item_optional(2, self.date.as_ref());
    map.item_optional(3, self.description.as_ref());
    map.item_optional(4, self.homepage.as_ref());
    map.item_optional(5, self.nfo.as_ref());
  }
}
