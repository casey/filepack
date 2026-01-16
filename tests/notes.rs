use super::*;

#[test]
fn duplicate_note() {
  let test = Test::new()
    .data_dir("alice")
    .arg("keygen")
    .success()
    .data_dir("bob")
    .arg("keygen")
    .success()
    .arg("create")
    .success()
    .data_dir("alice")
    .arg("sign")
    .success()
    .data_dir("bob")
    .arg("sign")
    .success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let mut iter = manifest.notes[0].signatures.iter();
  let (&first_key, &first_sig) = iter.next().unwrap();
  let (&second_key, &second_sig) = iter.next().unwrap();

  manifest.notes = vec![
    Note {
      signatures: [(first_key, first_sig)].into(),
      time: None,
    },
    Note {
      signatures: [(second_key, second_sig)].into(),
      time: None,
    },
  ];

  manifest.save(&manifest_path).unwrap();

  test
    .arg("verify")
    .stderr("error: note 1 and 2 have the same digest\n")
    .failure()
    .arg("sign")
    .stderr("error: note 1 and 2 have the same digest\n")
    .failure();
}

#[test]
fn duplicate_signature() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .arg("create")
    .success()
    .arg("sign")
    .success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let public_key = test.read("keychain/master.public");

  let first_note = manifest.notes[0].clone();
  manifest.notes.push(first_note);

  manifest.save(&manifest_path).unwrap();

  test
    .arg("verify")
    .stderr(&format!(
      "error: note 1 and 2 both have signatures from key {public_key}\n"
    ))
    .failure()
    .arg("sign")
    .stderr(&format!(
      "error: note 1 and 2 both have signatures from key {public_key}\n"
    ))
    .failure();
}

#[test]
fn invalid_signature() {
  let test = Test::new().arg("keygen").success().arg("create").success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  manifest.notes.push(Note {
    signatures: [(
      PUBLIC_KEY.parse::<PublicKey>().unwrap(),
      "0".repeat(128).parse::<Signature>().unwrap(),
    )]
    .into(),
    time: None,
  });

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
fn unsigned_note() {
  Test::new()
    .write(
      "filepack.json",
      json! {
        files: {},
        notes: [
          {
            signatures: {}
          }
        ]
      },
    )
    .arg("verify")
    .stderr("error: note 1 is unsigned\n")
    .failure()
    .arg("keygen")
    .success()
    .arg("sign")
    .stderr("error: note 1 is unsigned\n")
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

  let public_key = test.read_public_key("keychain/master.public");
  let signature = manifest.notes[0].signatures.remove(&public_key).unwrap();

  manifest.notes[0]
    .signatures
    .insert(PUBLIC_KEY.parse::<PublicKey>().unwrap(), signature);

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
fn time_removal_invalidates_signature() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .args(["create", "--sign", "--time"])
    .success();

  let public_key = test.read("keychain/master.public");

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  assert!(manifest.notes[0].time.is_some());
  manifest.notes[0].time = None;
  manifest.save(&manifest_path).unwrap();

  test
    .arg("verify")
    .stderr_regex(&format!(
      "error: invalid signature for key `{public_key}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure();
}

#[test]
fn time_modification_invalidates_signature() {
  let test = Test::new()
    .arg("keygen")
    .success()
    .args(["create", "--sign", "--time"])
    .success();

  let public_key = test.read("keychain/master.public");

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let time = manifest.notes[0].time.unwrap();
  manifest.notes[0].time = Some(time + 1);
  manifest.save(&manifest_path).unwrap();

  test
    .arg("verify")
    .stderr_regex(&format!(
      "error: invalid signature for key `{public_key}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure();
}
