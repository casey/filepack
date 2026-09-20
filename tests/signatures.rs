use super::*;

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

#[test]
fn invalid_signature_error() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success();

  let manifest_path = test.path().join("foo/manifest.filepack");

  let fingerprint = Loader::load(Some(&manifest_path))
    .unwrap()
    .fingerprint()
    .unwrap()
    .to_string();

  let hex = &fingerprint["package1".len()..];

  let tampered = format!(
    "{}{}",
    if hex.starts_with('0') { '1' } else { '0' },
    &hex[1..],
  );

  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let signature = manifest.signatures.pop_first().unwrap().to_string();

  manifest
    .signatures
    .insert(signature.replacen(hex, &tampered, 1).parse().unwrap());

  manifest.save(&manifest_path).unwrap();

  test
    .args(["signatures", "foo"])
    .stderr_regex(
      "
        error: invalid signature for key `public1[0-9a-f]{64}`
               ├─ signature error
               └─ Verification equation was not satisfied
      ",
    )
    .failure();
}

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
    .args(["sign", "--timestamp", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["signatures", "--format", "json", "foo"])
    .stdout_regex(&format!(
      r#"\[\{{"public_key":"{public_key}","timestamp":\d+\}}\]\n"#,
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
      "[{{\"public_key\":\"{public_key}\",\"timestamp\":null}}]\n"
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
    .args(["sign", "--timestamp", "foo"])
    .success();

  let public_key = test.read("keychain/master.public");

  test
    .args(["signatures", "--format", "tsv", "foo"])
    .stdout_regex(&format!("{public_key}\t\\d+\n"))
    .success();
}
