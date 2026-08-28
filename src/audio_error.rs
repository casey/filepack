use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum AudioError {
  #[snafu(display("failed to decode embedded JPEG image"))]
  EmbeddedImageDecodeJpeg {
    source: zune_jpeg::errors::DecodeErrors,
  },
  #[snafu(display("failed to decode embedded PNG image"))]
  EmbeddedImageDecodePng { source: png::DecodingError },
  #[snafu(display("embedded image has unsupported media type `{media_type}`"))]
  EmbeddedImageMediaType { media_type: Mime },
  #[snafu(display("embedded image has invalid media type `{media_type}`"))]
  EmbeddedImageMediaTypeParse {
    media_type: String,
    source: mime::FromStrError,
  },
  #[snafu(display("embedded image media type is not valid UTF-8"))]
  EmbeddedImageMediaTypeUtf8 { source: Utf8Error },
  #[snafu(display("failed to decode FLAC"))]
  FlacDecode { source: claxon::Error },
  #[snafu(display("unknown sample count"))]
  FlacSampleCountUnknown,
  #[snafu(display("truncated FLAC metadata block"))]
  FlacTruncated,
  #[snafu(display("failed to decode MP3"))]
  Mp3Decode { source: Mp3Error },
  #[snafu(display("failed to read ID3 tag"))]
  Mp3Tag { source: id3::Error },
  #[snafu(display("missing ID3 tag"))]
  Mp3TagMissing,
  #[snafu(display("empty `{tag}` tag"))]
  TagEmpty { tag: &'static str },
  #[snafu(display("invalid integer `{tag}` tag"))]
  TagInteger {
    source: NumberError,
    tag: &'static str,
  },
  #[snafu(display("invalid `{tag}` tag"))]
  TagInvalid {
    source: TextError,
    tag: &'static str,
  },
  #[snafu(display("missing `{tag}` tag"))]
  TagMissing { tag: &'static str },
  #[snafu(display("multiple `{tag}` tags"))]
  TagMultiple { tag: &'static str },
  #[snafu(display("`{tag}` tag not in format `NUMBER/TOTAL`"))]
  TagPair { tag: &'static str },
}
