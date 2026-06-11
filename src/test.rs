use super::*;

pub(crate) const FINGERPRINT: &str =
  "package1a4uf5nw04lxs6dgzqfh4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3ql7ykr3";

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str = concat!(
  "private1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyq24p4zsr2nh04f4pkgtxf",
  "zv5yle473x4jue7s6lkwg9tdkk73q59qxqurh4",
);

pub(crate) const PUBLIC_KEY: &str =
  "public1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqcxnd63";

pub(crate) const SIGNATURE: &str = concat!(
  "signature1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyq4uf5nw04lxs6dgzqf",
  "h4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3qqe5zmy0jwfe33a8rr70fk0zv8wgwuy7zqdmp6jdull0l6",
  "kjl9lcxsvmqjz2zqhn92j3enhg9r3gu922j84e54fthhz78anp6cg27wpcrcgx4r",
);

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public1aqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsqtuc8";

#[track_caller]
pub(crate) fn assert_cbor<T: Debug + Decode + Encode + PartialEq>(value: T, cbor: &str) {
  let buffer = value.encode_to_vec();
  assert_eq!(hex::encode(&buffer), cbor);
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_cbor_eq<T: Debug + Decode + Encode + PartialEq>(
  value: T,
  expected: impl Encode,
) {
  assert_cbor(value, &hex::encode(expected.encode_to_vec()));
}

#[track_caller]
pub(crate) fn assert_encoding<T: Debug + Decode + Encode + PartialEq>(value: T) {
  let buffer = value.encode_to_vec();
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

pub(crate) fn checksum(s: &str) -> String {
  let checked_hrpstring = CheckedHrpstring::new::<bech32::NoChecksum>(s).unwrap();
  checked_hrpstring
    .fe32_iter::<std::vec::IntoIter<u8>>()
    .with_checksum::<bech32::Bech32m>(&checked_hrpstring.hrp())
    .chars()
    .collect()
}

pub(crate) fn flac(comments: &[&str]) -> Vec<u8> {
  let mut bytes = b"fLaC".to_vec();

  bytes.push(if comments.is_empty() { 0x80 } else { 0x00 });
  bytes.extend_from_slice(&34u32.to_be_bytes()[1..]);
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&[0; 6]);
  bytes.extend_from_slice(&[0x0a, 0xc4, 0x42, 0xf0]);
  bytes.extend_from_slice(&[0; 20]);

  if !comments.is_empty() {
    let mut body = Vec::new();
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&u32::try_from(comments.len()).unwrap().to_le_bytes());

    for comment in comments {
      body.extend_from_slice(&u32::try_from(comment.len()).unwrap().to_le_bytes());
      body.extend_from_slice(comment.as_bytes());
    }

    bytes.push(0x84);
    bytes.extend_from_slice(&u32::try_from(body.len()).unwrap().to_be_bytes()[1..]);
    bytes.extend(body);
  }

  bytes
}

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
      .display_secret()
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

pub(crate) fn tempdir() -> (TempDir, Utf8PathBuf) {
  let tempdir = tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap();

  let path = Utf8Path::from_path(tempdir.path()).unwrap().into();

  (tempdir, path)
}
