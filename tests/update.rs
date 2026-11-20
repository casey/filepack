use super::*;

#[test]
fn draft_keeps_parents() {
  let test = Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success();

  let path = test.path().join("foo/manifest.filepack");

  let parent = fingerprint(&path).revision().hash();

  add_parent(&path);

  test
    .touch("foo/bar")
    .args(["update", "foo"])
    .assert_manifest(
      "foo/manifest.filepack",
      json_pretty! {
        embedded: {},
        package: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        parents: [(parent.to_string())],
        signatures: [],
      },
    )
    .success();
}

#[test]
fn manifest_not_found() {
  Test::new()
    .arg("update")
    .stderr("error: manifest `manifest.filepack` not found\n")
    .failure();
}

#[test]
fn metadata_is_overwritten() {
  Test::new()
    .write("metadata.yaml", "title: Foo")
    .arg("create")
    .success()
    .arg("update")
    .success();
}

#[test]
fn published_revision_is_parent() {
  let test = Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success();

  let revision = fingerprint(&test.path().join("foo/manifest.filepack"))
    .revision()
    .hash();

  test
    .write(
      "foo/.filepack/state.json",
      serde_json::to_string(&State {
        servers: BTreeMap::from([(
          "bar".into(),
          ServerState {
            number: 0,
            revision,
          },
        )]),
      })
      .unwrap(),
    )
    .touch("foo/baz")
    .args(["update", "foo"])
    .assert_manifest(
      "foo/manifest.filepack",
      json_pretty! {
        embedded: {},
        package: {
          baz: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        parents: [(revision.to_string())],
        signatures: [],
      },
    )
    .success();
}

#[test]
fn sign() {
  Test::new()
    .arg("keygen")
    .success()
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .touch("foo/bar")
    .args(["update", "--sign", "foo"])
    .success()
    .args(["verify", "foo", "--key", "master"])
    .stderr("successfully verified 1 file totaling 0 bytes with 1 signature\n")
    .success();
}

#[test]
fn signatures_are_dropped() {
  Test::new()
    .arg("keygen")
    .success()
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success()
    .touch("foo/bar")
    .args(["update", "foo"])
    .assert_manifest(
      "foo/manifest.filepack",
      json_pretty! {
        embedded: {},
        package: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
      },
    )
    .success();
}

#[test]
fn unchanged_package_is_preserved() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .create_dir("foo")
    .args(["create", "foo"])
    .success();

  let path = test.path().join("foo/manifest.filepack");

  add_parent(&path);

  let test = test.args(["sign", "foo"]).success();

  let bytes = fs::read(&path).unwrap();

  let test = test.args(["update", "foo"]).success();

  assert_eq!(
    fs::read(test.path().join("foo/manifest.filepack")).unwrap(),
    bytes
  );
}

#[test]
fn unknown_entries_are_rejected() {
  Test::new()
    .create_dir("foo")
    .write_manifest(
      "foo/manifest.filepack",
      json! {
        embedded: {},
        package: {},
        signatures: [],
        unknown: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        }
      },
    )
    .args(["update", "foo"])
    .stderr_path("error: manifest `foo/manifest.filepack` contains unknown entries: `bar`\n")
    .failure();
}

#[test]
fn unknown_fields_are_rejected() {
  let test = Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success();

  add_unknown_field(&test.path().join("foo/manifest.filepack"));

  test
    .args(["update", "foo"])
    .stderr_path(
      "
        error: failed to decode manifest at `foo/manifest.filepack`
               └─ unknown field with key 18446744073709551615
      ",
    )
    .failure();
}

#[test]
fn with_manifest_path() {
  Test::new()
    .args(["create", "--manifest", "foo"])
    .success()
    .touch("bar")
    .args(["update", "--manifest", "foo"])
    .assert_manifest(
      "foo",
      json_pretty! {
        embedded: {},
        package: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
      },
    )
    .success();
}
