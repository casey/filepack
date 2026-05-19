use {super::*, reqwest::Version};

#[test]
fn http1_is_supported() {
  let server = Test::new().serve().spawn();

  let response = reqwest::blocking::Client::builder()
    .http1_only()
    .build()
    .unwrap()
    .get(format!("{}/file/{}", server.address(), Hash::bytes(b"")))
    .send()
    .unwrap();

  assert_eq!(response.version(), Version::HTTP_11);
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  server.terminate().success();
}

#[test]
fn http2_is_supported() {
  let server = Test::new().serve().spawn();

  let response = reqwest::blocking::Client::builder()
    .http2_prior_knowledge()
    .build()
    .unwrap()
    .get(format!("{}/file/{}", server.address(), Hash::bytes(b"")))
    .send()
    .unwrap();

  assert_eq!(response.version(), Version::HTTP_2);
  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  server.terminate().success();
}
