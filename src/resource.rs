use super::*;

pub(crate) struct Resource {
  pub(crate) content_length: u64,
  pub(crate) file: fs::File,
  pub(crate) hash: Hash,
  pub(crate) ty: ResourceType,
}

impl Resource {
  pub(crate) fn ty(self, ty: ResourceType) -> Self {
    Self { ty, ..self }
  }
}

impl IntoResponse for Resource {
  fn into_response(self) -> Response {
    let mut builder = Response::builder()
      .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
      .header(header::CONTENT_LENGTH, self.content_length)
      .header(header::CONTENT_SECURITY_POLICY, "sandbox")
      .header(header::CONTENT_TYPE, self.ty.content_type().essence_str())
      .header(header::ETAG, format!("\"{}\"", self.hash));

    if let Some(content_disposition) = self.ty.content_disposition() {
      builder = builder.header(header::CONTENT_DISPOSITION, content_disposition);
    }

    builder
      .body(Body::from_stream(ReaderStream::new(
        tokio::fs::File::from_std(self.file),
      )))
      .unwrap()
  }
}
