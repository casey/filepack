use super::*;

#[test]
fn rejects_unsorted_hashes() {
  let server = Test::new().serve().spawn();

  let mut hashes = BTreeSet::from([Hash::bytes(b"foo"), Hash::bytes(b"bar")])
    .into_iter()
    .collect::<Vec<_>>();

  hashes.reverse();

  let mut encoder = Encoder::new();
  let mut map = encoder.map::<u64>(1);
  map.item(0, hashes);
  drop(map);

  let response = reqwest::blocking::Client::new()
    .post(format!("{}/missing", server.address()))
    .body(encoder.finish())
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);

  server.terminate().success();
}

#[test]
fn returns_missing_hashes() {
  let server = Test::new().serve().spawn();

  let present = Hash::bytes(b"bar");
  let absent = Hash::bytes(b"baz");

  reqwest::blocking::Client::new()
    .put(format!("{}/file/{present}", server.address()))
    .body("bar")
    .send()
    .unwrap();

  let hashes = BTreeSet::from([present, absent]);

  let response = reqwest::blocking::Client::new()
    .post(format!("{}/missing", server.address()))
    .body(
      api::missing::Request {
        hashes: hashes.into(),
      }
      .encode_to_vec(),
    )
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let bytes = response.bytes().unwrap();

  let missing = api::missing::Response::decode_from_slice(&bytes).unwrap();

  assert_eq!(missing.hashes.into_inner(), vec![absent]);

  server.terminate().success();
}
