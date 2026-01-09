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

  let fingerprint = "f521b82c0bcadb3c61cdf8c7a4831505b2b251175a8f16c3dc44c66d577fd6a1";

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
