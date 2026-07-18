use super::*;

const MAX_CACHE: &str = "public, max-age=31536000, immutable";

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
  fn into_response(mut self) -> Response {
    fn response(result: http::Result<Response>) -> Response {
      match result {
        Ok(response) => response,
        Err(err) => server_error::InvalidResponse
          .into_error(err)
          .into_response(),
      }
    }

    let mut builder = Response::builder()
      .header(header::ACCEPT_RANGES, "bytes")
      .header(header::CONTENT_TYPE, self.ty.content_type().essence_str())
      .header(header::ETAG, format!("\"{}\"", self.hash));

    if self.ty.sandbox() {
      builder = builder.header(header::CONTENT_SECURITY_POLICY, "sandbox");
    }

    if let Some(content_disposition) = self.ty.content_disposition() {
      builder = builder.header(header::CONTENT_DISPOSITION, content_disposition);
    }

    let Some(range) = self.range else {
      return response(
        builder
          .header(header::CACHE_CONTROL, MAX_CACHE)
          .header(header::CONTENT_LENGTH, self.content_length)
          .body(Body::from_stream(ReaderStream::new(
            tokio::fs::File::from_std(self.file),
          ))),
      );
    };

    let resolved = range
      .satisfiable_ranges(self.content_length)
      .next()
      .and_then(|(start, end)| {
        let start = match start {
          Bound::Included(start) => start,
          Bound::Excluded(start) => start + 1,
          Bound::Unbounded => 0,
        };

        let end = match end {
          Bound::Included(end) => end.min(self.content_length.saturating_sub(1)),
          Bound::Excluded(end) => end.saturating_sub(1),
          Bound::Unbounded => self.content_length.saturating_sub(1),
        };

        (self.content_length > 0 && start < self.content_length && start <= end)
          .then_some((start, end))
      });

    let Some((start, end)) = resolved else {
      return response(
        builder
          .status(StatusCode::RANGE_NOT_SATISFIABLE)
          .header(
            header::CONTENT_RANGE,
            format!("bytes */{}", self.content_length),
          )
          .header(header::CONTENT_LENGTH, 0)
          .body(Body::empty()),
      );
    };

    let length = end - start + 1;

    if let Err(err) = self.file.seek(SeekFrom::Start(start)) {
      return server_error::FileIo { hash: self.hash }
        .into_error(err)
        .into_response();
    }

    response(
      builder
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CACHE_CONTROL, MAX_CACHE)
        .header(
          header::CONTENT_RANGE,
          format!("bytes {start}-{end}/{}", self.content_length),
        )
        .header(header::CONTENT_LENGTH, length)
        .body(Body::from_stream(ReaderStream::new(
          tokio::fs::File::from_std(self.file).take(length),
        ))),
    )
  }
}
