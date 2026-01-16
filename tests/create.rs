use super::*;

#[test]
fn allow_lint() {
  if cfg!(windows) {
    return;
  }

  Test::new().touch("aux").args(["create", "."]).success();
}

#[test]
fn backslash_error() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("\\")
    .args(["create", "--deny", "all", "."])
    .stderr(
      "\
error: invalid path `\\`
       └─ paths may not contain separator character `\\`
",
    )
    .failure();
}

#[test]
fn deny_case_insensitive_filesystem_path_conflict() {
  if cfg!(windows) || cfg!(target_os = "macos") {
    return;
  }

  Test::new()
    .touch("foo")
    .touch("FOO")
    .args(["create", "--deny", "all", "."])
    .stderr(
      "\
error: paths would conflict on case-insensitive filesystem:
       ├─ `FOO`
       └─ `foo`
error: 1 lint error
",
    )
    .failure();
}

#[test]
fn deny_lint() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .args(["create", "--deny", "all", "."])
    .stderr(
      "\
error: path failed lint: `aux`
       └─ Windows does not allow files named `aux`
error: 1 lint error
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
      json! {
        files: {
          foo: {
          },
        },
        notes: [],
      },
    )
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn file_in_subdirectory() {
  Test::new()
    .touch("foo/bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
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
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
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
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
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
fn metadata_already_exists() {
  Test::new()
    .touch("foo/bar")
    .touch("foo/metadata.json")
    .write("metadata.yaml", "title: Foo")
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .stderr_path("error: metadata `foo/metadata.json` already exists\n")
    .failure()
    .args(["create", "foo", "--metadata", "metadata.yaml", "--force"])
    .assert_file("foo/metadata.json", json! { title: "Foo" })
    .success();
}

#[test]
fn metadata_template_may_not_have_unknown_keys() {
  Test::new()
    .touch("foo/bar")
    .write("metadata.yaml", "title: Foo\nbar: baz")
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .stderr_regex(".*unknown field `bar`.*")
    .failure();
}

#[test]
fn metadata_template_should_not_be_included_in_package() {
  Test::new()
    .touch("foo")
    .write("metadata.yaml", "title: Foo")
    .args(["create", ".", "--metadata", "metadata.yaml"])
    .stderr("error: metadata template `metadata.yaml` should not be included in package\n")
    .failure();
}

#[test]
fn mismatched_key() {
  Test::new()
    .data_dir("foo")
    .args(["keygen"])
    .success()
    .args(["keygen"])
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
      json! {
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
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn nested_empty_directories_are_included() {
  Test::new()
    .create_dir("foo/bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json! {
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
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn no_files() {
  Test::new()
    .args(["create", "."])
    .assert_file("filepack.json", json! { files: {}, notes: [] })
    .success()
    .args(["verify", "."])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures across 0 notes\n")
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
    .stderr_regex(
      "error: invalid private key `.*master.private`.*invalid private key byte length 0.*",
    )
    .failure();
}

#[test]
fn sign_creates_valid_signature() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .touch("foo/bar")
    .args(["create", "--sign", "foo"])
    .success();

  let manifest_path = test.path().join("foo/filepack.json");
  let manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let public_key = test.read_public_key("keychain/master.public");

  assert_eq!(manifest.notes.len(), 1);
  assert_eq!(manifest.notes[0].signatures.len(), 1);
  assert!(manifest.notes[0].signatures.contains_key(&public_key));

  test
    .args(["verify", "foo"])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn sign_fails_if_master_key_not_available() {
  Test::new()
    .args(["keygen"])
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
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_file_mmap() {
  Test::new()
    .touch("foo")
    .args(["--mmap", "create", "."])
    .assert_file(
      "filepack.json",
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_file_omit_root() {
  Test::new()
    .touch("foo")
    .args(["create"])
    .assert_file(
      "filepack.json",
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_file_parallel() {
  Test::new()
    .touch("foo")
    .args(["--parallel", "create", "."])
    .assert_file(
      "filepack.json",
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_non_empty_file() {
  Test::new()
    .write("foo", "bar")
    .args(["create", "."])
    .assert_file(
      "filepack.json",
      json! {
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
    .stderr("successfully verified 1 file totaling 3 bytes with 0 signatures across 0 notes\n")
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
      json! {
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
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn with_metadata() {
  Test::new()
    .touch("foo/bar")
    .write("metadata.yaml", "title: Foo")
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .assert_file(
      "foo/filepack.json",
      json! {
        files: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          },
          "metadata.json": {
            hash: "395190e326d9f4b03fff68cacda59e9c31b9b2a702d46a12f89bfb1ec568c0f1",
            size: 16
          }
        },
        notes: [],
      },
    )
    .assert_file("foo/metadata.json", json! { title: "Foo" })
    .success()
    .args(["verify", "foo"])
    .stderr("successfully verified 2 files totaling 16 bytes with 0 signatures across 0 notes\n")
    .success();
}
