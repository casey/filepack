use super::*;

pub(crate) struct Attributes {
  pub(crate) flags: HashSet<ContainerAttribute>,
  pub(crate) magic: Option<LitByteStr>,
}

impl Attributes {
  pub(crate) fn magic(&self) -> Option<&LitByteStr> {
    self.magic.as_ref()
  }

  pub(crate) fn strict(&self) -> bool {
    self.flags.contains(&ContainerAttribute::Strict)
  }

  pub(crate) fn transparent(&self) -> bool {
    self.flags.contains(&ContainerAttribute::Transparent)
  }

  pub(crate) fn validate(&self) -> bool {
    self.flags.contains(&ContainerAttribute::Validate)
  }
}
