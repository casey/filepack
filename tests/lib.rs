use {
  self::{
    child::Child, dedent::Dedent, expected::Expected, request::Request, stub_server::StubServer,
    test::Test,
  },
  axum::{body, response::IntoResponse},
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{
    Decode, DecodeOptions, Decoded, Decoder, Directory, DirectoryExt, Encode, Encoder, Entry,
    Fingerprint, FlacBuilder, Hash, Loader, MagicType, Manifest, Metadata, Mp3Builder, Mp4Builder,
    NonEmptyOrderedSet, PackageIdentifier, Page, PngBuilder, PrivateKey, PublicKey, Reference,
    Revision, ServerState, State, Totals, api, gradient, gradient_alpha, hex,
    templates::{DirectoryHtml, PackageHtml},
  },
  image::{DynamicImage, ImageFormat},
  regex::Regex,
  reqwest::{
    Method, StatusCode, Version,
    header::{self, HeaderName},
  },
  std::{
    assert_matches,
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, BufRead, BufReader, Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    path::{MAIN_SEPARATOR_STR, Path},
    process::{Command, Stdio},
    str,
    sync::{
      Arc, LazyLock,
      atomic::{AtomicBool, Ordering},
      mpsc,
    },
    thread,
    time::{Duration, Instant},
  },
  tempfile::{NamedTempFile, TempDir},
  tokio::runtime::Runtime,
  unindent::unindent,
  usized::IntoU64,
};

mod archive;
mod child;
mod completions;
mod contains;
mod create;
mod data_dir;
mod dedent;
mod delete;
mod download;
mod expected;
mod files;
mod fingerprint;
mod forget;
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
mod magic;
mod man;
mod manifest;
mod metadata;
mod misc;
mod print;
mod reference;
mod request;
mod revision;
mod serve;
mod sign;
mod signatures;
mod size;
mod state;
mod stub_server;
mod test;
mod update;
mod upload;
mod verify;

const EMPTY_HASH: &str = "hash1af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

const PRIVATE_KEY: &str = concat!(
  "private1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8554351406a9ddf54d43642",
  "cc913284fe6be89ab2e67d0d7ece4156db5bd1050a",
);

const PUBLIC_KEY: &str = "public1d79b36defbee7c1d26099c9e28f849f1f0dceb6e0217b83c87f5cff6fd8c4fc8";

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

fn add_parent(path: &Utf8Path) -> Revision {
  let fingerprint = fingerprint(path);

  let revision = Revision {
    package: fingerprint,
    parents: Some(NonEmptyOrderedSet::singleton(fingerprint.revision().hash())),
  };

  let mut manifest = manifest(path);
  manifest.parents = revision.parents.clone();
  manifest.save(path).unwrap();

  revision
}

fn add_unknown_field(path: &Utf8Path) -> Vec<u8> {
  let bytes = fs::read(path).unwrap();
  let mut decoder = Decoder::new(&bytes);
  <&[u8]>::decode(&mut decoder).unwrap();
  <&[u8]>::decode(&mut decoder).unwrap();
  let mut fields = BTreeMap::<u64, Vec<u8>>::decode(&mut decoder).unwrap();
  assert!(fields.insert(u64::MAX, b"foo".to_vec()).is_none());
  let mut encoder = Encoder::new();
  fields.encode(&mut encoder);
  encoder.magic(MagicType::Archive);
  let bytes = encoder.finish();
  fs::write(path, &bytes).unwrap();
  bytes
}

fn add_unknown_signature(path: &Utf8Path) -> Vec<u8> {
  let mut manifest = manifest(path);
  let signature = manifest
    .signatures
    .iter()
    .find_map(Decoded::known)
    .unwrap()
    .encode_to_vec();
  let mut fields = BTreeMap::<u64, Vec<u8>>::decode_from_slice(&signature).unwrap();
  assert!(fields.insert(4, b"foo".to_vec()).is_none());
  let payload = Vec::<u8>::decode_from_slice(&fields.encode_to_vec()).unwrap();
  assert!(
    manifest
      .signatures
      .insert(Decoded::Unknown(payload.clone()))
  );
  manifest.save(path).unwrap();
  payload
}

fn fingerprint(path: &Utf8Path) -> Fingerprint {
  Loader::load_with_options(DecodeOptions::strict(), Some(path))
    .unwrap()
    .fingerprint()
    .unwrap()
}

fn manifest(path: &Utf8Path) -> Manifest {
  Manifest::load_with_options(DecodeOptions::strict(), Some(path)).unwrap()
}

fn signed_revision() -> (Revision, PublicKey, BTreeMap<u64, Vec<u8>>) {
  let test = Test::new()
    .arg("keygen")
    .success()
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success();

  let path = test.path().join("foo/manifest.filepack");

  let manifest = manifest(&path);

  let signature = manifest.signatures.first().unwrap().known().unwrap();

  (
    fingerprint(&path).revision(),
    signature.public_key(),
    BTreeMap::decode_from_slice(&signature.encode_to_vec()).unwrap(),
  )
}

fn tempdir() -> TempDir {
  tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap()
}
