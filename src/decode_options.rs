#[derive(Clone, Copy, Default)]
pub struct DecodeOptions {
  pub(crate) strict: bool,
}

impl DecodeOptions {
  pub fn strict() -> Self {
    Self { strict: true }
  }
}
