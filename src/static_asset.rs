use {super::*, http::header, rust_embed::RustEmbed};

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
      content_type: match content.metadata.mimetype() {
        "application/x-sh" => "application/x-sh; charset=utf-8".into(),
        "text/css" => "text/css; charset=utf-8".into(),
        "text/html" => "text/html; charset=utf-8".into(),
        mimetype => mimetype.into(),
      },
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn content_type() {
    #[track_caller]
    fn case(path: &str, content_type: &str) {
      assert_eq!(StaticAsset::get(path).unwrap().content_type, content_type);
    }

    case("favicon.png", "image/png");
    case("index.css", "text/css; charset=utf-8");
    case("index.html", "text/html; charset=utf-8");
    case("install.sh", "application/x-sh; charset=utf-8");
  }
}
