use super::*;

#[test]
fn creates_archive_from_json() {
  Test::new()
    .touch("foo")
    .write(
      "manifest.json",
      json! {
        embedded: {},
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
      },
    )
    .args(["archive", "manifest.json", "manifest.filepack"])
    .success()
    .remove_file("manifest.json")
    .arg("verify")
    .stderr("successfully verified 1 file totaling 0 bytes\n")
    .success();
}

#[test]
fn round_trip() {
  Test::new()
    .touch("foo")
    .write(
      "manifest.json",
      json! {
        embedded: {},
        files: {
          foo: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [],
      },
    )
    .args(["archive", "manifest.json", "manifest.filepack"])
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
fn signature_fingerprint_mismatch() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success();

  let manifest_path = test.path().join("foo/manifest.filepack");
  let manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let signature = manifest.signatures.iter().next().unwrap().to_string();

  test
    .write(
      "manifest.json",
      json! {
        embedded: {},
        files: {
          bar: {
            hash: EMPTY_HASH,
            size: 0
          }
        },
        signatures: [signature]
      },
    )
    .args(["archive", "manifest.json", "out.filepack"])
    .stderr_regex(
      "error: signature fingerprint `package1a.*` does not match package fingerprint `package1a.*`\n",
    )
    .failure();
}
