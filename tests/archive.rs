use super::*;

#[test]
fn creates_archive_from_json() {
  Test::new()
    .touch("foo")
    .write(
      "manifest.json",
      json! {
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
