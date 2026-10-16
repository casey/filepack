use std::{fs, process::Command};

#[test]
fn summary() {
  let tempdir = tempfile::tempdir().unwrap();

  let status = Command::new(env!("CARGO_BIN_EXE_generate"))
    .current_dir(&tempdir)
    .status()
    .unwrap();

  assert!(status.success());

  assert_eq!(
    fs::read_to_string(tempdir.path().join("book/src/SUMMARY.md")).unwrap(),
    include_str!("../../../book/src/SUMMARY.md"),
  );
}
