use super::*;

pub(crate) const BYTES: &[u8] = b"filepack\0";

pub trait Magic {
  const TYPE: MagicType;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn magic() {
    #[track_caller]
    fn case<T: Magic>(expected: &[u8]) {
      let mut encoder = Encoder::new();
      encoder.magic(T::TYPE);
      let magic = encoder.finish();
      assert_eq!(magic, expected);
      assert!(str::from_utf8(&magic).is_err());
      assert!(magic.contains(&0));
    }
    case::<Archive>(b"\x89filepack\0\x87archive");
    case::<Metadata>(b"\x89filepack\0\x88metadata");
  }
}
