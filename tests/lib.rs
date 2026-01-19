use {
  self::{expected::Expected, test::Test},
  camino::{Utf8Path, Utf8PathBuf},
  executable_path::executable_path,
  filepack::{Manifest, Note, PrivateKey, PublicKey, Signature, assert_matches},
  regex::Regex,
  std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{MAIN_SEPARATOR_STR, Path},
    process::{Command, Stdio},
    str,
    time::{SystemTime, UNIX_EPOCH},
  },
};

mod contains;
mod create;
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
mod metadata;
mod misc;
mod notes;
mod sign;
mod size;
mod test;
mod verify;

const EMPTY_HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
const PRIVATE_KEY: &str = "private124p4zsr2nh04f4pkgtxfzv5yle473x4jue7s6lkwg9tdkk73q59qz34d70";
const PUBLIC_KEY: &str = "public167dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqdq9may";
const SIGNATURE: &str = "signature1gc6dglnp0v32znv204688sd05nekguae2p6ajhmpnqwqsqxxay76s88w7r32qqyf52u8y8hlu9crgjyeg2jamru7kswmqq3j0npfjzglmt8d2";
