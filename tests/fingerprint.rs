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

  let fingerprint = "aef130d1d67b911c301079bf05ab37d4160f96c15cc4d1832ea77a9a24a2a73e";

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
