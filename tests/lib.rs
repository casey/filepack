use {
  assert_cmd::Command,
  assert_fs::{
    assert::PathAssert,
    fixture::{FileTouch, FileWriteStr, PathChild},
    TempDir,
  },
  std::error::Error,
};

type Result = std::result::Result<(), Box<dyn Error>>;

const SEPARATOR: char = std::path::MAIN_SEPARATOR;

mod create;
mod verify;
