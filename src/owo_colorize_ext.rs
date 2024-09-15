use super::*;

pub(crate) trait OwoColorizeExt {
  fn style(&self, style: Style) -> Styled<&Self>;
}

impl<T: owo_colors::OwoColorize> OwoColorizeExt for T {
  fn style(&self, style: Style) -> Styled<&Self> {
    self.style(style.effective_style())
  }
}
