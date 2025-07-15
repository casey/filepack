use {
  assert_cmd::Command,
  assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteBin, FileWriteStr, PathChild, PathCreateDir},
    TempDir,
  },
  blake3::Hash,
  executable_path::executable_path,
  predicates::str::RegexPredicate,
  serde::{Deserialize, Serialize},
  std::{
    collections::BTreeMap,
    fs, iter,
    net::TcpListener,
    path::Path,
    str, thread,
    time::{Duration, Instant},
  },
};

const SEPARATOR: char = if cfg!(windows) { '\\' } else { '/' };

fn is_match<S>(pattern: S) -> RegexPredicate
where
  S: AsRef<str>,
{
  predicates::prelude::predicate::str::is_match(format!("^(?s){}$", pattern.as_ref())).unwrap()
}

fn load_key(path: &Path) -> String {
  fs::read_to_string(path).unwrap().trim().into()
}

fn free_port() -> u16 {
  TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port()
}

#[derive(Deserialize, Serialize)]
struct Manifest {
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  files: BTreeMap<String, Entry>,
  #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
  signatures: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Entry {
  pub(crate) hash: String,
  pub(crate) size: u64,
}

impl Manifest {
  fn load(path: &Path) -> Self {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
  }

  fn store(&self, path: &Path) {
    fs::write(path, serde_json::to_string(self).unwrap()).unwrap();
  }
}

mod archive;
mod create;
mod fingerprint;
mod hash;
mod key;
mod keygen;
mod man;
mod misc;
mod render;
mod server;
mod sign;
mod verify;
