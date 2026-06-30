use super::*;

pub trait Page: Display + Sized {
  fn page(self) -> PageHtml<Self> {
    PageHtml { content: self }
  }

  fn stylesheet(&self) -> Option<&str> {
    None
  }

  fn title(&self) -> String;
}

impl<T: Page> From<T> for PageHtml<T> {
  fn from(content: T) -> PageHtml<T> {
    content.page()
  }
}
