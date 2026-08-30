use super::*;

#[derive(Boilerplate)]
pub struct PageHtml<T: Page> {
  pub(crate) base: Option<Url>,
  pub(crate) content: T,
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  struct HomePage;

  impl Display for HomePage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(f, "foo")
    }
  }

  impl Page for HomePage {
    fn home(&self) -> bool {
      true
    }

    fn title(&self) -> String {
      "home".into()
    }
  }

  struct ImagePage;

  impl Display for ImagePage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(f, "foo")
    }
  }

  impl Page for ImagePage {
    fn open_graph_description(&self) -> Option<String> {
      Some("qux".into())
    }

    fn open_graph_image(&self) -> Option<OpenGraphImage> {
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        path: "bar".into(),
      })
    }

    fn title(&self) -> String {
      "image".into()
    }
  }

  struct NavigationPage;

  impl Display for NavigationPage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(f, "bar")
    }
  }

  impl Page for NavigationPage {
    fn next(&self) -> Option<String> {
      Some("/foo".into())
    }

    fn prev(&self) -> Option<String> {
      Some("/bar".into())
    }

    fn title(&self) -> String {
      "navigation".into()
    }

    fn up(&self) -> Option<String> {
      Some("/baz".into())
    }
  }

  struct ScriptPage;

  impl Display for ScriptPage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
      write!(f, "bar")
    }
  }

  impl Page for ScriptPage {
    fn script(&self) -> Option<&'static str> {
      Some("/foo.js")
    }

    fn title(&self) -> String {
      "script".into()
    }
  }

  #[test]
  fn home() {
    assert_eq!(
      HomePage.page(None).to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>home</title>
              <meta name=description content='Filepack package server'>
              <meta name=google content=notranslate>
              <meta property=og:site_name content=Filepack>
              <link href=/static/page.css rel=stylesheet>
            </head>
            <body>
              <header>
                <nav>
                  <a>Filepack</a>
                  <a href=https://github.com/casey/filepack>GitHub</a>
                </nav>
              </header>
              <main>
                foo
              </main>
            </body>
          </html>
        "
      ),
    );
  }

  #[test]
  fn navigation() {
    assert_eq!(
      NavigationPage.page(None).to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>navigation</title>
              <meta name=description content='Filepack package server'>
              <meta name=google content=notranslate>
              <meta property=og:site_name content=Filepack>
              <link href=/static/page.css rel=stylesheet>
              <link href=/foo rel=next>
              <link href=/bar rel=prev>
              <link href=/baz rel=up>
            </head>
            <body>
              <header>
                <nav>
                  <a href=/>Filepack</a>
                  <a href=https://github.com/casey/filepack>GitHub</a>
                </nav>
              </header>
              <main>
                bar
              </main>
            </body>
          </html>
        "
      ),
    );
  }

  #[test]
  fn open_graph_image() {
    assert_eq!(
      ImagePage
        .page(Some("https://example.com".parse().unwrap()))
        .to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>image</title>
              <meta name=description content='Filepack package server'>
              <meta name=google content=notranslate>
              <meta property=og:description content='qux'>
              <meta property=og:image content='https://example.com/bar'>
              <meta property=og:image:height content=1>
              <meta property=og:image:width content=2>
              <meta property=og:site_name content=Filepack>
              <link href=/static/page.css rel=stylesheet>
            </head>
            <body>
              <header>
                <nav>
                  <a href=/>Filepack</a>
                  <a href=https://github.com/casey/filepack>GitHub</a>
                </nav>
              </header>
              <main>
                foo
              </main>
            </body>
          </html>
        "
      ),
    );

    assert_eq!(
      ImagePage.page(None).to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>image</title>
              <meta name=description content='Filepack package server'>
              <meta name=google content=notranslate>
              <meta property=og:description content='qux'>
              <meta property=og:site_name content=Filepack>
              <link href=/static/page.css rel=stylesheet>
            </head>
            <body>
              <header>
                <nav>
                  <a href=/>Filepack</a>
                  <a href=https://github.com/casey/filepack>GitHub</a>
                </nav>
              </header>
              <main>
                foo
              </main>
            </body>
          </html>
        "
      ),
    );
  }

  #[test]
  fn script() {
    assert_eq!(
      ScriptPage.page(None).to_string(),
      unindent(
        "
          <!doctype html>
          <html lang=en>
            <head>
              <meta charset=utf-8>
              <meta name=viewport content='width=device-width,initial-scale=1.0'>
              <title>script</title>
              <meta name=description content='Filepack package server'>
              <meta name=google content=notranslate>
              <meta property=og:site_name content=Filepack>
              <link href=/static/page.css rel=stylesheet>
              <script src=/foo.js type=module></script>
            </head>
            <body>
              <header>
                <nav>
                  <a href=/>Filepack</a>
                  <a href=https://github.com/casey/filepack>GitHub</a>
                </nav>
              </header>
              <main>
                bar
              </main>
            </body>
          </html>
        "
      ),
    );
  }
}
