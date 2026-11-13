use {
  super::*,
  clap::error::{ContextKind, ContextValue, ErrorKind},
};

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
  assert_eq!(T::decode_strict(&buffer).unwrap(), value);
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
  assert_eq!(T::decode_strict(&value.encode_to_vec()).unwrap(), value);
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
