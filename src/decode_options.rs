#[derive(Clone, Copy, Default)]
pub struct DecodeOptions {
  pub(crate) strict: bool,
}

impl DecodeOptions {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn strict() -> Self {
    Self { strict: true }
  }
}
