use {
  self::{expected::Expected, test::Test},
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{Manifest, PrivateKey, PublicKey, Signature, assert_matches},
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

mod bech32;
mod contains;
mod create;
mod data_dir;
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

const PRIVATE_KEY: &str = "private1a24p4zsr2nh04f4pkgtxfzv5yle473x4jue7s6lkwg9tdkk73q59qluezpp";

const PUBLIC_KEY: &str = "public1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqcxnd63";

pub(crate) const SIGNATURE: &str = concat!(
  "signature1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyq4uf5nw04lxs6dgzqf",
  "h4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3qppampjlm7qs0g4amn9fnq87crhn70k5lv5wf48ajy6k77",
  "4tqw6yc9s5n0kpq5420jrz644sgu7geahpffl8l7nuv9azsqv8jpgtrcqsz79ak7",
);
