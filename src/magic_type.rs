use super::*;

#[derive(Debug, DecodeFromStr, Display, EncodeDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum MagicType {
  Archive,
  Metadata,
}
