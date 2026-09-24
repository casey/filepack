use super::*;

#[derive(Clone, Copy, Display, EnumString, Eq, Hash, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum ContainerAttribute {
  Magic,
  Strict,
  Transparent,
  Validate,
}
