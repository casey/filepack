use super::*;

#[test]
fn fingerprint() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  cargo_bin_cmd!("filepack")
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  let json = json! {
    files: {
      foo: {
        hash: EMPTY_HASH,
        size: 0
      }
    }
  };

  dir.child("filepack.json").assert(json.clone());

  let fingerprint = "7636fe8bf9b25782e9c193e887c1b004e0a951a168cc37298da782f1a8516aaa";

  cargo_bin_cmd!("filepack")
    .arg("fingerprint")
    .current_dir(&dir)
    .assert()
    .stdout(format!("{fingerprint}\n"))
    .success();

  cargo_bin_cmd!("filepack")
    .args(["fingerprint", dir.path().to_str().unwrap()])
    .assert()
    .stdout(format!("{fingerprint}\n"))
    .success();

  cargo_bin_cmd!("filepack")
    .args([
      "fingerprint",
      dir.path().join("filepack.json").to_str().unwrap(),
    ])
    .assert()
    .stdout(format!("{fingerprint}\n"))
    .success();
}
