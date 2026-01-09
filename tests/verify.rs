use super::*;

#[test]
fn no_files() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! { files: {} })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
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
    .success();
}

#[test]
fn extra_fields_are_not_allowed() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {},
      foo: "bar"
    })
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr(
      "\
error: failed to deserialize manifest at `filepack.json`
       └─ unknown field `foo`, expected `files` or `signatures` at line 1 column 17\n",
    )
    .failure();
}

#[test]
fn extraneous_file_error() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! { files: {} })
    .unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: extraneous file not in manifest: `foo`\n")
    .failure();
}

#[test]
fn hash_mismatch() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").write_str("foo").unwrap();

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
       manifest: 04e0bb39f30b1a3feb89f536c93be15055482df748674b00d26e5a75777702e9 (3 bytes)
           file: f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d (3 bytes)
error: 1 mismatched file
",
    )
    .failure();
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
fn multiple_mismatches() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();
  dir.child("bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo").write_str("baz").unwrap();
  dir.child("bar").write_str("bob").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
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
    .success();
}

#[test]
fn manifest_paths_are_relative_to_root() {
  let dir = TempDir::new().unwrap();

  dir.child("dir/foo").touch().unwrap();

  dir
    .child("dir/filepack.json")
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
    .args(["verify", "dir"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn manifest_not_found_error_message() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("verify")
    .current_dir(&dir)
    .assert()
    .stderr("error: manifest `filepack.json` not found\n")
    .failure();
}

#[test]
fn file_not_found_error_message() {
  let dir = TempDir::new().unwrap();

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
    .stderr(is_match("error: file missing: `foo`\n"))
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
fn missing_signature_error() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  cargo_bin_cmd!("filepack")
    .args([
      "verify",
      "--key",
      "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b",
    ])
    .current_dir(&dir)
    .assert()
    .stderr(
      "error: no signature found for key \
      7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b\n",
    )
    .failure();
}

#[test]
fn malformed_signature_error() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  let path = dir.child("filepack.json");

  let manifest_json = json! {
    files: {},
    signatures: {
      "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b":
        "7f1420cdc898f9370fd196b9e8e5606a7992fab5144fc1873d91b8c65ef5db6b"
    }
  };
  fs::write(&path, manifest_json).unwrap();

  cargo_bin_cmd!("filepack")
    .arg("verify")
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: failed to deserialize manifest at `filepack.json`\n.*invalid signature byte length.*",
    ))
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

  let (_path, mut manifest) =
    Manifest::load(Some(dir.child("foo/filepack.json").utf8_path())).unwrap();

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
    .success();
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
      "f521b82c0bcadb3c61cdf8c7a4831505b2b251175a8f16c3dc44c66d577fd6a1",
    ])
    .current_dir(&dir)
    .assert()
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
              actual: f521b82c0bcadb3c61cdf8c7a4831505b2b251175a8f16c3dc44c66d577fd6a1
error: fingerprint mismatch\n",
    ))
    .failure();
}

#[test]
fn ignore_missing() {
  let dir = TempDir::new().unwrap();

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
    .stderr(is_match("error: file missing: `foo`\n"))
    .failure();

  cargo_bin_cmd!("filepack")
    .args(["verify", "--ignore-missing"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn metadata_allows_unknown_keys() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {
        "metadata.json": {
          hash: "1845a2ea1b86a250cb1c24115032cc0fdc064001f59af4a5e9a17be5cd7efbbc",
          size: 25
        }
      }
    })
    .unwrap();

  dir
    .child("metadata.json")
    .write_str(
      json! {
        title: "Foo",
        bar: 100
      }
      .trim_end_matches('\n'),
    )
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn metadata_may_not_be_invalid() {
  let dir = TempDir::new().unwrap();

  dir
    .child("filepack.json")
    .write_str(&json! {
      files: {
        "metadata.json": {
          hash: "f113b1430243e68a2976426b0e13f21e5795cc107a914816fbf6c2f511092f4b",
          size: 13
        }
      }
    })
    .unwrap();

  dir
    .child("metadata.json")
    .write_str(json! { title: 100 }.trim_end_matches('\n'))
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify"])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: failed to deserialize metadata at `.*metadata.json`\n.*",
    ))
    .failure();
}

#[test]
fn extraneous_empty_directory_error() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: extraneous directory not in manifest: `foo`\n")
    .failure();
}

#[test]
fn missing_empty_directory_error() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  std::fs::remove_dir(dir.path().join("foo")).unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: directory missing: `foo`\n")
    .failure();
}

#[test]
fn nested_extraneous_empty_directory_error() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar/baz").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: extraneous directory not in manifest: `foo/bar/baz`\n")
    .failure();
}

#[test]
fn nested_missing_empty_directory_error() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar/baz").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  std::fs::remove_dir(dir.path().join("foo/bar/baz")).unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: directory missing: `foo/bar/baz`\n")
    .failure();
}
