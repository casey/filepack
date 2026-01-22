use super::*;

#[allow(dead_code)]
#[track_caller]
pub(crate) fn bech32m_placeholder(s: &str) -> String {
  use bech32::{Bech32m, primitives::decode::UncheckedHrpstring};

  let unchecked = UncheckedHrpstring::new(s)
    .unwrap()
    .remove_checksum::<Bech32m>();

  let hrp = unchecked.hrp();

  let data = unchecked
    .data_part_ascii_no_checksum()
    .iter()
    .map(|c| bech32::Fe32::from_char(char::from(*c)).unwrap());

  let encoded = data
    .with_checksum::<Bech32m>(&hrp)
    .chars()
    .collect::<String>();

  panic!("bech32m_placeholder: {encoded}");
}

pub(crate) const FINGERPRINT: &str =
  "package1a4uf5nw04lxs6dgzqfh4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3ql7ykr3";

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str =
  "private1a24p4zsr2nh04f4pkgtxfzv5yle473x4jue7s6lkwg9tdkk73q59qluezpp";

pub(crate) const PUBLIC_KEY: &str =
  "public1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqcxnd63";

pub(crate) const SIGNATURE: &str = concat!(
  "signature1af0qppampjlm7qs0g4amn9fnq87crhn70k5lv5wf48ajy6k774tq",
  "w6yc9s5n0kpq5420jrz644sgu7geahpffl8l7nuv9azsqv8jpgtrcqstq8xwz",
);

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public1aqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsqtuc8";

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
fn public_key_is_valid() {
  assert_eq!(
    test::PUBLIC_KEY.parse::<PublicKey>().unwrap().to_string(),
    test::PUBLIC_KEY,
  );
}

#[test]
fn signature_is_valid() {
  assert_eq!(
    test::SIGNATURE.parse::<Signature>().unwrap().to_string(),
    test::SIGNATURE,
  );
}

#[test]
fn signature_matches() {
  let private_key = PRIVATE_KEY.parse::<PrivateKey>().unwrap();
  let message = Message {
    fingerprint: FINGERPRINT.parse().unwrap(),
    time: None,
  };
  let signature = private_key.sign(&message.serialize());
  assert_eq!(signature.to_string(), SIGNATURE);
}
