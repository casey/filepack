use super::*;

pub(crate) struct Attributes(pub(crate) HashSet<ContainerAttribute>);

impl Attributes {
  pub(crate) fn allow_unknown_fields(&self) -> bool {
    self.0.contains(&ContainerAttribute::AllowUnknownFields)
  }

  pub(crate) fn transparent(&self) -> bool {
    self.0.contains(&ContainerAttribute::Transparent)
  }

  pub(crate) fn validate(&self) -> bool {
    self.0.contains(&ContainerAttribute::Validate)
  }
}
