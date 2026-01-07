use super::*;

#[test]
fn keygen_generates_master_key_by_default() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .success();

  let public = dir.child("keys/master.public");

  public.assert(is_match("[0-9a-f]{64}\n"));

  let private = dir.child("keys/master.private");

  private.assert(is_match("[0-9a-f]{64}\n"));

  let public = ed25519_dalek::VerifyingKey::from_bytes(
    &hex::decode(load_key(&public)).unwrap().try_into().unwrap(),
  )
  .unwrap();

  let private = ed25519_dalek::SigningKey::from_bytes(
    &hex::decode(load_key(&private)).unwrap().try_into().unwrap(),
  );

  assert!(!public.is_weak());

  assert_eq!(private.verifying_key(), public);
}

#[test]
fn error_if_master_public_key_already_exists() {
  let dir = TempDir::new().unwrap();

  let keys = dir.child("keys");

  fs::create_dir_all(&keys).unwrap();

  fs::write(keys.child("master.public"), "foo").unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: public key already exists: `.*master.public`\n",
    ))
    .failure();
}

#[test]
fn error_if_master_private_key_already_exists() {
  let dir = TempDir::new().unwrap();

  let keys = dir.child("keys");

  fs::create_dir_all(&keys).unwrap();

  fs::write(keys.child("master.private"), "foo").unwrap();

  cargo_bin_cmd!("filepack")
    .env("FILEPACK_DATA_DIR", dir.path())
    .arg("keygen")
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: private key already exists: `.*master.private`\n",
    ))
    .failure();
}
