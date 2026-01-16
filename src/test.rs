use super::*;

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str =
  "8eb33440cce2a651c6d8867c331392f642ebfd9b96e485cd2124643461fb41a2";

pub(crate) const PUBLIC_KEY: &str =
  "26892a0ef203b97c2702052336f2b8711efaf1442430ff0d9fb4d03df794a0ac";

pub(crate) const SIGNATURE: &str = concat!(
  "3f814a19e6db6431959f0393d362920846224af1d44ceee851e0caded9412d93",
  "9a221a15f6ba9a5d118a570a6b1cc48c95c7fb73581eeec1e33afdb4d0163907",
);

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
