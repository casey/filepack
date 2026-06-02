use super::*;

pub(crate) enum ContentDisposition {
  Attachment,
  Inline,
}

impl ContentDisposition {
  pub(crate) fn value(self) -> Option<HeaderValue> {
    match self {
      Self::Attachment => Some(HeaderValue::from_static("attachment")),
      Self::Inline => None,
    }
  }
}
