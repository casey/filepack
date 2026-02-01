use super::*;

#[test]
fn no_signatures() {
  Test::new()
    .arg("create")
    .success()
    .arg("signatures")
    .stdout("[]\n")
    .success();
}

#[test]
fn signature_with_time() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["sign", "--time", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["signatures", "--format", "json", "foo"])
    .stdout_regex(&format!(
      r#"\[\{{"public-key":"{public_key}","timestamp":\d+\}}\]\n"#,
    ))
    .success();
}

#[test]
fn signature_without_time() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["sign", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["signatures", "--format", "json", "foo"])
    .stdout(format!(
      "[{{\"public-key\":\"{public_key}\",\"timestamp\":null}}]\n"
    ))
    .success();
}

#[test]
fn tsv_format() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["sign", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["signatures", "--format", "tsv", "foo"])
    .stdout(format!("{public_key}\t\n"))
    .success();
}

#[test]
fn tsv_format_with_time() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .touch("foo/bar")
    .args(["create", "foo"])
    .success()
    .args(["sign", "--time", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["signatures", "--format", "tsv", "foo"])
    .stdout_regex(&format!("{public_key}\t\\d+\n"))
    .success();
}

#[test]
fn defaults_to_current_directory() {
  Test::new()
    .create_dir("foo")
    .args(["create", "foo"])
    .success()
    .current_dir("foo")
    .arg("signatures")
    .stdout("[]\n")
    .success();
}
