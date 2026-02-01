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
  "h4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3qsvczv268s6lkxtsc0eufqc5xz3g99640gtpwadk349d8f",
  "qkjgl3tkp2m95ujz9arxzwt74ggzd3f9vnc6skcns9kn6xnxuqz6v26yrgw3lv6u",
);

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public1aqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsqtuc8";

pub(crate) fn checksum(s: &str) -> String {
  let checked_hrpstring = CheckedHrpstring::new::<bech32::NoChecksum>(s).unwrap();
  checked_hrpstring
    .fe32_iter::<std::vec::IntoIter<u8>>()
    .with_checksum::<bech32::Bech32m>(&checked_hrpstring.hrp())
    .chars()
    .collect()
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
  let message = Message {
    fingerprint: FINGERPRINT.parse().unwrap(),
    timestamp: None,
  };
  let signature = private_key.sign(&message);
  assert_eq!(signature.to_string(), SIGNATURE);
}
