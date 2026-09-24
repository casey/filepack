use super::*;

#[derive(Clone, Copy, Debug, DecodeFromStr, Display, EncodeDisplay, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum MagicType {
  Archive,
  Metadata,
}
