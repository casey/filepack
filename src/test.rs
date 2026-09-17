use super::*;

pub(crate) const FINGERPRINT: &str =
  "package1a0af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str = concat!(
  "private1c0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8",
  "554351406a9ddf54d43642cc913284fe6be89ab2e67d0d7ece4156db5bd1050a",
);

pub(crate) const PUBLIC_KEY: &str =
  "public1a0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8";

pub(crate) const SIGNATURE: &str = concat!(
  "signature1f08800a0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8",
  "01a200a0af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
  "02c03fd90842958b5349abc421dffce4950da98669631ed6cff2ce9f66b05d119e94",
  "9454b61ca79545e2b4899fc72cc531f874f47cc42db8c960df13f7db63e2ca0c",
);

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public1a00000000000000000000000000000000000000000000000000000000000000000";

#[track_caller]
pub(crate) fn assert_deco<T: Debug + Decode + Encode + PartialEq>(value: T, deco: &str) {
  let buffer = value.encode_to_vec();
  assert_eq!(hex::encode(&buffer), deco);
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_deco_eq<T: Debug + Decode + Encode + PartialEq>(
  value: T,
  expected: impl Encode,
) {
  assert_deco(value, &hex::encode(&expected.encode_to_vec()));
}

#[track_caller]
pub(crate) fn assert_encoding<T: Debug + Decode + Encode + PartialEq>(value: T) {
  let buffer = value.encode_to_vec();
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_redb_impls<K>(values: &[K])
where
  K: for<'a> redb::Value<SelfType<'a> = K> + redb::Key + Ord,
{
  for value in values {
    let bytes = K::as_bytes(value);

    if let Some(width) = K::fixed_width() {
      assert_eq!(bytes.as_ref().len(), width);
    }

    assert_eq!(K::from_bytes(bytes.as_ref()), *value);
  }

  for a in values {
    for b in values {
      assert_eq!(
        K::compare(K::as_bytes(a).as_ref(), K::as_bytes(b).as_ref()),
        a.cmp(b),
        "{a:?} vs {b:?}",
      );
    }
  }
}

pub(crate) fn exif(orientation: u16) -> Vec<u8> {
  let mut bytes = b"II".to_vec();

  bytes.extend_from_slice(&42u16.to_le_bytes());
  bytes.extend_from_slice(&8u32.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&0x0112u16.to_le_bytes());
  bytes.extend_from_slice(&3u16.to_le_bytes());
  bytes.extend_from_slice(&1u32.to_le_bytes());
  bytes.extend_from_slice(&orientation.to_le_bytes());
  bytes.extend_from_slice(&[0; 2]);
  bytes.extend_from_slice(&0u32.to_le_bytes());

  bytes
}

pub(crate) fn tempdir() -> (TempDir, Utf8PathBuf) {
  let tempdir = tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap();

  let path = Utf8Path::from_path(tempdir.path()).unwrap().into();

  (tempdir, path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_is_valid() {
    HASH.parse::<Hash>().unwrap();
  }

  #[test]
  fn private_key_is_valid() {
    assert_eq!(
      test::PRIVATE_KEY
        .parse::<PrivateKey>()
        .unwrap()
        .display_private_key()
        .to_string(),
      test::PRIVATE_KEY,
    );
  }

  #[test]
  fn signature_matches() {
    let private_key = PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let statement = Statement {
      fingerprint: FINGERPRINT.parse().unwrap(),
      timestamp: None,
    };
    let signature = private_key.sign(&statement);
    assert_eq!(signature.to_string(), SIGNATURE);
  }
}
