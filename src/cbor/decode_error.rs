use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum DecodeError {
  #[snafu(transparent)]
  Component { source: ComponentError },
  #[snafu(display("key order violation"))]
  KeyOrder,
  #[snafu(display("overlong integer"))]
  OverlongInteger,
  #[snafu(display("trailing bytes"))]
  TrailingBytes,
  #[snafu(display("expected {expected}, got {actual}"))]
  TypeMismatch {
    expected: MajorType,
    actual: MajorType,
  },
  #[snafu(display("unconsumed map entries"))]
  UnconsumedEntries,
  #[snafu(display("unexpected key"))]
  UnexpectedKey,
  #[snafu(display("unexpected value"))]
  UnexpectedValue,
  #[snafu(display("unsupported additional info: {value}"))]
  UnsupportedAdditionalInfo { value: u8 },
}
