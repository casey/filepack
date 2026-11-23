use std::{fs, process::Command};

#[test]
fn clean() {
  let tempdir = tempfile::tempdir().unwrap();

  let status = Command::new("git")
    .args(["init", "--quiet"])
    .current_dir(&tempdir)
    .status()
    .unwrap();

  assert!(status.success());

  fs::write(tempdir.path().join(".gitignore"), "/book/src/foo\n").unwrap();

  let path = tempdir.path().join("book/src/foo/bar.md");

  fs::create_dir_all(path.parent().unwrap()).unwrap();

  fs::write(&path, "baz").unwrap();

  let status = Command::new(env!("CARGO_BIN_EXE_generate"))
    .current_dir(&tempdir)
    .status()
    .unwrap();

  assert!(status.success());

  assert!(!path.exists());
}
