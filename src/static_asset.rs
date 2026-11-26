use {super::*, http::header, rust_embed::RustEmbed};

#[derive(RustEmbed)]
#[folder = "static"]
struct StaticAssets;

pub(crate) struct StaticAsset {
  content: Cow<'static, [u8]>,
  content_type: String,
}

impl StaticAsset {
  pub(crate) fn get(path: &str) -> ServerResult<Self> {
    let content = StaticAssets::get(path).context(server_error::PageNotFound)?;

    Ok(Self {
      content: content.data,
      content_type: match content.metadata.mimetype() {
        "application/x-sh" => "application/x-sh;charset=utf-8".into(),
        "text/css" => "text/css;charset=utf-8".into(),
        "text/javascript" => "text/javascript;charset=utf-8".into(),
        mimetype => mimetype.into(),
      },
    })
  }
}

impl IntoResponse for StaticAsset {
  fn into_response(self) -> Response<Body> {
    ([(header::CONTENT_TYPE, self.content_type)], self.content).into_response()
  }
}

impl Describe for StaticAsset {
  fn describe(endpoint: &mut reference::Endpoint, _registry: &mut deco::Registry) {
    endpoint.response = Some(reference::Response::Static);
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

    case("favicon.svg", "image/svg+xml");
    case("install.sh", "application/x-sh;charset=utf-8");
    case("item.js", "text/javascript;charset=utf-8");
    case("page.css", "text/css;charset=utf-8");
  }

  #[test]
  fn open_graph_image_fallback() {
    let fallback = OpenGraphImage::fallback();

    let image = ::image::load_from_memory(
      &StaticAsset::get(fallback.path.strip_prefix("static/").unwrap())
        .unwrap()
        .content,
    )
    .unwrap();

    assert_eq!(
      Dimensions {
        height: image.height().into(),
        width: image.width().into(),
      },
      fallback.dimensions,
    );
  }
}
