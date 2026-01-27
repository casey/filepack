pub(crate) struct SerializedMessage(pub Vec<u8>);

impl SerializedMessage {
  pub(crate) fn as_bytes(&self) -> &[u8] {
    &self.0
  }
}
