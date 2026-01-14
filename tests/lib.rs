use {
  self::test::Test,
  assert_cmd::cargo::cargo_bin_cmd,
  filepack::{Manifest, PublicKey, Signature},
  std::{fs, path::Path, str},
};

mod create;
mod files;
mod fingerprint;
mod hash;
mod json;
mod key;
mod keygen;
mod man;
mod misc;
mod sign;
mod size;
mod test;
mod verify;

const EMPTY_HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

fn load_key(path: &Path) -> String {
  fs::read_to_string(path).unwrap().trim().into()
}

fn path(message: &str) -> String {
  message.replace('/', std::path::MAIN_SEPARATOR_STR)
}
