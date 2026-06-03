use super::*;

pub(crate) struct Resource {
  pub(crate) content_length: u64,
  pub(crate) file: fs::File,
  pub(crate) hash: Hash,
  pub(crate) range: Option<headers::Range>,
  pub(crate) ty: ResourceType,
}

impl Resource {
  pub(crate) fn range(self, range: Option<TypedHeader<headers::Range>>) -> Self {
    Self {
      range: range.map(|TypedHeader(range)| range),
      ..self
    }
  }

  pub(crate) fn ty(self, ty: ResourceType) -> Self {
    Self { ty, ..self }
  }
}

impl IntoResponse for Resource {
  fn into_response(self) -> Response {
    let mut builder = Response::builder()
      .header(header::ACCEPT_RANGES, "bytes")
      .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
      .header(header::CONTENT_SECURITY_POLICY, "sandbox")
      .header(header::CONTENT_TYPE, self.ty.content_type().essence_str())
      .header(header::ETAG, format!("\"{}\"", self.hash));

    if let Some(content_disposition) = self.ty.content_disposition() {
      builder = builder.header(header::CONTENT_DISPOSITION, content_disposition);
    }

    let len = self.content_length;

    let Some(range) = self.range else {
      return builder
        .header(header::CONTENT_LENGTH, len)
        .body(Body::from_stream(ReaderStream::new(
          tokio::fs::File::from_std(self.file),
        )))
        .unwrap();
    };

    let resolved = range
      .satisfiable_ranges(len)
      .next()
      .and_then(|(start, end)| {
        let start = match start {
          Bound::Included(start) => start,
          Bound::Excluded(start) => start + 1,
          Bound::Unbounded => 0,
        };

        let end = match end {
          Bound::Included(end) => end.min(len.saturating_sub(1)),
          Bound::Excluded(end) => end.saturating_sub(1),
          Bound::Unbounded => len.saturating_sub(1),
        };

        (len > 0 && start < len && start <= end).then_some((start, end))
      });

    let Some((start, end)) = resolved else {
      return builder
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(header::CONTENT_RANGE, format!("bytes */{len}"))
        .header(header::CONTENT_LENGTH, 0u64)
        .body(Body::empty())
        .unwrap();
    };

    let length = end - start + 1;

    let mut file = self.file;
    file.seek(SeekFrom::Start(start)).unwrap();

    builder
      .status(StatusCode::PARTIAL_CONTENT)
      .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"))
      .header(header::CONTENT_LENGTH, length)
      .body(Body::from_stream(ReaderStream::new(
        tokio::fs::File::from_std(file).take(length),
      )))
      .unwrap()
  }
}
