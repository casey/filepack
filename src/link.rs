use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Link {
  code: bool,
  href: Option<String>,
  text: String,
  title: Option<String>,
}

impl Link {
  pub(crate) fn code(self) -> Self {
    Self { code: true, ..self }
  }

  pub(crate) fn href(self, href: impl Display) -> Self {
    Self {
      href: Some(href.to_string()),
      ..self
    }
  }

  pub(crate) fn new(text: impl Display) -> Self {
    Self {
      code: false,
      href: None,
      text: text.to_string(),
      title: None,
    }
  }

  pub(crate) fn title(self, title: impl Display) -> Self {
    Self {
      title: Some(title.to_string()),
      ..self
    }
  }
}

impl Display for Link {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    fn escape(f: &mut Formatter, s: &str) -> fmt::Result {
      boilerplate::Escape::escape(&s, f)
    }

    write!(f, "<a")?;

    if let Some(href) = &self.href {
      write!(f, " href='")?;
      escape(f, href)?;
      write!(f, "'")?;
    }

    if let Some(title) = &self.title {
      write!(f, " title='")?;
      escape(f, title)?;
      write!(f, "'")?;
    }

    write!(f, ">")?;

    if self.code {
      write!(f, "<code>")?;
    }

    escape(f, &self.text)?;

    if self.code {
      write!(f, "</code>")?;
    }

    write!(f, "</a>")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(link: Link, expected: &str) {
      assert_eq!(link.to_string(), expected);
    }

    case(Link::new("foo"), "<a>foo</a>");

    case(Link::new("foo").href("/bar"), "<a href='/bar'>foo</a>");

    case(
      Link::new("foo").href("/bar").code(),
      "<a href='/bar'><code>foo</code></a>",
    );

    case(
      Link::new("foo").href("/bar").title("baz"),
      "<a href='/bar' title='baz'>foo</a>",
    );

    case(
      Link::new("<").href("'").title("&"),
      "<a href='&apos;' title='&amp;'>&lt;</a>",
    );
  }
}
