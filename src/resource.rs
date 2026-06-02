use super::*;

pub(crate) struct Resource {
  pub(crate) content_disposition: ContentDisposition,
  pub(crate) content_length: u64,
  pub(crate) content_type: Mime,
  pub(crate) file: fs::File,
  pub(crate) hash: Hash,
}

impl Resource {
  pub(crate) fn content_disposition(self, content_disposition: ContentDisposition) -> Self {
    Self {
      content_disposition,
      ..self
    }
  }

  pub(crate) fn content_type(self, content_type: Mime) -> Self {
    Self {
      content_type,
      ..self
    }
  }
}

impl IntoResponse for Resource {
  fn into_response(self) -> Response {
    let mut builder = Response::builder()
      .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
      .header(header::CONTENT_LENGTH, self.content_length)
      .header(header::CONTENT_SECURITY_POLICY, "sandbox")
      .header(header::CONTENT_TYPE, self.content_type.essence_str())
      .header(header::ETAG, format!("\"{}\"", self.hash));

    match self.content_disposition {
      ContentDisposition::Attachment => {
        builder = builder.header(header::CONTENT_DISPOSITION, "attachment")
      }
      ContentDisposition::Inline => {}
    }

    builder
      .body(Body::from_stream(ReaderStream::new(
        tokio::fs::File::from_std(self.file),
      )))
      .unwrap()
  }
}
