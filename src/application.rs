use super::*;

#[derive(EncodeDisplay, EnumString, Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Application {
  Filepack,
}
