use super::*;

pub trait Page: Display + Sized {
  fn og_image(&self) -> Option<String> {
    None
  }

  fn page(self, base: Option<Url>) -> PageHtml<Self> {
    PageHtml {
      base,
      content: self,
    }
  }

  fn stylesheet(&self) -> Option<&'static str> {
    None
  }

  fn title(&self) -> String;
}
