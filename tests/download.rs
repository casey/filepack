use super::*;

#[test]
fn invalid_github_release_error() {
  Command::cargo_bin("filepack")
    .unwrap()
    .args(["download", "--github-release", "a/b/c/d"])
    .assert()
    .stderr(is_match("error: invalid value 'a/b/c/d' for '--github-release <GITHUB_RELEASE>': must be of the form 'OWNER/REPO/TAG'.*"))
    .failure();
}

#[test]
#[ignore]
fn download() {
  let dir = TempDir::new().unwrap();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["download", "--github-release", "casey/filepack/0.0.3"])
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .arg("create")
    .current_dir(&dir)
    .assert()
    .success();

  Command::cargo_bin("filepack")
    .unwrap()
    .args(["hash", dir.path().join("filepack.json").to_str().unwrap()])
    .current_dir(&dir)
    .assert()
    .stdout("d8e3d0f33e58e0f0f0e43082fcdeb2a38f7e560b3bdc0e8862608e8d959e852e\n")
    .success();
}
