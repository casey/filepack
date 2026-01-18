use super::*;

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str =
  "private124p4zsr2nh04f4pkgtxfzv5yle473x4jue7s6lkwg9tdkk73q59qz34d70";

pub(crate) const PUBLIC_KEY: &str =
  "public167dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqdq9may";

pub(crate) const SIGNATURE: &str = "signature1gc6dglnp0v32znv204688sd05nekguae2p6ajhmpnqwqsqxxay76s88w7r32qqyf52u8y8hlu9crgjyeg2jamru7kswmqq3j0npfjzglmt8d2";

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9xa2lj";

pub(crate) const WEAK_SIGNATURE: &str = "signature1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq83s9ss";

#[test]
fn hash_is_valid() {
  HASH.parse::<Hash>().unwrap();
}

#[test]
fn private_key_is_valid() {
  PRIVATE_KEY.parse::<PrivateKey>().unwrap();
}

#[test]
fn public_key_is_valid() {
  PUBLIC_KEY.parse::<PublicKey>().unwrap();
}

#[test]
fn signature_is_valid() {
  SIGNATURE.parse::<Signature>().unwrap();
}

#[test]
fn signature_matches() {
  let private_key = PRIVATE_KEY.parse::<PrivateKey>().unwrap();
  let signature = private_key.sign(Digest(HASH.parse().unwrap()));
  assert_eq!(signature.to_string(), SIGNATURE);
}
