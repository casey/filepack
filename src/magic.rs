use super::*;

pub trait Magic {
  const BYTES: MagicBytes;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn magic_bytes() {
    #[track_caller]
    fn case<T: Magic>() {
      let magic_bytes = T::BYTES.encode_to_vec();
      assert!(str::from_utf8(&magic_bytes).is_err());
      assert!(magic_bytes.starts_with(&[0x92]));
      assert!(magic_bytes.ends_with(&[0]));
      assert!(magic_bytes[1..].starts_with(b"filepack-"));
    }
    case::<Archive>();
    case::<Metadata>();
  }
}
