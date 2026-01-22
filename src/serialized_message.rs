pub struct SerializedMessage(pub(crate) Vec<u8>);

impl SerializedMessage {
  #[must_use]
  pub fn as_bytes(&self) -> &[u8] {
    &self.0
  }
}
