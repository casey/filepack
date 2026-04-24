use super::*;

#[test]
fn includes_embeded_files() {
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
fn default_format() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .arg("manifest")
    .stdout(json_pretty! {
      embedded: {},
      files: {
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
fn json_format() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .args(["manifest", "--format", "json"])
    .stdout(json! {
      embedded: {},
      files: {
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
fn json_pretty_format() {
  Test::new()
    .touch("foo")
    .arg("create")
    .success()
    .args(["manifest", "--format", "json-pretty"])
    .stdout(json_pretty! {
      embedded: {},
      files: {
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
fn tsv_error() {
  Test::new()
    .arg("create")
    .success()
    .args(["manifest", "--format", "tsv"])
    .stderr("error: manifest cannot be formatted as TSV\n")
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
      files: {
        bar: {
          hash: EMPTY_HASH,
          size: 0
        }
      },
      signatures: [],
    })
    .success();
}
