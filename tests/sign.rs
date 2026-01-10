use super::*;

#[test]
fn appends_filename_if_argument_is_directory() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = load_key(&dir.child("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn defaults_to_current_directory() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = load_key(&dir.child("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("sign")
    .current_dir(dir.join("foo"))
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn existing_signatures_are_preserved() {
  let dir = TempDir::new().unwrap();

  let a = dir.path().join("a");
  let b = dir.path().join("b");

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path().join("a"))
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path().join("b"))
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", &a)
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", &b)
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  let a = load_key(&a.join("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &a])
    .current_dir(&dir)
    .assert()
    .success();

  let b = load_key(&b.join("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &b])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn existing_signatures_must_be_valid() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let (_path, mut manifest) =
    Manifest::load(Some(dir.child("foo/filepack.json").utf8_path())).unwrap();

  manifest.signatures.insert(
    "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b"
      .parse::<PublicKey>()
      .unwrap(),
    "0".repeat(128).parse::<Signature>().unwrap(),
  );

  manifest
    .save(dir.child("foo/filepack.json").utf8_path())
    .unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .stderr(is_match("error: invalid signature for public key `7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b`\n.*Verification equation was not satisfied\n"))
    .failure();
}

#[test]
fn re_signing_requires_force() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = load_key(&dir.child("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .stderr(format!(
      "error: manifest has already been signed by public key `{public_key}`\n"
    ))
    .failure();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "--force", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn updates_manifest_with_signature() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["create", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = load_key(&dir.child("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(format!(
      "error: no signature found for key {public_key}\n"
    )))
    .failure();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .args(["sign", "foo/filepack.json"])
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .success();
}
