use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum DecodeError {
  #[snafu(display("failed to parse component"))]
  Component { source: ComponentError },
  #[snafu(display("integer out of range"))]
  IntegerRange { source: TryFromIntError },
  #[snafu(display("map keys out of order"))]
  KeyOrder,
  #[snafu(display("missing required field: {key}"))]
  MissingField { key: String },
  #[snafu(display("overlong integer"))]
  OverlongInteger,
  #[snafu(display("failed to parse text: {message}"))]
  Parse { message: String },
  #[snafu(display("reserved additional information value: {value}"))]
  ReservedAdditionalInformation { value: u8 },
  #[snafu(display("size out of range"))]
  SizeRange { source: TryFromIntError },
  #[snafu(display("trailing bytes"))]
  TrailingBytes,
  #[snafu(display("truncated"))]
  Truncated,
  #[snafu(display("unconsumed map entries"))]
  UnconsumedEntries,
  #[snafu(display("unexpected key"))]
  UnexpectedKey,
  #[snafu(display("expected {expected} but found {actual}"))]
  UnexpectedType {
    expected: MajorType,
    actual: MajorType,
  },
  #[snafu(display("unexpected value"))]
  UnexpectedValue,
  #[snafu(display("string not valid unicode"))]
  Unicode { source: Utf8Error },
  #[snafu(display("unsupported additional information value: {value}"))]
  UnsupportedAdditionalInformation { value: u8 },
}
