use super::*;

#[test]
fn defaults_to_current_directory() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .arg("manifest")
    .stdout(json_pretty! {
      embedded: {},
      package: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      },
      signatures: [],
    })
    .success();
}

#[test]
fn includes_embedded_files() {
  Test::new()
    .write("metadata.yaml", "title: foo")
    .touch("bar")
    .arg("create")
    .success()
    .arg("manifest")
    .stdout_regex(r#".*"embedded": \{[^}].*"#)
    .success();
}

#[test]
fn unknown_signatures_error() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo")
    .args(["create", "--sign"])
    .success();

  add_unknown_signature(&test.path().join("manifest.filepack"));

  test
    .arg("manifest")
    .stderr("error: manifest contains 1 unrecognized signature\n")
    .failure();
}

#[test]
fn with_path() {
  Test::new()
    .touch("pkg/bar")
    .args(["create", "pkg"])
    .success()
    .args(["manifest", "pkg"])
    .stdout(json_pretty! {
      embedded: {},
      package: {
        bar: {
          hash: EMPTY_HASH,
          size: 0
        }
      },
      signatures: [],
    })
    .success();
}
