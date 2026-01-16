use super::*;

#[test]
fn duplicate_note() {
  let test = Test::new()
    .data_dir("alice")
    .args(["keygen"])
    .success()
    .data_dir("bob")
    .args(["keygen"])
    .success()
    .args(["create"])
    .success()
    .data_dir("alice")
    .args(["sign"])
    .success()
    .data_dir("bob")
    .args(["sign"])
    .success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let mut iter = manifest.notes[0].signatures.iter();
  let (&first_key, &first_sig) = iter.next().unwrap();
  let (&second_key, &second_sig) = iter.next().unwrap();

  manifest.notes = vec![
    Note {
      signatures: [(first_key, first_sig)].into(),
    },
    Note {
      signatures: [(second_key, second_sig)].into(),
    },
  ];

  manifest.save(&manifest_path).unwrap();

  test
    .args(["verify"])
    .stderr("error: note 1 and 2 have the same digest\n")
    .failure()
    .args(["sign"])
    .stderr("error: note 1 and 2 have the same digest\n")
    .failure();
}

#[test]
fn duplicate_signature() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .args(["create"])
    .success()
    .args(["sign"])
    .success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  let public_key = test.read("keychain/master.public");

  let first_note = manifest.notes[0].clone();
  manifest.notes.push(first_note);

  manifest.save(&manifest_path).unwrap();

  test
    .args(["verify"])
    .stderr(&format!(
      "error: note 1 and 2 both have signatures from key {public_key}\n"
    ))
    .failure()
    .args(["sign"])
    .stderr(&format!(
      "error: note 1 and 2 both have signatures from key {public_key}\n"
    ))
    .failure();
}

#[test]
fn invalid_signature() {
  let test = Test::new()
    .args(["keygen"])
    .success()
    .args(["create"])
    .success();

  let manifest_path = test.path().join("filepack.json");
  let mut manifest = Manifest::load(Some(&manifest_path)).unwrap();

  manifest.notes.push(Note {
    signatures: [(
      PUBLIC_KEY.parse::<PublicKey>().unwrap(),
      "0".repeat(128).parse::<Signature>().unwrap(),
    )]
    .into(),
  });

  manifest.save(&manifest_path).unwrap();

  test
    .args(["verify"])
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure()
    .args(["sign"])
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure();
}

#[test]
fn valid_signature_for_wrong_pubkey() {
  let test = Test::new()
    .args(["keygen"])
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
    .args(["verify"])
    .stderr_regex(&format!(
      "error: invalid signature for key `{PUBLIC_KEY}`\n\
      .*Verification equation was not satisfied.*"
    ))
    .failure()
    .args(["sign"])
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
    .args(["verify"])
    .stderr("error: note 1 is unsigned\n")
    .failure()
    .args(["keygen"])
    .success()
    .args(["sign"])
    .stderr("error: note 1 is unsigned\n")
    .failure();
}
