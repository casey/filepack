use {
  assert_cmd::Command,
  assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteStr, PathChild},
    TempDir,
  },
};

const SEPARATOR: char = std::path::MAIN_SEPARATOR;

mod create;
mod verify;
