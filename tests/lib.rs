use {
  assert_cmd::Command,
  assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteStr, PathChild, PathCreateDir},
    TempDir,
  },
  predicates::prelude::predicate,
};

const SEPARATOR: char = std::path::MAIN_SEPARATOR;

mod create;
mod misc;
mod verify;
