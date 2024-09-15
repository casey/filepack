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

  pub(crate) fn effective_style(self) -> owo_colors::Style {
    if self.is_terminal {
      self.inner
    } else {
      owo_colors::Style::default()
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
