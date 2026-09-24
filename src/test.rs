use {
  super::*,
  clap::error::{ContextKind, ContextValue, ErrorKind},
};

pub(crate) const FINGERPRINT: &str =
  "package1af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str = concat!(
  "private1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8554351406a9ddf54d43642",
  "cc913284fe6be89ab2e67d0d7ece4156db5bd1050a",
);

pub(crate) const PUBLIC_KEY: &str =
  "public1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8";

pub(crate) const REVISION: &str =
  "revision14d6e1438411c7f6947bcb9bcfd5e74cf7077d2b3ccf24085cc2016afc7f0306c";

pub(crate) const SIGNATURE: &str = concat!(
  "signature1000001a0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc802a4000001a0",
  "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f326203c0b94281b2638e52cfecb6c18485",
  "6ab482080c21f451c816f3b3be4313b0ba7fbab6128bf8a9045de1d2b5e5a3a86dba2a562b7cfea6d66eb4273a0fc1",
  "4c8a9e04",
);

pub(crate) const TOKEN: &str = concat!(
  "token1000001a0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8028900000183666f",
  "6f020003c0290c82aa8dbed1c9f020c8be2f0320fc319412d590c7e9cf345bb9893c861ccd57e9b54eaa1b17c96f9f",
  "8900443f37d2f250b3643fd632c44dabcebe078fd702",
);

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public10000000000000000000000000000000000000000000000000000000000000000";

#[track_caller]
pub(crate) fn assert_argument_conflict<T: Parser>(args: &[&str], argument: &str, conflict: &str) {
  let error = T::try_parse_from(["filepack"].iter().chain(args))
    .map(drop)
    .unwrap_err();
  assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
  assert_eq!(
    error.get(ContextKind::InvalidArg),
    Some(&ContextValue::String(argument.into())),
  );
  assert_eq!(
    error.get(ContextKind::PriorArg),
    Some(&ContextValue::String(conflict.into())),
  );
}

#[track_caller]
pub(crate) fn assert_deco<T: Debug + DecodeOwned + Encode + PartialEq>(value: T, deco: &str) {
  let buffer = value.encode_to_vec();
  assert_eq!(hex::encode(&buffer), deco);
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_deco_eq<T: Debug + DecodeOwned + Encode + PartialEq>(
  value: T,
  expected: impl Encode,
) {
  assert_deco(value, &hex::encode(&expected.encode_to_vec()));
}

#[track_caller]
pub(crate) fn assert_encoding<T: Debug + DecodeOwned + Encode + PartialEq>(value: T) {
  let buffer = value.encode_to_vec();
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_invalid_argument_value<T: Parser>(
  args: &[&str],
  argument: &str,
  value: &str,
  message: &str,
) {
  let error = T::try_parse_from(["filepack"].iter().chain(args))
    .map(drop)
    .unwrap_err();
  assert_eq!(error.kind(), ErrorKind::ValueValidation);
  assert_eq!(
    error.get(ContextKind::InvalidArg),
    Some(&ContextValue::String(argument.into())),
  );
  assert_eq!(
    error.get(ContextKind::InvalidValue),
    Some(&ContextValue::String(value.into())),
  );
  assert_eq!(
    std::error::Error::source(&error).unwrap().to_string(),
    message,
  );
}

#[track_caller]
pub(crate) fn assert_missing_argument<T: Parser>(args: &[&str], missing: &[&str]) {
  let error = T::try_parse_from(["filepack"].iter().chain(args))
    .map(drop)
    .unwrap_err();
  assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
  assert_eq!(
    error.get(ContextKind::InvalidArg),
    Some(&ContextValue::Strings(
      missing.iter().map(ToString::to_string).collect()
    )),
  );
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

pub(crate) fn with_unknown_field(value: impl Encode) -> Vec<u8> {
  let bytes = value.encode_to_vec();
  let mut fields = BTreeMap::<u64, Vec<u8>>::decode_from_slice(&bytes).unwrap();
  assert!(fields.insert(u64::MAX, b"foo".to_vec()).is_none());
  fields.encode_to_vec()
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
  fn revision_is_valid() {
    assert_eq!(
      test::REVISION.parse::<Revision>().unwrap().to_string(),
      test::REVISION,
    );
  }

  #[test]
  fn signature_matches() {
    let private_key = PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let statement = Statement {
      version: Version::Zero,
      fingerprint: FINGERPRINT.parse().unwrap(),
      timestamp: None,
    };
    let signature = private_key.sign(statement);
    assert_eq!(signature.to_string(), SIGNATURE);
  }

  #[test]
  fn token_matches() {
    let private_key = PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let claims = Claims {
      version: Version::Zero,
      audience: "foo".into(),
      timestamp: 0,
    };
    let token = private_key.sign(claims);
    assert_eq!(token.to_string(), TOKEN);
  }
}
