use {
  assert_cmd::Command,
  assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteBin, FileWriteStr, PathChild, PathCreateDir},
    TempDir,
  },
  predicates::str::RegexPredicate,
  serde::{Deserialize, Serialize},
  std::{collections::BTreeMap, fs, path::Path, str},
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

mod create;
mod fingerprint;
mod hash;
mod key;
mod keygen;
mod man;
mod misc;
mod render;
mod sign;
mod verify;
