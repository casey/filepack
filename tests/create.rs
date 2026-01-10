use super::*;

#[test]
fn allow_lint() {
  if cfg!(windows) {
    return;
  }

  let dir = TempDir::new().unwrap();

  dir.child("aux").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn backslash_error() {
  if cfg!(windows) {
    return;
  }

  let dir = TempDir::new().unwrap();

  dir.child("\\").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--deny", "all", "."])
    .current_dir(&dir)
    .assert()
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

  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();
  dir.child("FOO").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--deny", "all", "."])
    .current_dir(&dir)
    .assert()
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

  let dir = TempDir::new().unwrap();

  dir.child("aux").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--deny", "all", "."])
    .current_dir(&dir)
    .assert()
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
  let dir = TempDir::new().unwrap();

  dir.child("foo").create_dir_all().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
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
fn file_in_subdirectory() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(json! {
    files: {
      foo: {
        bar: {
          hash: EMPTY_HASH,
          size: 0
        }
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
fn force_overwrites_manifest() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").touch().unwrap();
  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--force", "."])
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
fn force_overwrites_manifest_with_destination() {
  let dir = TempDir::new().unwrap();

  dir.child("foo.json").touch().unwrap();
  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "--force", ".", "--manifest", "foo.json"])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo.json").assert(json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  });

  cargo_bin_cmd!("filepack")
    .args(["verify", ".", "--manifest", "foo.json"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn manifest_already_exists_error() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: manifest `filepack.json` already exists\n")
    .failure();
}

#[test]
fn metadata_already_exists() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir.child("foo/metadata.json").touch().unwrap();

  dir.child("metadata.yaml").write_str("title: Foo").unwrap();

  cargo_bin_cmd!("filepack")
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .stderr(path("error: metadata `foo/metadata.json` already exists\n"))
    .failure();

  cargo_bin_cmd!("filepack")
    .args(["create", "foo", "--metadata", "metadata.yaml", "--force"])
    .current_dir(&dir)
    .assert()
    .success();

  dir
    .child("foo/metadata.json")
    .assert(json! { title: "Foo" });
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

  let (_path, manifest) = Manifest::load(Some(dir.child("foo/filepack.json").utf8_path())).unwrap();

  let public_key = load_key(&dir.child("keys/master.public"))
    .parse::<PublicKey>()
    .unwrap();

  assert_eq!(manifest.signatures.len(), 1);

  let signature = manifest.signatures[&public_key].clone();

  public_key
    .verify(manifest.fingerprint().as_bytes(), &signature)
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
