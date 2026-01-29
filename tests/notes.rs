use super::*;

#[test]
fn invalid_signature() {
  let test = Test::new().arg("keygen").success().arg("create").success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  manifest.signatures.insert(SIGNATURE.parse().unwrap());

  manifest.save(&manifest_path).unwrap();

  test
    .arg("verify")
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure()
    .arg("sign")
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure();
}

#[test]
fn valid_signature_for_wrong_pubkey() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .args(["create", "--sign"])
    .success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  manifest.notes[0].signatures.clear();
  manifest.notes[0]
    .signatures
    .insert(SIGNATURE.parse::<Signature>().unwrap());

  manifest.save(&manifest_path).unwrap();

  test
    .arg("verify")
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure()
    .arg("sign")
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure();
}
