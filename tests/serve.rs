use {super::*, reqwest::Version, reqwest::header};

#[test]
fn download_response_includes_content_length() {
  let hash = Hash::bytes(b"bar");

  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_address()
    .write(&format!("files/{hash}"), "bar")
    .spawn();

  let response = reqwest::blocking::Client::new()
    .get(format!("{}/{hash}", server.address()))
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get(header::CONTENT_LENGTH).unwrap(), "3");
  assert_eq!(response.bytes().unwrap().as_ref(), b"bar");

  server.terminate().success();
}

#[test]
fn http1_is_supported() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_address()
    .spawn();

  let response = reqwest::blocking::Client::builder()
    .http1_only()
    .build()
    .unwrap()
    .get(format!("{}/{}", server.address(), Hash::bytes(b"")))
    .send()
    .unwrap();

  assert_eq!(response.version(), Version::HTTP_11);
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  server.terminate().success();
}

#[test]
fn http2_is_supported() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_address()
    .spawn();

  let response = reqwest::blocking::Client::builder()
    .http2_prior_knowledge()
    .build()
    .unwrap()
    .get(format!("{}/{}", server.address(), Hash::bytes(b"")))
    .send()
    .unwrap();

  assert_eq!(response.version(), Version::HTTP_2);
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  server.terminate().success();
}
