use super::*;

static AUDIO_FLAC: LazyLock<Mime> = LazyLock::new(|| "audio/flac".parse().unwrap());

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

  pub(crate) fn content_type(self) -> &'static Mime {
    match self {
      Self::Binary => &mime::APPLICATION_OCTET_STREAM,
      Self::Flac => &AUDIO_FLAC,
      Self::Jpeg => &mime::IMAGE_JPEG,
      Self::Png => &mime::IMAGE_PNG,
    }
  }
}
