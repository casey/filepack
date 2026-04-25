use super::*;

#[derive(DecodeFromStr, EncodeDisplay, EnumString, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Context {
  Manifest,
  Message,
}
