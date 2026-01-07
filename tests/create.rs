use super::*;

#[test]
fn no_files() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert("{}\n");

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file_omit_root() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_non_empty_file() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").write_str("bar").unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":{"hash":"f2e897eed7d206cd855d441598fa521abc75aa96953e97c030c9612c30c1293d","size":3}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file_mmap() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["--mmap", "create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["--mmap", "verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn single_file_parallel() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["--parallel", "create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["--parallel", "verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn file_in_subdirectory() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo/bar":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

// disable test on macos, since it does not allow non-unicode filenames
#[cfg(not(target_os = "macos"))]
#[test]
fn non_unicode_path_error() {
  use std::path::PathBuf;

  let dir = TempDir::new().unwrap();

  let path: PathBuf;

  #[cfg(unix)]
  {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    path = OsStr::from_bytes(&[0x80]).into();
  };

  #[cfg(windows)]
  {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};

    path = OsString::from_wide(&[0xd800]).into();
  };

  dir.child(path).touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr(format!("error: path not valid unicode: `.{SEPARATOR}�`\n"))
    .failure();
}

#[test]
fn symlink_error() {
  let dir = TempDir::new().unwrap();

  #[cfg(unix)]
  std::os::unix::fs::symlink("foo", dir.path().join("bar")).unwrap();

  #[cfg(windows)]
  std::os::windows::fs::symlink_file("foo", dir.path().join("bar")).unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: symlink at `bar`\n")
    .failure();
}

#[test]
fn empty_directory_error() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").create_dir_all().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: empty directory `foo`\n")
    .failure();
}

#[test]
fn multiple_empty_directory_error() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").create_dir_all().unwrap();

  dir.child("bar").create_dir_all().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: empty directories `bar` and `foo`\n")
    .failure();
}

#[test]
fn only_leaf_empty_directory_is_reported() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").create_dir_all().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr(path("error: empty directory `foo/bar`\n"))
    .failure();
}

#[cfg(not(windows))]
#[test]
fn backslash_error() {
  let dir = TempDir::new().unwrap();

  dir.child("\\").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
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

#[cfg(all(not(windows), not(target_os = "macos")))]
#[test]
fn deny_case_insensitive_filesystem_path_conflict() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();
  dir.child("FOO").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
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

#[cfg(not(windows))]
#[test]
fn deny_lint() {
  let dir = TempDir::new().unwrap();

  dir.child("aux").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
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

#[cfg(not(windows))]
#[test]
fn allow_lint() {
  let dir = TempDir::new().unwrap();

  dir.child("aux").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn manifest_already_exists_error() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "."])
    .current_dir(&dir)
    .assert()
    .stderr("error: manifest `filepack.json` already exists\n")
    .failure();
}

#[test]
fn force_overwrites_manifest() {
  let dir = TempDir::new().unwrap();

  dir.child("filepack.json").touch().unwrap();
  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "--force", "."])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("filepack.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_string() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
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

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "--force", ".", "--manifest", "foo.json"])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", ".", "--manifest", "foo.json"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn with_manifest_path() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "--manifest", "hello.json"])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("hello.json").assert(
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.to_owned() + "\n",
  );

  Command::cargo_bin("filepack")
    .unwrap()
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

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/filepack.json").assert(
    r#"{"files":{"bar":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0},"metadata.json":{"hash":"395190e326d9f4b03fff68cacda59e9c31b9b2a702d46a12f89bfb1ec568c0f1","size":16}}}"#.to_owned() + "\n",
  );

  dir
    .child("foo/metadata.json")
    .assert(r#"{"title":"Foo"}"#.to_owned() + "\n");

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn metadata_template_may_not_have_unknown_keys() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir
    .child("metadata.yaml")
    .write_str("title: Foo\nbar: baz")
    .unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
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

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", ".", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .stderr("error: metadata template `metadata.yaml` should not be included in package\n")
    .failure();
}

#[test]
fn sign_fails_if_master_key_not_available() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
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
fn private_key_load_error_message() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir.child("keys/master.private").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
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

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("keygen")
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  dir.child("foo/bar").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "--sign", "foo"])
    .env("FILEPACK_DATA_DIR", dir.path())
    .current_dir(&dir)
    .assert()
    .success();

  let manifest = Manifest::load(&dir.child("foo/filepack.json"));

  let public_key = load_key(&dir.child("keys/master.public"));

  assert_eq!(manifest.signatures.len(), 1);

  let signature = manifest.signatures[&public_key]
    .parse::<ed25519_dalek::Signature>()
    .unwrap();

  let public_key =
    ed25519_dalek::VerifyingKey::from_bytes(&hex::decode(public_key).unwrap().try_into().unwrap())
      .unwrap();

  let fingerprint = blake3::hash(r#"{"files":{"bar":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#.as_bytes());

  public_key
    .verify_strict(fingerprint.as_bytes(), &signature)
    .unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["verify", "foo"])
    .current_dir(&dir)
    .assert()
    .success();
}

#[test]
fn metadata_already_exists() {
  let dir = TempDir::new().unwrap();

  dir.child("foo/bar").touch().unwrap();

  dir.child("foo/metadata.json").touch().unwrap();

  dir.child("metadata.yaml").write_str("title: Foo").unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "foo", "--metadata", "metadata.yaml"])
    .current_dir(&dir)
    .assert()
    .stderr(path("error: metadata `foo/metadata.json` already exists\n"))
    .failure();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["create", "foo", "--metadata", "metadata.yaml", "--force"])
    .current_dir(&dir)
    .assert()
    .success();

  dir
    .child("foo/metadata.json")
    .assert(r#"{"title":"Foo"}"#.to_owned() + "\n");
}
