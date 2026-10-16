use super::*;

#[derive(Default)]
pub(crate) struct Attributes {
  pub(crate) rename_all: Option<RenameAll>,
  pub(crate) strict: bool,
  pub(crate) tag: Option<String>,
}

impl Attributes {
  pub(crate) fn insert(&mut self, meta: &ParseNestedMeta, attribute: ContainerAttribute) -> Result {
    let name = <&str>::from(&attribute);

    let duplicate = match attribute {
      ContainerAttribute::DenyUnknownFields => {
        let duplicate = self.strict;
        self.strict = true;
        duplicate
      }
      ContainerAttribute::RenameAll(rename_all) => self.rename_all.replace(rename_all).is_some(),
      ContainerAttribute::Tag(tag) => self.tag.replace(tag).is_some(),
    };

    if duplicate {
      return Err(meta.error(format!("duplicate `#[serde({name})]` attribute")));
    }

    Ok(())
  }
}
