use super::*;

#[test]
fn allow_lint() {
  if cfg!(windows) {
    return;
  }

  Test::new()
    .touch("aux")
    .args(["create", "."])
    .success();
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
    .success()
    .assert_file(
      "filepack.json",
      json! {
        files: {
          foo: {
          },
        },
      },
    )
    .args(["verify", "."])
    .stderr("successfully verified 0 files totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn file_in_subdirectory() {
  Test::new()
    .touch("foo/bar")
    .args(["create", "."])
    .success()
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
        }
      },
    )
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn force_overwrites_manifest() {
  Test::new()
    .touch("filepack.json")
    .touch("foo")
    .args(["create", "--force", "."])
    .success()
    .assert_file(
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
    .args(["verify", "."])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
    .success();
}

#[test]
fn force_overwrites_manifest_with_destination() {
  Test::new()
    .touch("foo.json")
    .touch("foo")
    .args(["create", "--force", ".", "--manifest", "foo.json"])
    .success()
    .assert_file(
      "foo.json",
      json! {
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        }
      },
    )
    .args(["verify", ".", "--manifest", "foo.json"])
    .stderr("successfully verified 1 file totaling 0 bytes with 0 signatures\n")
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
    .stderr(&path("error: metadata `foo/metadata.json` already exists\n"))
    .failure()
    .args(["create", "foo", "--metadata", "metadata.yaml", "--force"])
    .success()
    .assert_file("foo/metadata.json", json! { title: "Foo" });
}

#[test]
fn metadata_template_may_not_have_unknown_keys() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir
    .child("metadata.yaml")
    .write_str("title: Foo\nbar: baz")
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .stderr(is_match(".*unknown field `bar`.*"))
    .failure();
}

#[test]
fn metadata_template_should_not_be_included_in_package() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  dir.child("metadata.yaml").write_str("title: Foo").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", ".", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .stderr("error: metadata template `metadata.yaml` should not be included in package\n")
    .failure();
}

#[test]
fn multiple_empty_directory_are_included() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").create_dir_all().unwrap();

  dir.child("bar").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      bar: {
      },
      foo: {
      },
    },
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn nested_empty_directories_are_included() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        bar: {
        },
      },
    },
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn no_files() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert("{}\n");

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
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

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr(path("error: path not valid unicode: `./�`\n"))
    .failure();
}

#[test]
fn private_key_load_error_message() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir.child("keys/master.private").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--sign", "foo"])
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: invalid private key `.*master.private`.*invalid private key byte length 0.*",
    ))
    .failure();
}

#[test]
fn sign_creates_valid_signature() {
  let dir = TempDir::new().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--sign", "foo"])
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  let manifest = Manifest::load(Some(dir.child("foo/filepack.json").utf8_path())).unwrap();

  let public_key = load_key(&dir.child("keys/master.public"))
    .parse::<PublicKey>()
    .unwrap();

  assert_eq!(manifest.signatures.len(), 1);

  let signature = manifest.signatures[&public_key].clone();

  public_key
    .verify(manifest.fingerprint(), &signature)
    .unwrap();

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn sign_fails_if_master_key_not_available() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--sign", "foo"])
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .stderr(is_match(
      "error: private key not found: `.*master.private`\n",
    ))
    .failure();
}

#[test]
fn single_file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file_mmap() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["--mmap", "create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["--mmap", "verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file_omit_root() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file_parallel() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["--parallel", "create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["--parallel", "verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_non_empty_file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").write_str("bar").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        hash: "f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d",
        size: 3
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn symlink_error() {
  let dir = TempDir::new().unwrap();

  #[cfg(unix)]
  std::os::unix::fs::symlink("foo", dir.path().join("bar")).unwrap();

  #[cfg(windows)]
  std::os::windows::fs::symlink_file("foo", dir.path().join("bar")).unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: symlink at `bar`\n")
    .failure();
}

#[test]
fn with_manifest_path() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--manifest", "hello.json"])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("hello.json").assert(json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", "--manifest", "hello.json"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn with_metadata() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir.child("metadata.yaml").write_str("title: Foo").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/filepack.json").assert(json! {
    files: {
      bar: {
        hash: EMPTY_HASH,
        size: 0
      },
      "metadata.json": {
        hash: "395190e326d9f4b03fff68cacda59e9c31b9b2a702d46a12f89bfb1ec568c0f1",
        size: 16
      }
    }
  });

  dir
    .child("foo/metadata.json")
    .assert(json! { title: "Foo" });

  cargo_bin_cmd!("filepack")
    .args(["verify", "foo"])
    .current_dir(&dir)
    .assert()
    .success();
}
