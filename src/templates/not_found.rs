use super::*;

#[derive(Boilerplate)]
pub(crate) struct NotFoundHtml;

impl Page for NotFoundHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/not-found.css")
  }

  fn title(&self) -> String {
    "not found · filepack".into()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn render() {
    assert_eq!(NotFoundHtml.to_string(), "<div><a href=/>?</a></div>\n");
  }
}
