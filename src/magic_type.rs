use super::*;

/// File type identified by magic bytes.
#[derive(Debug, DecodeFromStr, Display, EncodeDisplay, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum MagicType {
  Archive,
  Metadata,
}
