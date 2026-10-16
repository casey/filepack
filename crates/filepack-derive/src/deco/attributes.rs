use super::*;

#[derive(Default)]
pub(crate) struct Attributes {
  pub(crate) magic: Option<Path>,
  pub(crate) strict: bool,
  pub(crate) transparent: bool,
  pub(crate) validate: bool,
}

impl Attributes {
  pub(crate) fn insert(&mut self, meta: &ParseNestedMeta, attribute: ContainerAttribute) -> Result {
    let name = attribute.name();

    let duplicate = match attribute {
      ContainerAttribute::Magic(magic) => self.magic.replace(magic).is_some(),
      ContainerAttribute::Strict => mem::replace(&mut self.strict, true),
      ContainerAttribute::Transparent => mem::replace(&mut self.transparent, true),
      ContainerAttribute::Validate => mem::replace(&mut self.validate, true),
    };

    if duplicate {
      return Err(meta.error(format!("duplicate `#[deco({name})]` attribute")));
    }

    Ok(())
  }
}
