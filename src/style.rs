use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Style {
  is_terminal: bool,
  inner: owo_colors::Style,
}

impl Style {
  pub(crate) fn bad(self) -> Self {
    Self {
      inner: self.inner.red(),
      ..self
    }
  }

  pub(crate) fn error(self) -> Self {
    Self {
      inner: self.inner.red(),
      ..self
    }
  }

  pub(crate) fn good(self) -> Self {
    Self {
      inner: self.inner.green(),
      ..self
    }
  }

  pub(crate) fn message(self) -> Self {
    Self {
      inner: self.inner.bold(),
      ..self
    }
  }

  pub(crate) fn stderr() -> Self {
    Self {
      is_terminal: io::stderr().is_terminal(),
      inner: owo_colors::Style::default(),
    }
  }
}

pub(crate) trait OwoColorizeExt {
  fn style(&self, style: Style) -> Styled<&Self>;
}

impl<T: owo_colors::OwoColorize> OwoColorizeExt for T {
  fn style(&self, style: Style) -> Styled<&Self> {
    if style.is_terminal {
      self.style(style.inner)
    } else {
      self.style(owo_colors::Style::default())
    }
  }
}
