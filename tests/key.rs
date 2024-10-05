use super::*;

#[test]
fn prints_pubkey() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  let output = Command::cargo_bin("filepack")
    .unwrap()
    .arg("key")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success()
    .stdout(is_match("[0-9a-f]{64}\n"))
    .get_output()
    .clone();

  let public_key = fs::read_to_string(dir.child("keys/master.public")).unwrap();

  assert_eq!(str::from_utf8(&output.stdout).unwrap(), public_key);
}

#[test]
fn missing_public_key_error() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  fs::remove_file(dir.child("keys/master.public")).unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("key")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .stderr(is_match("error: public key not found: `.*master.public`\n"))
    .failure();
}

#[test]
fn missing_private_key_error() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  fs::remove_file(dir.child("keys/master.private")).unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("key")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: private key not found: `.*master.private`\n",
    ))
    .failure();
}

#[test]
fn mismatched_key_error() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path().join("foo"))
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  fs::rename(
    dir.path().join("foo/keys/master.private"),
    dir.path().join("keys/master.private"),
  )
  .unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("key")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: public key `master.public` doesn't match private key `master.private`\n",
    ))
    .failure();
}
