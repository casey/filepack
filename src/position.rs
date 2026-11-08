pub(crate) enum Position {
  Combined(&'static str),
  Separate {
    number: &'static str,
    total: &'static str,
  },
}

impl Position {
  pub(crate) fn tuple(&self) -> (&'static str, &'static str) {
    match *self {
      Self::Combined(name) => (name, name),
      Self::Separate { number, total } => (number, total),
    }
  }
}
