use {
  self::{child::Child, dedent::Dedent, expected::Expected, test::Test},
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{
    Archive, Decode, Decoder, Directory, DirectoryExt, Encode, Encoder, Entry, Fingerprint,
    FlacBuilder, Hash, Manifest, Metadata, Mp3Builder, Mp4Builder, Page, PngBuilder, PrivateKey,
    PublicKey, Totals, gradient, gradient_alpha, hex,
    templates::{DirectoryHtml, PackageHtml},
  },
  image::{DynamicImage, ImageFormat},
  regex::Regex,
  reqwest::StatusCode,
  std::{
    assert_matches,
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
    net::TcpListener,
    path::{MAIN_SEPARATOR_STR, Path},
    process::{Command, Stdio},
    str,
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
  },
  tempfile::{NamedTempFile, TempDir},
  unindent::unindent,
  usized::IntoU64,
};

mod archive;
mod child;
mod contains;
mod create;
mod data_dir;
mod dedent;
mod delete;
mod download;
mod expected;
mod files;
mod fingerprint;
mod gc;
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
mod serve;
mod sign;
mod signatures;
mod size;
mod test;
mod upload;
mod verify;

const EMPTY_HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

const PRIVATE_KEY: &str = concat!(
  "private1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8554351406a9ddf54d43642",
  "cc913284fe6be89ab2e67d0d7ece4156db5bd1050a",
);

const PUBLIC_KEY: &str = "public1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8";

const USAGE_ERROR: i32 = 2;

fn fingerprint(path: &Utf8Path) -> Fingerprint {
  Archive::load(path).unwrap().fingerprint().unwrap()
}

fn tempdir() -> TempDir {
  tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap()
}
