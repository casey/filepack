use super::*;

#[test]
fn appends_filename_if_argument_is_directory() {
  let test = Test::new()
    .keygen("master")
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();
}

#[test]
fn defaults_to_current_directory() {
  let test = Test::new()
    .keygen("master")
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .current_dir("foo")
    .arg("sign")
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();
}

#[test]
fn existing_signatures_are_preserved() {
  let test = Test::new()
    .data_dir("a")
    .keygen("master")
    .data_dir("b")
    .keygen("master")
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .data_dir("a")
    .args(["sign", "foo/manifest.filepack"])
    .success()
    .data_dir("b")
    .args(["sign", "foo/manifest.filepack"])
    .success();

  let a = test.read("a/keychain/master.public");
  let b = test.read("b/keychain/master.public");

  test
    .args(["verify", "foo", "--key", &a])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures\n")
    .success()
    .args(["verify", "foo", "--key", &b])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures\n")
    .success();
}

#[test]
fn mismatched_key() {
  Test::new()
    .data_dir("foo")
    .keygen("master")
    .keygen("master")
    .rename("foo/keychain/master.private", "keychain/master.private")
    .create_dir("bar")
    .args(["create", "bar"])
    .success()
    .args(["sign", "bar"])
    .stderr("error: public key `master.public` doesn't match private key `master.private`\n")
    .failure();
}

#[test]
fn named() {
  let test = Test::new()
    .keygen("deploy")
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/deploy.public");

  test
    .args(["sign", "--key", "deploy", "foo/manifest.filepack"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();
}

#[test]
fn preserves_embedded_files() {
  let test = Test::new()
    .keygen("master")
    .write("pkg/filepack.yaml", "title: foo")
    .touch("pkg/bar")
    .args(["create", "pkg"])
    .success()
    .args(["sign", "pkg"])
    .success();

  let manifest = manifest(&test.path().join("pkg/manifest.filepack"));
  assert!(!manifest.embedded.is_empty());
}

#[test]
fn re_signing_is_idempotent() {
  Test::new()
    .keygen("master")
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .args(["sign", "foo/manifest.filepack"])
    .success()
    .args(["verify", "foo", "--key", "master"])
    .stderr("successfully verified 0 files with 1 signature\n")
    .success()
    .args(["sign", "foo/manifest.filepack"])
    .success()
    .args(["verify", "foo", "--key", "master"])
    .stderr("successfully verified 0 files with 1 signature\n")
    .success();
}

#[test]
fn stored_revision_is_signed() {
  let test = Test::new()
    .keygen("master")
    .create_dir("foo")
    .args(["create", "foo"])
    .success();

  let revision = add_parent(&test.path().join("foo/manifest.filepack"));

  let test = test.args(["sign", "foo"]).success();

  manifest(&test.path().join("foo/manifest.filepack"))
    .signatures
    .first()
    .unwrap()
    .known()
    .unwrap()
    .verify(revision.hash())
    .unwrap();
}

#[test]
fn unknown_fields_are_rejected() {
  let test = Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success();

  let path = test.path().join("foo/manifest.filepack");

  let bytes = add_unknown_field(&path);

  Manifest::load(Some(&path)).unwrap();

  let test = test
    .args(["sign", "foo"])
    .stderr_path(
      "
        error: failed to decode manifest at `foo/manifest.filepack`
               └─ unknown field with key 18446744073709551615
      ",
    )
    .failure();

  assert_eq!(
    fs::read(test.path().join("foo/manifest.filepack")).unwrap(),
    bytes,
  );
}

#[test]
fn unknown_root_entries_are_preserved() {
  let hash = Hash::bytes(b"qux");

  let test = Test::new()
    .keygen("master")
    .touch("foo/bar")
    .write_manifest(
      "foo/manifest.filepack",
      json! {
        embedded: {
          *hash: (hex::encode(b"qux"))
        },
        package: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
        unknown: {
          baz: {
            qux: {
              hash: hash,
              size: 3
            }
          },
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        }
      },
    );

  let public_key = test.read("keychain/master.public");

  let test = test
    .args(["sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();

  let manifest = manifest(&test.path().join("foo/manifest.filepack"));

  assert_eq!(manifest.signatures.len(), 1);

  assert_eq!(
    manifest.embedded,
    BTreeMap::from([(hash, b"qux".to_vec().into())])
  );

  assert_eq!(
    serde_json::to_string(&manifest.unknown).unwrap() + "\n",
    json! {
      baz: {
        qux: {
          hash: hash,
          size: 3
        }
      },
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    },
  );
}

#[test]
fn unknown_signatures_are_preserved() {
  let test = Test::new()
    .keygen("master")
    .keygen("bar")
    .touch("foo/bar")
    .args(["create", "--sign", "foo"])
    .success();

  let payload = add_unknown_signature(&test.path().join("foo/manifest.filepack"));

  let test = test.args(["sign", "--key", "bar", "foo"]).success();

  let manifest = manifest(&test.path().join("foo/manifest.filepack"));

  assert_eq!(manifest.signatures.len(), 3);

  assert!(manifest.signatures.contains(&Decoded::Unknown(payload)));
}

#[test]
fn updates_manifest_with_signature() {
  let test = Test::new()
    .keygen("master")
    .touch("foo/bar")
    .args(["create", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  let test = test
    .args(["verify", "foo", "--key", &public_key])
    .stderr(&format!(
      "error: no signature found for key `{public_key}`\n"
    ))
    .failure()
    .args(["sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();

  let manifest_path = test.path().join("foo/manifest.filepack");
  let fingerprint = fingerprint(&manifest_path);
  let manifest = manifest(&manifest_path);
  manifest
    .signatures
    .first()
    .unwrap()
    .known()
    .unwrap()
    .verify(fingerprint.revision().hash())
    .unwrap();
}
