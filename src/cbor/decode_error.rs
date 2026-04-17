use super::*;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum DecodeError {
  #[snafu(display("failed to decode component"))]
  Component { source: ComponentError },
  #[snafu(display("integer out of range"))]
  IntegerRange { source: TryFromIntError },
  #[snafu(display("map keys out of order"))]
  KeyOrder,
  #[snafu(display("overlong integer"))]
  OverlongInteger,
  #[snafu(display("reserved additional information value: {value}"))]
  ReservedAdditionalInformation { value: u8 },
  #[snafu(display("trailing bytes"))]
  TrailingBytes,
  #[snafu(display("unconsumed map entries"))]
  UnconsumedEntries,
  #[snafu(display("unexpected key"))]
  UnexpectedKey,
  #[snafu(display("unexpected value"))]
  UnexpectedValue,
  #[snafu(display("expected {expected} but found {actual}"))]
  UnexpectedType {
    expected: MajorType,
    actual: MajorType,
  },
  #[snafu(display("unsupported additional information value: {value}"))]
  UnsupportedAdditionalInformation { value: u8 },
}
