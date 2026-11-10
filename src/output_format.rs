use super::*;

#[derive(Clone, Default, Display, ValueEnum)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum OutputFormat {
  Json,
  #[default]
  JsonPretty,
  Tsv,
}
