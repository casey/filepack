use super::*;

#[derive(DecodeFromStr, EncodeDisplay, EnumString, Display)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub(crate) enum Context {
  Statement,
}
