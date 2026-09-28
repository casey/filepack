use {
  super::*,
  clap::error::{ContextKind, ContextValue, ErrorKind},
};

pub(crate) const FINGERPRINT: &str =
  "package1af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const HASH: &str =
  "hash1af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str = concat!(
  "private1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8554351406a9ddf54d43642",
  "cc913284fe6be89ab2e67d0d7ece4156db5bd1050a",
);

pub(crate) const PUBLIC_KEY: &str =
  "public1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8";

pub(crate) const REVISION: &str =
  "revision1a895f0cd0338254bc76650e829441165eb127ae1e51b92e87189ed1580195617";

pub(crate) const SIGNATURE: &str = concat!(
  "signature101a0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc802a201a0af1349b9",
  "f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f326203c067bf3fe66d78dc9d75200161c0880b4d23",
  "e0f14bb2b535bf3274b1a5b267a0ea42008cf0ce9a46ed1bf9e4f91b1e4437687d0b7937c0d1f67b445a91f5b48a07",
);

pub(crate) const TOKEN: &str = concat!(
  "token101a0d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc802870183666f6f020003",
  "c06d3cf670eeaaa03ff933fca86543edf4f715ff49a27febf887b31442069b3af422b6fc651271d310ed2fd8a9eb0d",
  "89dcaebc41598affe8d3d5f7572b06294a08",
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
      audience: "foo".into(),
      timestamp: 0,
    };
    let token = private_key.sign(claims);
    assert_eq!(token.to_string(), TOKEN);
  }
}
