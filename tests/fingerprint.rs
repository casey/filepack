use super::*;

#[test]
fn single_file_omit_root() {
  let dir = TempDir::new().unwrap();

  dir.child("foo").touch().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  let json = r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#;

  dir.child("filepack.json").assert(json.to_owned() + "\n");

  assert_eq!(
    blake3::hash(json.as_bytes()),
    "74ddbe0dcf48c634aca1d90f37defd60b230fc52857ffa4b6c956583e8a4daaf"
      .parse::<blake3::Hash>()
      .unwrap(),
  );

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("fingerprint")
    .current_dir(&dir)
    .assert()
    .stdout("74ddbe0dcf48c634aca1d90f37defd60b230fc52857ffa4b6c956583e8a4daaf\n")
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["fingerprint", dir.path().to_str().unwrap()])
    .assert()
    .stdout("74ddbe0dcf48c634aca1d90f37defd60b230fc52857ffa4b6c956583e8a4daaf\n")
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args([
      "fingerprint",
      dir.path().join("filepack.json").to_str().unwrap(),
    ])
    .assert()
    .stdout("74ddbe0dcf48c634aca1d90f37defd60b230fc52857ffa4b6c956583e8a4daaf\n")
    .success();
}
