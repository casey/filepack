use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ExifError {
  #[snafu(display("invalid byte order"))]
  ByteOrder,
  #[snafu(display("expected magic 42 but found {magic}"))]
  Magic { magic: u16 },
  #[snafu(display("invalid orientation count {count}"))]
  OrientationCount { count: u32 },
  #[snafu(display("invalid orientation type {ty}"))]
  OrientationType { ty: u16 },
  #[snafu(display("invalid orientation value {value}"))]
  OrientationValue { value: u16 },
  #[snafu(display("truncated EXIF data"))]
  Truncated,
}
