use super::*;

#[test]
fn duplicate_key_literal() {
  Test::new()
    .arg("create")
    .success()
    .args(["verify", "--key", PUBLIC_KEY, "--key", PUBLIC_KEY])
    .stderr(&format!("error: duplicate key: `{PUBLIC_KEY}`\n"))
    .failure();
}

#[test]
fn duplicate_key_named() {
  Test::new()
    .arg("keygen")
    .success()
    .arg("create")
    .success()
    .args(["verify", "--key", "master", "--key", "master"])
    .stderr_regex("error: duplicate key: `master`\n")
    .failure();
}

#[test]
fn duplicate_key_named_and_literal() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .arg("create")
    .success();

  let key = test.read("keychain/master.public");

  test
    .args(["verify", "--key", "master", "--key", &key])
    .stderr_regex("error: duplicate key: `master` and `[a-f0-9]{64}`\n")
    .failure();
}

#[test]
fn extra_fields_are_not_allowed() {
  Test::new()
    .args(["verify", "."])
    .write(
      "filepack.json",
      json! {
        files: {},
        notes: [],
        foo: "bar"
      },
    )
    .stderr(
      "\
error: failed to deserialize manifest at `filepack.json`
       └─ unknown field `foo`, expected `files` or `notes` at line 1 column 28\n",
    )
    .failure();
}

#[test]
fn extraneous_empty_directory_error() {
  Test::new()
    .args(["create", "."])
    .success()
    .create_dir("foo")
    .args(["verify", "."])
    .stderr("error: extraneous directory not in manifest: `foo`\n")
    .failure();
}

#[test]
fn extraneous_file_error() {
  Test::new()
    .write("filepack.json", json! { files: {}, notes: [] })
    .touch("foo")
    .args(["verify", "."])
    .stderr("error: extraneous file not in manifest: `foo`\n")
    .failure();
}

#[test]
fn file_not_found_error_message() {
  Test::new()
    .write(
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
    .arg("verify")
    .stderr_regex("error: file missing: `foo`\n")
    .failure();
}

#[test]
fn hash_mismatch() {
  Test::new()
    .write("foo", "foo")
    .args(["create", "."])
    .success()
    .write("foo", "bar")
    .args(["verify", "."])
    .stderr(
      "\
mismatched file: `foo`
       manifest: 04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9 (3 bytes)
           file: f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d (3 bytes)
error: 1 mismatched file
",
    )
    .failure();
}

#[test]
fn ignore_missing() {
  Test::new()
    .write(
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
    .arg("verify")
    .stderr_regex("error: file missing: `foo`\n")
    .failure()
    .args(["verify", "--ignore-missing"])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn malformed_signature_error() {
  Test::new()
    .arg("create")
    .success()
    .write(
      "filepack.json",
      json! {
        files: {},
        notes: [
          {
            signatures: {
              "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b": "00"
            }
          }
        ]
      },
    )
    .arg("verify")
    .stderr_regex(
      "error: failed to deserialize manifest at `filepack.json`\n.*invalid signature byte length.*",
    )
    .failure();
}

#[test]
fn manifest_not_found_error_message() {
  Test::new()
    .arg("verify")
    .stderr("error: manifest `filepack.json` not found\n")
    .failure();
}

#[test]
fn manifest_paths_are_relative_to_root() {
  Test::new()
    .touch("dir/foo")
    .write(
      "dir/filepack.json",
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
    .args(["verify", "dir"])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn metadata_allows_unknown_keys() {
  let metadata = json! {
    title: "Foo",
    bar: 100
  };

  let hash = blake3::hash(metadata.as_bytes()).to_string();

  Test::new()
    .write(
      "filepack.json",
      json! {
        files: {
          "metadata.json": {
            hash: hash,
            size: 26
          }
        },
        notes: [],
      },
    )
    .write("metadata.json", metadata)
    .arg("verify")
    .stderr("successfully verified 1 file totaling 26 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn metadata_may_not_be_invalid() {
  Test::new()
    .write(
      "filepack.json",
      json! {
        files: {
          "metadata.json": {
            hash: "bc9ebf24ee55783c96f0794cc208c03933e318986ed9b3f347020606e21f7b4b",
            size: 14
          }
        },
        notes: [],
      },
    )
    .write("metadata.json", json! { title: 100 })
    .arg("verify")
    .stderr_regex("error: failed to deserialize metadata at `.*metadata.json`\n.*")
    .failure();
}

#[test]
fn missing_empty_directory_error() {
  Test::new()
    .create_dir("foo")
    .args(["create", "."])
    .success()
    .remove_dir("foo")
    .args(["verify", "."])
    .stderr("error: directory missing: `foo`\n")
    .failure();
}

#[test]
fn missing_signature_for_literal_key() {
  Test::new()
    .arg("create")
    .success()
    .args(["verify", "--key", PUBLIC_KEY])
    .stderr(&format!(
      "error: no signature found for key `{PUBLIC_KEY}`\n",
    ))
    .failure();
}

#[test]
fn missing_signature_for_named_key() {
  Test::new()
    .arg("keygen")
    .success()
    .arg("create")
    .success()
    .args(["verify", "--key", "master"])
    .stderr("error: no signature found for key `master`\n")
    .failure();
}

#[test]
fn multiple_keys() {
  let test = Test::new()
    .data_dir("alice")
    .arg("keygen")
    .success()
    .data_dir("bob")
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .data_dir("alice")
    .args(["sign", "foo"])
    .success()
    .data_dir("bob")
    .args(["sign", "foo"])
    .success();

  let alice = test.read("alice/keychain/master.public");
  let bob = test.read("bob/keychain/master.public");

  test
    .args(["verify", "foo", "--key", &alice, "--key", &bob])
    .stderr("successfully verified 1 file totaling 0 bytes with 2 signatures across 1 note\n")
    .success();
}

#[test]
fn multiple_keys_one_missing() {
  let test = Test::new()
    .data_dir("alice")
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .data_dir("alice")
    .args(["sign", "foo/filepack.json"])
    .success();

  let alice = test.read("alice/keychain/master.public");

  test
    .args(["verify", "foo", "--key", &alice, "--key", PUBLIC_KEY])
    .stderr(&format!(
      "error: no signature found for key `{PUBLIC_KEY}`\n",
    ))
    .failure();
}

#[test]
fn multiple_mismatches() {
  Test::new()
    .touch("foo")
    .touch("bar")
    .args(["create", "."])
    .success()
    .write("foo", "baz")
    .write("bar", "bob")
    .args(["verify", "."])
    .stderr(
      "\
mismatched file: `bar`
       manifest: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 (0 bytes)
           file: e476f1b379438de7a1acfd567a94a8c53f08b9714042f7f17e5791645afc3176 (3 bytes)
mismatched file: `foo`
       manifest: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 (0 bytes)
           file: 9624faa79d245cea9c345474fdb1a863b75921a8dd7aff3d84b22c65d1fc0847 (3 bytes)
error: 2 mismatched files
",
    )
    .failure();
}

#[test]
fn named_key() {
  Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "--sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", "master"])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn named_key_invalid() {
  Test::new()
    .arg("create")
    .success()
    .args(["verify", "--key", "@invalid"])
    .stderr(
      "error: invalid value '@invalid' for '--key <KEY>': invalid public key name `@invalid`\n\n\
      For more information, try '--help'.\n",
    )
    .status(2);
}

#[test]
fn named_key_not_found() {
  Test::new()
    .arg("create")
    .success()
    .args(["verify", "--key", "nonexistent"])
    .stderr_regex_path("error: public key not found: `.*keychain/nonexistent.public`\n")
    .failure();
}

#[test]
fn nested_extraneous_empty_directory_error() {
  Test::new()
    .create_dir("foo/bar")
    .args(["create", "."])
    .success()
    .create_dir("foo/bar/baz")
    .args(["verify", "."])
    .stderr_path("error: extraneous directory not in manifest: `foo/bar/baz`\n")
    .failure();
}

#[test]
fn nested_missing_empty_directory_error() {
  Test::new()
    .create_dir("foo/bar/baz")
    .args(["create", "."])
    .success()
    .remove_dir("foo/bar/baz")
    .args(["verify", "."])
    .stderr("error: directory missing: `foo/bar/baz`\n")
    .failure();
}

#[test]
fn no_files() {
  Test::new()
    .write("filepack.json", json! { files: {}, notes: [] })
    .args(["verify", "."])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn non_unicode_manifest_deserialize_error() {
  Test::new()
    .write("filepack.json", [0x80])
    .args(["verify", "."])
    .stderr(
      "\
error: I/O error at `filepack.json`
       └─ stream did not contain valid UTF-8
",
    )
    .failure();
}

#[test]
fn non_unicode_path_error() {
  if cfg!(target_os = "macos") {
    return;
  }

  Test::new()
    .touch_non_unicode()
    .write("filepack.json", json! { files: {}, notes: [] })
    .args(["verify", "."])
    .stderr_path("error: path not valid unicode: `./�`\n")
    .failure();
}

#[test]
fn print() {
  let manifest = json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    },
    notes: [],
  };

  Test::new()
    .touch("foo")
    .write("filepack.json", &manifest)
    .args(["verify", "--print", "."])
    .stdout(&manifest)
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn signature_verification_success() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "--sign", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["verify", "foo", "--key", &public_key])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature across 1 note\n")
    .success();
}

#[test]
fn single_file() {
  Test::new()
    .touch("foo")
    .write(
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
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_file_mmap() {
  Test::new()
    .touch("foo")
    .write(
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
    .args(["--mmap", "verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_file_omit_directory() {
  Test::new()
    .touch("foo")
    .write(
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
    .arg("verify")
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn single_file_parallel() {
  Test::new()
    .touch("foo")
    .write(
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
    .args(["--parallel", "verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}

#[test]
fn size_mismatch() {
  Test::new()
    .touch("foo")
    .args(["create", "."])
    .success()
    .write("foo", "bar")
    .args(["verify", "."])
    .stderr(
      "\
mismatched file: `foo`
       manifest: af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 (0 bytes)
           file: f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d (3 bytes)
error: 1 mismatched file
",
    )
    .failure();
}

#[test]
fn verify_fingerprint() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .args([
      "verify",
      "--fingerprint",
      "864e5111ebe431702448d7d7c3f9b962d5659f761fb4287049d52d6376a4c20e",
    ])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success()
    .args([
      "verify",
      "--fingerprint",
      "0000000000000000000000000000000000000000000000000000000000000000",
    ])
    .stderr_regex(
      "\
fingerprint mismatch: `.*filepack\\.json`
            expected: 0000000000000000000000000000000000000000000000000000000000000000
              actual: 864e5111ebe431702448d7d7c3f9b962d5659f761fb4287049d52d6376a4c20e
error: fingerprint mismatch\n",
    )
    .failure();
}

#[test]
fn weak_signature_public_key() {
  Test::new()
    .touch("bar")
    .arg("create")
    .success()
    .write(
      "filepack.json",
      json! {
        files: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        notes: [
          {
            signatures: {
              "0000000000000000000000000000000000000000000000000000000000000000": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            }
          }
        ]
      },
    )
    .arg("verify")
    .stderr_regex(
      "error: failed to deserialize manifest at `filepack.json`\n.*weak public key.*",
    )
    .failure();
}

#[test]
fn with_manifest_path() {
  Test::new()
    .touch("foo")
    .write(
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
    .args(["verify", "--manifest", "hello.json"])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures across 0 notes\n")
    .success();
}
