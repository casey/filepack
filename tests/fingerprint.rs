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

  let fingerprint = "864e5111ebe431702448d7d7c3f9b962d5659f761fb4287049d52d6376a4c20e";

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
