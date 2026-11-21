use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Style {
  color: bool,
  inner: owo_colors::Style,
}

impl Style {
  pub(crate) fn bad(self) -> Self {
    Self {
      inner: self.inner.red(),
      ..self
    }
  }

  pub(crate) fn color(self) -> bool {
    self.color
  }

  pub(crate) fn color_choice() -> ColorChoice {
    fn set(key: &str) -> bool {
      env::var_os(key).is_some_and(|value| !value.is_empty())
    }

    if set("NO_COLOR") {
      ColorChoice::Never
    } else if set("FORCE_COLOR") {
      ColorChoice::Always
    } else {
      ColorChoice::Auto
    }
  }

  pub(crate) fn effective_style(self) -> owo_colors::Style {
    if self.color {
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

  pub(crate) fn new(color: bool) -> Self {
    Self {
      color,
      inner: owo_colors::Style::default(),
    }
  }

  pub(crate) fn stderr() -> Self {
    Self::stream(io::stderr())
  }

  pub(crate) fn stdout() -> Self {
    Self::stream(io::stdout())
  }

  fn stream(stream: impl IsTerminal) -> Self {
    match Self::color_choice() {
      ColorChoice::Always => Self::new(true),
      ColorChoice::Auto => Self::new(stream.is_terminal()),
      ColorChoice::Never => Self::new(false),
    }
  }

  pub(crate) fn warning(self) -> Self {
    Self {
      inner: self.inner.yellow(),
      ..self
    }
  }
}
