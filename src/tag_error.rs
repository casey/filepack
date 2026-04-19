use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum TagError {
  #[snafu(display("tags must match regex `{}`", &re::TAG.as_str()[1..re::TAG.as_str().len() - 1]))]
  Parse,
}
