use super::*;

#[derive(DecodeFromStr, EncodeDisplay, EnumString, Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Context {
  Manifest,
  Message,
}
