use super::*;

/// Application a signed message belongs to.
#[derive(EncodeDisplay, Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Application {
  Filepack,
}
