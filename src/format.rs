use super::*;

#[derive(Clone, ValueEnum)]
pub(crate) enum Format {
  Json,
  JsonPretty,
  Tsv,
}
