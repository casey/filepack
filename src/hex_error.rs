use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum HexError {
  #[snafu(display("failed to decode {}", tag.name()))]
  DecoDecode { source: DecodeError, tag: Tag },
  #[snafu(display("{} contains invalid hex digit `{}`", tag.name(), digit.escape_default()))]
  Digit { digit: char, tag: Tag },
  #[snafu(display("{} has odd number of hex digits: {len}", tag.name()))]
  OddLength { len: usize, tag: Tag },
  #[snafu(display("{} missing tag `{}1…`", tag.name(), tag.prefix()))]
  TagMissing { tag: Tag },
  #[snafu(display(
    "expected {} with tag `{}1…` but found `{actual}1…`",
    expected.name(), expected.prefix(),
  ))]
  UnexpectedTag { actual: String, expected: Tag },
}
