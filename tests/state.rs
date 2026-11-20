use super::*;

#[test]
fn empty() {
  Test::new()
    .arg("create")
    .success()
    .arg("state")
    .stdout(json_pretty! {
      servers: {},
    })
    .success();
}

#[test]
fn with_path() {
  let state = json_pretty! {
    servers: {
      bar: {
        number: 0,
        revision: "revision12c7922354e5c2496ffc5e87ee0b29cc8767dc35a7c0e31e9d239014df5b44583"
      }
    }
  };

  Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .write("foo/.filepack/state.json", state.clone())
    .args(["state", "foo"])
    .stdout(state)
    .success();
}
