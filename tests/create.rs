use super::*;

#[test]
fn backslash_error() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("\\")
    .arg("create")
    .stderr(
      "\
error: invalid path `\\`
       ├─ paths contains invalid component `\\`
       └─ component may not contain path separator `\\`
",
    )
    .failure();
}

#[test]
fn empty_directories_are_included() {
  Test::new()
    .create_dir("foo")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
          },
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 0 files\n")
    .success();
}

#[test]
fn file_in_subdirectory() {
  Test::new()
    .touch("foo/bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            bar: {
              hash: EMPTY_HASH,
              size: 0
            }
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn force_overwrites_manifest() {
  Test::new()
    .touch("filepack.json")
    .touch("foo")
    .args(["create", "--force", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn force_overwrites_manifest_with_destination() {
  Test::new()
    .touch("foo.json")
    .touch("foo")
    .args(["create", "--force", ".", "--manifest", "foo.json"])
    .assert_file(
      "foo.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", ".", "--manifest", "foo.json"])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn manifest_already_exists_error() {
  Test::new()
    .touch("filepack.json")
    .args(["create", "."])
    .stderr("error: manifest `filepack.json` already exists\n")
    .failure();
}

#[test]
fn mismatched_key() {
  Test::new()
    .data_dir("foo")
    .arg("keygen")
    .success()
    .arg("keygen")
    .success()
    .rename("foo/keychain/master.private", "keychain/master.private")
    .create_dir("bar")
    .args(["create", "bar", "--sign"])
    .stderr("error: public key `master.public` doesn't match private key `master.private`\n")
    .failure();
}

#[test]
fn multiple_empty_directory_are_included() {
  Test::new()
    .create_dir("foo")
    .create_dir("bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          bar: {
          },
          foo: {
          },
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 0 files\n")
    .success();
}

#[test]
fn nested_empty_directories_are_included() {
  Test::new()
    .create_dir("foo/bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            bar: {
            },
          },
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 0 files\n")
    .success();
}

#[test]
fn no_files() {
  Test::new()
    .args(["create", "."])
    .assert_file("filepack.json", json_pretty! { files: {}, notes: [] })
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 0 files\n")
    .success();
}

#[test]
fn non_unicode_path_error() {
  if cfg!(target_os = "macos") {
    return;
  }

  Test::new()
    .touch_non_unicode()
    .args(["create", "."])
    .stderr_path("error: path not valid unicode: `./�`\n")
    .failure();
}

#[test]
fn private_key_load_error_message() {
  Test::new()
    .touch("foo/bar")
    .touch("keychain/master.private")
    .write("keychain/master.public", PUBLIC_KEY)
    .chmod("keychain", 0o700)
    .chmod("keychain/master.private", 0o600)
    .args(["create", "--sign", "foo"])
    .stderr_regex("error: invalid private key `.*master.private`.*failed to decode bech32.*")
    .failure();
}

#[test]
fn sign_creates_valid_signature() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "--sign", "foo"])
    .success()
    .args(["verify", "foo"])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();

  let manifest_path = test.path().join("foo/filepack.json");
  let manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let public_key = test.read_public_key("keychain/master.public");

  assert_eq!(manifest.notes.len(), 1);
  assert_eq!(manifest.notes[0].signatures.len(), 1);
  assert!(manifest.notes[0].signatures.contains_key(&public_key));
  assert!(manifest.notes[0].time.is_none());
}

#[test]
fn sign_fails_if_master_key_not_available() {
  Test::new()
    .arg("keygen")
    .success()
    .remove_file("keychain/master.private")
    .touch("foo/bar")
    .args(["create", "--sign", "foo"])
    .stderr_regex("error: private key not found: `.*master.private`\n")
    .failure();
}

#[test]
fn sign_with_named_key() {
  let test = Test::new()
    .args(["keygen", "--name", "deploy"])
    .success()
    .touch("foo/bar")
    .args(["create", "--sign", "--key", "deploy", "foo"])
    .success();

  let manifest_path = test.path().join("foo/filepack.json");
  let manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let public_key = test.read_public_key("keychain/deploy.public");

  assert_eq!(manifest.notes.len(), 1);
  assert_eq!(manifest.notes[0].signatures.len(), 1);
  assert!(manifest.notes[0].signatures.contains_key(&public_key));

  test
    .args(["verify", "foo"])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn sign_with_time() {
  use std::time::{SystemTime, UNIX_EPOCH};

  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "--sign", "--time", "foo"])
    .success()
    .args(["verify", "foo"])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();

  let manifest_path = test.path().join("foo/filepack.json");
  let manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let public_key = test.read_public_key("keychain/master.public");

  assert_eq!(manifest.notes.len(), 1);
  assert_eq!(manifest.notes[0].signatures.len(), 1);
  assert!(manifest.notes[0].signatures.contains_key(&public_key));

  let time = manifest.notes[0].time.unwrap();
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let one_minute_ago = now - 60 * 1_000_000_000;
  assert!(time >= one_minute_ago && time <= now);
}

#[test]
fn sign_with_unknown_key() {
  Test::new()
    .touch("foo/bar")
    .args(["create", "--sign", "--key", "deploy", "foo"])
    .stderr_regex("error: public key not found: `.*deploy.public`\n")
    .failure();
}

#[test]
fn single_file() {
  Test::new()
    .touch("foo")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn single_file_mmap() {
  Test::new()
    .touch("foo")
    .args(["--mmap", "create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["--mmap", "verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn single_file_omit_root() {
  Test::new()
    .touch("foo")
    .arg("create")
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn single_file_parallel() {
  Test::new()
    .touch("foo")
    .args(["--parallel", "create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["--parallel", "verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn single_non_empty_file() {
  Test::new()
    .write("foo", "bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json_pretty! {
        files: {
          foo: {
            hash: "f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d",
            size: 3
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 3 bytes\n")
    .success();
}

#[test]
fn symlink_error() {
  Test::new()
    .symlink("foo", "bar")
    .args(["create", "."])
    .stderr("error: symlink at `bar`\n")
    .failure();
}

#[test]
fn with_manifest_path() {
  Test::new()
    .touch("foo")
    .args(["create", "--manifest", "hello.json"])
    .assert_file(
      "hello.json",
      json_pretty! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "--manifest", "hello.json"])
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn with_metadata() {
  Test::new()
    .touch("bar")
    .write("metadata.yaml", "title: Foo")
    .arg("create")
    .assert_file_regex("filepack.json", r#".*"metadata.yaml".*"#)
    .success()
    .arg("verify")
    .stderr("successfully verified 2 files totaling 10 bytes\n")
    .success();
}
