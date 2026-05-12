use {
  self::{child::Child, expected::Expected, test::Test},
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{Encoder, Hash, Manifest, PrivateKey, PublicKey, assert_matches},
  regex::Regex,
  std::{
    collections::BTreeMap,
    fs,
    io::{self, Read, Write},
    os::{fd::AsRawFd, unix::process::CommandExt},
    path::{MAIN_SEPARATOR_STR, Path},
    process::{Command, Stdio},
    str,
    time::{SystemTime, UNIX_EPOCH},
  },
  tempfile::{NamedTempFile, TempDir},
};

mod archive;
mod bech32;
mod child;
mod contains;
mod create;
mod data_dir;
mod download;
mod expected;
mod files;
mod fingerprint;
mod hash;
mod info;
mod json;
mod key;
mod keychain;
mod keygen;
mod languages;
mod lint;
mod lints;
mod man;
mod manifest;
mod metadata;
mod misc;
mod sign;
mod signatures;
mod size;
mod test;
mod upload;
mod verify;

fn tempdir() -> TempDir {
  tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap()
}

const EMPTY_HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

const PRIVATE_KEY: &str = "private1a24p4zsr2nh04f4pkgtxfzv5yle473x4jue7s6lkwg9tdkk73q59qluezpp";

const PUBLIC_KEY: &str = "public1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqcxnd63";
