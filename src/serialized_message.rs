pub struct SerializedMessage(pub(crate) Vec<u8>);

impl SerializedMessage {
  pub(crate) fn bytes(&self) -> &[u8] {
    &self.0
  }
}
