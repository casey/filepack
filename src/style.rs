use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Style {
  inner: owo_colors::Style,
  is_terminal: bool,
}

impl Style {
  pub(crate) fn bad(self) -> Self {
    Self {
      inner: self.inner.red(),
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

  pub(crate) fn json_key(self) -> Self {
    Self {
      inner: self.inner.blue().bold(),
      ..self
    }
  }

  pub(crate) fn json_null(self) -> Self {
    Self {
      inner: self.inner.dimmed(),
      ..self
    }
  }

  pub(crate) fn json_string(self) -> Self {
    Self {
      inner: self.inner.green(),
      ..self
    }
  }

  pub(crate) fn lint(self) -> Self {
    Self {
      inner: self.inner.cyan(),
      ..self
    }
  }

  pub(crate) fn message(self) -> Self {
    Self {
      inner: self.inner.bold(),
      ..self
    }
  }

  pub(crate) fn new(is_terminal: bool) -> Self {
    Self {
      inner: owo_colors::Style::default(),
      is_terminal,
    }
  }

  pub(crate) fn stderr() -> Self {
    Self::new(io::stderr().is_terminal())
  }

  pub(crate) fn stdout() -> Self {
    Self::new(io::stdout().is_terminal())
  }

  pub(crate) fn warning(self) -> Self {
    Self {
      inner: self.inner.yellow(),
      ..self
    }
  }
}
