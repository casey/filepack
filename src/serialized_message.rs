pub struct SerializedMessage(pub(crate) Vec<u8>);

impl AsRef<[u8]> for SerializedMessage {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}
