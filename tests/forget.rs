use super::*;

const REVISION: &str = "revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583";

#[test]
fn forget() {
  Test::new()
    .arg("create")
    .success()
    .write(
      ".filepack/state.json",
      json! {
        servers: {
          "https://bar/": {
            number: 0,
            revision: REVISION
          },
          "https://foo/": {
            number: 1,
            revision: REVISION
          }
        }
      },
    )
    .args(["forget", "--server", "foo"])
    .success()
    .arg("state")
    .stdout(json_pretty! {
      servers: {
        "https://bar/": {
          number: 0,
          revision: REVISION
        }
      }
    })
    .success();
}

#[test]
fn missing_entry() {
  Test::new()
    .arg("create")
    .success()
    .args(["forget", "--server", "foo"])
    .stderr("error: no state entry for server `https://foo/`\n")
    .failure();
}
