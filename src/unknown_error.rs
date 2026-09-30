use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum UnknownError {
  #[snafu(display("invalid discriminant {discriminant} for enum {name}"))]
  Discriminant {
    discriminant: u64,
    name: &'static str,
  },
  #[snafu(display("failed to parse language code"))]
  Language { source: LanguageError },
  #[snafu(display("unsupported version {version} for {name}"))]
  Version { name: &'static str, version: u64 },
}
