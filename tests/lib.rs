use {
  assert_cmd::Command,
  assert_fs::{
    TempDir,
    assert::PathAssert,
    fixture::{FileTouch, FileWriteBin, FileWriteStr, PathChild, PathCreateDir},
  },
  camino::Utf8Path,
  filepack::{Manifest, PublicKey, Signature},
  predicates::str::RegexPredicate,
  std::{fs, path::Path, str},
};

fn path(message: &str) -> String {
  message.replace('/', std::path::MAIN_SEPARATOR_STR)
}

fn is_match<S>(pattern: S) -> RegexPredicate
where
  S: AsRef<str>,
{
  predicates::prelude::predicate::str::is_match(format!("^(?s){}$", pattern.as_ref())).unwrap()
}

fn load_key(path: &Path) -> String {
  fs::read_to_string(path).unwrap().trim().into()
}

fn load_manifest(path: &Path) -> Manifest {
  let utf8_path = Utf8Path::from_path(path).unwrap();
  Manifest::load(Some(utf8_path)).unwrap().1
}

fn save_manifest(manifest: &Manifest, path: &Path) {
  let utf8_path = Utf8Path::from_path(path).unwrap();
  manifest.save(utf8_path).unwrap();
}

mod create;
mod fingerprint;
mod hash;
mod key;
mod keygen;
mod man;
mod misc;
mod sign;
mod verify;
