use super::*;

pub trait Page: Display + Sized {
  fn next(&self) -> Option<String> {
    None
  }

  fn open_graph_description(&self) -> Option<String> {
    None
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    None
  }

  fn page(self, base: Option<Url>) -> PageHtml<Self> {
    PageHtml {
      base,
      content: self,
    }
  }

  fn prev(&self) -> Option<String> {
    None
  }

  fn script(&self) -> Option<&'static str> {
    None
  }

  fn stylesheet(&self) -> Option<&'static str> {
    None
  }

  fn title(&self) -> String;
}
