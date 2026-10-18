use super::*;

#[derive(Default)]
pub(crate) struct Attributes {
  pub(crate) magic: Option<Path>,
  pub(crate) non_empty_ordered_set: bool,
  pub(crate) sorted_set: bool,
  pub(crate) strict: bool,
  pub(crate) transparent: bool,
  pub(crate) undocumented: bool,
  pub(crate) validate: bool,
}

impl Attributes {
  pub(crate) fn insert(&mut self, meta: &ParseNestedMeta, attribute: ContainerAttribute) -> Result {
    let name = attribute.name();

    let duplicate = match attribute {
      ContainerAttribute::Magic(magic) => self.magic.replace(magic).is_some(),
      ContainerAttribute::NonEmptyOrderedSet => mem::replace(&mut self.non_empty_ordered_set, true),
      ContainerAttribute::SortedSet => mem::replace(&mut self.sorted_set, true),
      ContainerAttribute::Strict => mem::replace(&mut self.strict, true),
      ContainerAttribute::Transparent => mem::replace(&mut self.transparent, true),
      ContainerAttribute::Undocumented => mem::replace(&mut self.undocumented, true),
      ContainerAttribute::Validate => mem::replace(&mut self.validate, true),
    };

    if duplicate {
      return Err(meta.error(format!("duplicate `#[deco({name})]` attribute")));
    }

    Ok(())
  }
}
