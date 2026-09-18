use super::*;

#[derive(Clone, Copy, Display, EnumString, Eq, Hash, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ContainerAttribute {
  AllowUnknownFields,
  Transparent,
  Validate,
}
