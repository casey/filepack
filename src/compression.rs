use super::*;

#[derive(Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Compression {
  Lossless,
  Lossy,
}
