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
