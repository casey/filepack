use super::*;

#[derive(EnumString, Eq, Hash, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ContainerAttribute {
  Transparent,
  Validate,
}
