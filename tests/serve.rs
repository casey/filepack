use {super::*, reqwest::Version};

#[test]
fn http1_is_supported() {
  let server = Test::new()
    .args(["serve", "--address", "127.0.0.1:0"])
    .ready_fd()
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
    .ready_fd()
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

#[test]
fn ready_fd_must_not_conflict_with_standard_streams() {
  Test::new()
    .args(["serve", "--address", "127.0.0.1:0", "--ready-fd", "2"])
    .stderr_regex(
      r"error: invalid value '2' for '--ready-fd <READY_FD>': 2 is not in 3\.\.=2147483647\n\n.*",
    )
    .status(2);
}
