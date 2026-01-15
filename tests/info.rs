use super::*;

#[test]
fn no_keys() {
  Test::new()
    .create_dir("keys")
    .chmod("keys", 0o700)
    .args(["info"])
    .stdout_regex(r#"\{\n  "data-dir": ".*",\n  "key-dir": ".*keys",\n  "keys": \{\}\n\}\n"#)
    .success();
}

#[test]
fn with_keys() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .args(["keygen", "--name", "deploy"])
    .success();

  let master_key = test.read("keys/master.public");
  let deploy_key = test.read("keys/deploy.public");

  test
    .args(["info"])
    .stdout_regex(&format!(
      r#"\{{\n  "data-dir": ".*",\n  "key-dir": ".*keys",\n  "keys": \{{\n    "deploy": "{deploy_key}",\n    "master": "{master_key}"\n  }}\n}}\n"#
    ))
    .success();
}
