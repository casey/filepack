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
    fn og_image(&self) -> Option<String> {
      Some("bar".into())
    }

    fn title(&self) -> String {
      "baz".into()
    }
  }

  #[test]
  fn og_image() {
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
              <meta property=og:image content='https://example.com/bar'>
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
