use super::*;

pub(crate) struct FieldHasher {
  array: Option<NonZeroU64>,
  hasher: Hasher,
  next: u64,
}

impl FieldHasher {
  pub(crate) fn array(&mut self, tag: u64, len: u64) {
    self.tag(tag);
    self.integer(len);
    self.array = NonZeroU64::new(len);
  }

  pub(crate) fn element(&mut self, hash: Hash) {
    self.array = NonZeroU64::new(self.array.expect("element outside of array").get() - 1);
    self.hasher.update(hash.as_bytes());
  }

  pub(crate) fn field(&mut self, tag: u64, hash: Hash) {
    self.tag(tag);
    self.hasher.update(hash.as_bytes());
  }

  pub(crate) fn finalize(self) -> Hash {
    assert_eq!(self.array, None, "unfinished array");
    self.hasher.finalize().into()
  }

  fn integer(&mut self, n: u64) {
    self.hasher.update(&n.to_le_bytes());
  }

  pub(crate) fn new(context: Context) -> Self {
    Self {
      array: None,
      hasher: Hasher::new_derive_key(&format!("filepack:{context}")),
      next: 0,
    }
  }

  fn tag(&mut self, tag: u64) {
    assert_eq!(self.next, tag, "unexpected tag {tag}");
    assert_eq!(self.array, None, "field in array");
    self.next = self.next.checked_add(1).unwrap();
    self.integer(tag);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[should_panic(expected = "unexpected tag 1")]
  fn tag_order_field() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.field(1, Hash::bytes(&[]));
  }

  #[test]
  #[should_panic(expected = "unexpected tag 1")]
  fn tag_order_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.array(1, 1);
  }

  #[test]
  #[should_panic(expected = "element outside of array")]
  fn element_outside_of_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.element(Hash::from([0; 32]));
  }

  #[test]
  #[should_panic(expected = "unfinished array")]
  fn unfinished_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.array(0, 1);
    hasher.finalize();
  }

  #[test]
  #[should_panic(expected = "field in array")]
  fn field_in_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.array(0, 1);
    hasher.field(1, Hash::bytes(&[]));
  }
}
