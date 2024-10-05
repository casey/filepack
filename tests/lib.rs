use {
  assert_cmd::Command,
  assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteBin, FileWriteStr, PathChild, PathCreateDir},
    TempDir,
  },
  predicates::str::RegexPredicate,
  std::{fs, str},
};

const SEPARATOR: char = if cfg!(windows) { '\\' } else { '/' };
const SEPARATOR_RE: &str = if cfg!(windows) { r"\\" } else { "/" };

fn is_match<S>(pattern: S) -> RegexPredicate
where
  S: AsRef<str>,
{
  predicates::prelude::predicate::str::is_match(format!("^(?s){}$", pattern.as_ref())).unwrap()
}

mod create;
mod hash;
mod key;
mod keygen;
mod man;
mod misc;
mod verify;
