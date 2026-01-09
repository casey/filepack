use super::*;

pub(crate) struct FieldHasher {
  array: Option<NonZeroU64>,
  hasher: Hasher,
  next: u64,
}

impl FieldHasher {
  pub(crate) fn array(&mut self, tag: u64, len: u64) {
    self.tag(tag);
    self.array = NonZeroU64::new(len);
    self.integer(len);
  }

  pub(crate) fn element(&mut self, hash: Hash) {
    self.array = NonZeroU64::new(self.array.unwrap().get() - 1);
    self.hasher.update(hash.as_bytes());
  }

  pub(crate) fn field(&mut self, tag: u64, contents: &[u8]) {
    self.tag(tag);
    self.integer(contents.len().into_u64());
    self.hasher.update(contents);
  }

  pub(crate) fn finalize(self) -> Hash {
    assert_eq!(self.array, None);
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
    assert_eq!(self.next, tag);
    assert_eq!(self.array, None);
    self.next = self.next.checked_add(1).unwrap();
    self.integer(tag);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[should_panic]
  fn tag_order_field() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.field(1, &[]);
  }

  #[test]
  #[should_panic]
  fn tag_order_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.array(1, 1);
  }

  #[test]
  #[should_panic]
  fn element_out_of_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.element(Hash::from([0; 32]));
  }

  #[test]
  #[should_panic]
  fn unfinished_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.array(0, 1);
    hasher.finalize();
  }

  #[test]
  #[should_panic]
  fn field_in_array() {
    let mut hasher = FieldHasher::new(Context::File);
    hasher.array(0, 1);
    hasher.field(1, &[]);
  }
}
