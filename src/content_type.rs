use super::*;

#[derive(Clone, Copy)]
pub(crate) enum ResourceType {
  Binary,
  Flac,
  Jpeg,
  Png,
}

impl ResourceType {
  pub(crate) fn content_disposition(self) -> Option<HeaderValue> {
    match self {
      Self::Binary => Some(HeaderValue::from_static("attachment")),
      Self::Flac | Self::Jpeg | Self::Png => None,
    }
  }

  pub(crate) fn content_type(self) -> Mime {
    match self {
      Self::Binary => mime::APPLICATION_OCTET_STREAM,
      Self::Flac => "audio/flac".parse().unwrap(),
      Self::Jpeg => mime::IMAGE_JPEG,
      Self::Png => mime::IMAGE_PNG,
    }
  }
}
