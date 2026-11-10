use super::*;

#[derive(Clone, Default, Display, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum OutputFormatJson {
  Json,
  #[default]
  JsonPretty,
}
