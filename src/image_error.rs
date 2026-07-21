use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ImageError {
  #[snafu(display("failed to decode JPEG"))]
  DecodeJpeg {
    source: zune_jpeg::errors::DecodeErrors,
  },
  #[snafu(display("failed to decode PNG"))]
  DecodePng { source: png::DecodingError },
  #[snafu(display("image is {actual} but metadata dimensions are {expected}"))]
  DimensionsMismatch {
    actual: Dimensions,
    expected: Dimensions,
  },
}
