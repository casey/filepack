use super::*;

#[test]
fn extra_fields_are_not_allowed() {
  Test::new()
    .args(["verify", "."])
    .write(
      "filepack.json",
      json! {
        files: {},
        foo: "bar"
      },
    )
    .stderr(
      "\
error: failed to deserialize manifest at `filepack.json`
       └─ unknown field `foo`, expected `files` or `signatures` at line 1 column 17\n",
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
    .write("filepack.json", json! { files: {} })
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
        }
      },
    )
    .args(["verify"])
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
        }
      },
    )
    .args(["verify"])
    .stderr_regex("error: file missing: `foo`\n")
    .failure()
    .args(["verify", "--ignore-missing"])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn malformed_signature_error() {
  Test::new()
    .args(["create"])
    .success()
    .write(
      "filepack.json",
      json! {
        files: {},
        signatures: {
          "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b":
            "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b"
        }
      },
    )
    .args(["verify"])
    .stderr_regex(
      "error: failed to deserialize manifest at `filepack.json`\n.*invalid signature byte length.*",
    )
    .failure();
}

#[test]
fn manifest_not_found_error_message() {
  Test::new()
    .args(["verify"])
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
        }
      },
    )
    .args(["verify", "dir"])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn metadata_allows_unknown_keys() {
  Test::new()
    .write(
      "filepack.json",
      json! {
        files: {
          "metadata.json": {
            hash: "1845a2ea1b86a250cb1c24115032cc0fdc064001f59af4a5e9a17be5cd7efbbc",
            size: 25
          }
        }
      },
    )
    .write(
      "metadata.json",
      json! {
        title: "Foo",
        bar: 100
      }
      .trim_end_matches('\n'),
    )
    .args(["verify"])
    .stderr("successfully verified 1 file totaling 25 bytes with 0 signatures\n")
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
            hash: "f113b1430243e68a2976426b0e13f21e5795cc107a914816fbf6c2f511092f4b",
            size: 13
          }
        }
      },
    )
    .write(
      "metadata.json",
      json! { title: 100 }.trim_end_matches('\n'),
    )
    .args(["verify"])
    .stderr_regex(
      "error: failed to deserialize metadata at `.*metadata.json`\n.*",
    )
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
fn missing_signature_error() {
  Test::new()
    .args(["create"])
    .success()
    .args([
      "verify",
      "--key",
      "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b",
    ])
    .stderr(
      "error: no signature found for key \
      7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b\n",
    )
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
fn nested_extraneous_empty_directory_error() {
  Test::new()
    .create_dir("foo/bar")
    .args(["create", "."])
    .success()
    .create_dir("foo/bar/baz")
    .args(["verify", "."])
    .stderr(&path(
      "error: extraneous directory not in manifest: `foo/bar/baz`\n",
    ))
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
    .write("filepack.json", json! { files: {} })
    .args(["verify", "."])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn non_unicode_manifest_deserialize_error() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").write_binary(&[0x80]).unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
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
  use std::path::PathBuf;

  // macos does not allow non-unicode filenames
  if cfg!(target_os = "macos") {
    return;
  }

  let dir = TempDir::new().unwrap();

  let invalid: PathBuf;

  #[cfg(unix)]
  {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    invalid = OsStr::from_bytes(&[0x80]).into();
  };

  #[cfg(windows)]
  {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    invalid = OsString::from_wide(&[0xd800]).into();
  };

  dir.child(invalid).touch().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! { files: {} })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(path("error: path not valid unicode: `./�`\n"))
    .failure();
}

#[test]
fn print() {
  let dir = TempDir::new().unwrap();

  let manifest = json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  };

  dir.child("foo").touch().unwrap();

  dir.child("filepack.json").write_str(&manifest).unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "--print", "."])
    .current_dir(&dir)
    .assert()
    .stdout(manifest)
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn signature_verification_success() {
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
    .args(["create", "--sign", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let public_key = load_key(&dir.child("keys/master.public"));

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo", "--key", &public_key])
    .current_dir(&dir)
    .assert()
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();
}

#[test]
fn single_file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      }
    })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn single_file_mmap() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      }
    })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["--mmap", "verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn single_file_omit_directory() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      }
    })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .arg("verify")
    .current_dir(&dir)
    .assert()
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn single_file_parallel() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      }
    })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["--parallel", "verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn size_mismatch() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo").write_str("bar").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
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
fn valid_signature_for_wrong_pubkey_error() {
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
    .args(["create", "--sign", "foo"])
    .current_dir(&dir)
    .assert()
    .success();

  let mut manifest = Manifest::load(Some(dir.child("foo/filepack.json").utf8_path())).unwrap();

  let public_key = load_key(&dir.child("keys/master.public"))
    .parse::<PublicKey>()
    .unwrap();

  let foo = manifest.signatures.remove(&public_key).unwrap();

  manifest.signatures.insert(
    "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b"
      .parse::<PublicKey>()
      .unwrap(),
    foo,
  );

  manifest
    .save(dir.child("foo/filepack.json").utf8_path())
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo"])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: invalid signature for public key `7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b`\n.*Verification equation was not satisfied.*",
    ))
    .failure();
}

#[test]
fn verify_fingerprint() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args([
      "verify",
      "--fingerprint",
      "864e5111ebe431702448d7d7c3f9b962d5659f761fb4287049d52d6376a4c20e",
    ])
    .current_dir(&dir)
    .assert()
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();

  cargo_bin_cmd!("filepack")
    .args([
      "verify",
      "--fingerprint",
      "0000000000000000000000000000000000000000000000000000000000000000",
    ])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "\
fingerprint mismatch: `.*filepack\\.json`
            expected: 0000000000000000000000000000000000000000000000000000000000000000
              actual: 864e5111ebe431702448d7d7c3f9b962d5659f761fb4287049d52d6376a4c20e
error: fingerprint mismatch\n",
    ))
    .failure();
}

#[test]
fn weak_signature_public_key() {
  let dir = TempDir::new().unwrap();

  dir.child("bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  let json = json! {
    files: {
      bar: {
        hash: EMPTY_HASH,
        size: 0
      }
    },
    signatures: {
      "0000000000000000000000000000000000000000000000000000000000000000":"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
    }
  };

  fs::write(dir.child("filepack.json"), json).unwrap();

  cargo_bin_cmd!("filepack")
    .arg("verify")
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: failed to deserialize manifest at `filepack.json`\n.*weak public key.*",
    ))
    .failure();
}

#[test]
fn with_manifest_path() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  dir
    .child("hello.json")
    .write_str(&json! {
      files: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      }
    })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "--manifest", "hello.json"])
    .current_dir(&dir)
    .assert()
    .success();
}
