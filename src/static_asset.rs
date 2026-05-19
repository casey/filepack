use {super::*, axum::http::header, rust_embed::RustEmbed};

#[derive(RustEmbed)]
#[folder = "static"]
struct StaticAssets;

pub(crate) struct StaticAsset {
  content: Cow<'static, [u8]>,
  content_type: String,
  status: StatusCode,
}

impl StaticAsset {
  pub(crate) fn get(path: &str) -> ServerResult<Self> {
    let content = StaticAssets::get(path).context(server_error::PageNotFound)?;

    Ok(Self {
      content: content.data,
      content_type: content.metadata.mimetype().to_owned(),
      status: StatusCode::OK,
    })
  }

  pub(crate) fn status(mut self, status: StatusCode) -> Self {
    self.status = status;
    self
  }
}

impl IntoResponse for StaticAsset {
  fn into_response(self) -> Response<Body> {
    (
      self.status,
      [(header::CONTENT_TYPE, self.content_type)],
      self.content,
    )
      .into_response()
  }
}
