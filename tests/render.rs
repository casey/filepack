use super::*;

#[test]
fn from_current_directory() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("render")
    .current_dir(&dir)
    .assert()
    .stdout(is_match("<!doctype html>.*"))
    .success();
}

#[test]
fn from_directory() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["render", "."])
    .current_dir(&dir)
    .assert()
    .stdout(is_match("<!doctype html>.*"))
    .success();
}

#[test]
fn from_file() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["render", "filepack.json"])
    .current_dir(&dir)
    .assert()
    .stdout(is_match("<!doctype html>.*"))
    .success();
}

#[test]
fn with_metadata() {
  let dir = TempDir::new().unwrap();

  dir.child("metadata.yaml").write_str("title: foo").unwrap();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "--metadata", "metadata.yaml", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["render", "foo"])
    .current_dir(&dir)
    .assert()
    .stdout(is_match("<!doctype html>.*<title>foo</title>.*"))
    .success();
}

#[test]
fn links_to_present_files() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["render", "foo"])
    .current_dir(&dir)
    .assert()
    .stdout(is_match(
      r#"<!doctype html>.*<td class=monospace><a href="bar">bar</a></td>.*"#,
    ))
    .success();
}

#[test]
fn does_not_link_to_missing_files() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  fs::remove_file(dir.child("foo/bar")).unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["render", "foo"])
    .current_dir(&dir)
    .assert()
    .stdout(is_match(
      r#"<!doctype html>.*<td class=monospace>bar</td>.*"#,
    ))
    .success();
}
