use super::*;

// todo:
// - deserialize manfiest error
// - existing signature fails to verify error
// - existing signatures are preserved
// - complain if signature already exists?

#[test]
fn updates_manifest_with_signature() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = fs::read_to_string(dir.child("keys/master.public"))
    .unwrap()
    .trim()
    .to_owned();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(format!(
      "error: no signature found for key {public_key}\n"
    )))
    .failure();

  Command::cargo_bin("filepack")
    .unwrap()
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo", "--key", public_key.trim()])
    .current_dir(&dir)
    .assert()
    .success();
}
