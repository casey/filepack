use super::*;

#[test]
fn creates_file() {
  let test = Test::new()
    .arg("create")
    .success()
    .args(["message", "--output", "foo"])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("filepack.json"))).unwrap();

  let message = Message {
    fingerprint: manifest.fingerprint(),
    time: None,
  };

  assert_eq!(test.read_bytes("foo"), message.serialize().as_bytes());
}

#[test]
fn with_time() {
  let test = Test::new()
    .arg("create")
    .success()
    .args(["message", "--time", "--output", "foo"])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("filepack.json"))).unwrap();

  let mut message = test.read_bytes("foo");

  let time = u128::from_le_bytes(message[88..104].try_into().unwrap());

  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let one_minute_ago = now - 60 * 1_000_000_000;
  assert!(time >= one_minute_ago && time <= now);

  message[88..104].copy_from_slice(&0u128.to_le_bytes());

  let expected = Message {
    fingerprint: manifest.fingerprint(),
    time: Some(0),
  }
  .serialize();

  assert_eq!(message, expected.as_bytes());
}

#[test]
fn explicit_manifest_path() {
  let test = Test::new()
    .create_dir("bar")
    .args(["create", "bar"])
    .success()
    .args(["message", "--output", "foo", "bar"])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("bar/filepack.json"))).unwrap();

  let message = Message {
    fingerprint: manifest.fingerprint(),
    time: None,
  };

  assert_eq!(test.read_bytes("foo"), message.serialize().as_bytes());
}

#[test]
fn defaults_to_current_directory() {
  let test = Test::new()
    .arg("create")
    .success()
    .args(["message", "--output", "bar"])
    .success();

  let manifest = Manifest::load(Some(&test.path().join("filepack.json"))).unwrap();

  let message = Message {
    fingerprint: manifest.fingerprint(),
    time: None,
  };

  assert_eq!(test.read_bytes("bar"), message.serialize().as_bytes());
}

#[test]
fn manifest_not_found() {
  Test::new()
    .args(["message", "--output", "output.bin"])
    .stderr_regex("error: manifest `.*filepack.json` not found\n")
    .failure();
}
