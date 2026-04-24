use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum DecodeError {
  #[snafu(display("failed to parse component"))]
  Component { source: ComponentError },
  #[snafu(display("failed to parse datetime"))]
  DateTime { source: chrono::ParseError },
  #[snafu(display("integer out of range"))]
  IntegerRange { source: TryFromIntError },
  #[snafu(display("map keys out of order"))]
  KeyOrder,
  #[snafu(display("failed to parse language code"))]
  Language { source: LanguageError },
  #[snafu(display("missing required field: {key}"))]
  MissingField { key: String },
  #[snafu(display("overlong integer"))]
  OverlongInteger,
  #[snafu(display("reserved additional information value: {value}"))]
  ReservedAdditionalInformation { value: u8 },
  #[snafu(display("size out of range"))]
  SizeRange { source: TryFromIntError },
  #[snafu(display("failed to parse tag"))]
  Tag { source: TagError },
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
  #[snafu(display("unexpected value, expected {expected} but found {actual}"))]
  UnexpectedValue {
    actual: String,
    expected: &'static str,
  },
  #[snafu(display("string not valid unicode"))]
  Unicode { source: Utf8Error },
  #[snafu(display("unsupported additional information value: {value}"))]
  UnsupportedAdditionalInformation { value: u8 },
  #[snafu(display("failed to parse URL"))]
  Url { source: ::url::ParseError },
}
