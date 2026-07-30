use super::*;

#[derive(Boilerplate)]
pub struct PageHtml<T: Page> {
  pub(crate) base: Option<Url>,
  pub(crate) content: T,
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  struct Foo;

  impl Display for Foo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(f, "foo")
    }
  }

  impl Page for Foo {
    fn open_graph_description(&self) -> Option<String> {
      Some("qux".into())
    }

    fn open_graph_image(&self) -> Option<OpenGraphImage> {
      Some(OpenGraphImage {
        height: 1,
        path: "bar".into(),
        width: 2,
      })
    }

    fn title(&self) -> String {
      "baz".into()
    }
  }

  #[test]
  fn open_graph_image() {
    assert_eq!(
      Foo
        .page(Some("https://example.com".parse().unwrap()))
        .to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>baz</title>
              <meta name=description content='Filepack package server'>
              <meta property=og:description content='qux'>
              <meta property=og:image content='https://example.com/bar'>
              <meta property=og:image:height content=1>
              <meta property=og:image:width content=2>
              <meta property=og:site_name content=filepack>
              <link href=/static/index.css rel=stylesheet>
            </head>
            <body>
              foo
            </body>
          </html>
        "
      ),
    );

    assert_eq!(
      Foo.page(None).to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>baz</title>
              <meta name=description content='Filepack package server'>
              <meta property=og:description content='qux'>
              <meta property=og:site_name content=filepack>
              <link href=/static/index.css rel=stylesheet>
            </head>
            <body>
              foo
            </body>
          </html>
        "
      ),
    );
  }
}
