use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum NumberError {
  #[snafu(transparent)]
  Integer { source: ParseIntError },
  #[snafu(display("invalid number `{number}`"))]
  Invalid { number: String },
}
