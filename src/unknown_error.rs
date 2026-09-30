use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum UnknownError {
  #[snafu(display("invalid discriminant {discriminant} for enum {name}"))]
  Discriminant {
    discriminant: u64,
    name: &'static str,
  },
  #[snafu(display("unknown field with key {key}"))]
  Field { key: u64 },
  #[snafu(display("failed to parse language code"))]
  Language { source: LanguageError },
  #[snafu(display("unknown URL scheme `{scheme}`"))]
  Scheme { scheme: String },
  #[snafu(display("unsupported version {version} for {name}"))]
  Version { name: &'static str, version: u64 },
}
