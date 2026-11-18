use super::*;

#[test]
fn revision() {
  Test::new()
    .arg("create")
    .success()
    .arg("revision")
    .stdout("revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583\n")
    .success();
}

#[test]
fn stored_revision_is_printed() {
  let test = Test::new().arg("create").success();

  let revision = add_parent(&test.path().join("manifest.filepack"));

  test
    .arg("revision")
    .stdout(format!("{}\n", revision.hash()))
    .success();
}

#[test]
fn with_path() {
  Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .args(["revision", "foo"])
    .stdout("revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583\n")
    .success();
}
