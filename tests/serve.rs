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

#[test]
fn redirect_alias() {
  let server = Test::new()
    .ready_address()
    .args([
      "serve",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--domain",
      "foo.com",
      "--redirect",
      "bar.com",
    ])
    .spawn();

  let client = reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .unwrap();

  let address = server.address();

  let response = client
    .get(format!("{address}/baz?qux=quux"))
    .header(reqwest::header::HOST, "bar.com")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
  assert_eq!(
    response.headers()[reqwest::header::LOCATION],
    "http://foo.com:0/baz?qux=quux",
  );

  let client = reqwest::blocking::Client::builder()
    .http2_prior_knowledge()
    .redirect(reqwest::redirect::Policy::none())
    .resolve(
      "bar.com",
      format!("127.0.0.1:{}", server.port()).parse().unwrap(),
    )
    .build()
    .unwrap();

  let response = client.get("http://bar.com/baz?qux=quux").send().unwrap();

  assert_eq!(response.version(), Version::HTTP_2);
  assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
  assert_eq!(
    response.headers()[reqwest::header::LOCATION],
    "http://foo.com:0/baz?qux=quux",
  );

  let response = client
    .get(format!("{address}/"))
    .header(reqwest::header::HOST, "foo.com")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  server.terminate().success();
}

#[test]
fn redirect_http_to_https() {
  #[track_caller]
  fn case(client: &reqwest::blocking::Client, address: &str, path: &str, location: &str) {
    let response = client.get(format!("{address}{path}")).send().unwrap();

    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(response.headers()[reqwest::header::LOCATION], location);
    assert_eq!(
      response.headers()[reqwest::header::X_CONTENT_TYPE_OPTIONS],
      "nosniff",
    );
  }

  let server = Test::new()
    .ready_address()
    .args([
      "serve",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--https-port",
      "0",
      "--redirect-http-to-https",
      "--domain",
      "foo.com",
      "--acme-directory",
      rustls_acme::acme::LETS_ENCRYPT_STAGING_DIRECTORY,
    ])
    .spawn();

  let client = reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .unwrap();

  let address = server.address();

  case(&client, &address, "/", "https://foo.com:0/");
  case(&client, &address, "/bar", "https://foo.com:0/bar");
  case(
    &client,
    &address,
    "/bar?baz=qux",
    "https://foo.com:0/bar?baz=qux",
  );

  server.terminate().success();
}

#[test]
fn redirect_rejects_canonical_domain() {
  Test::new()
    .args(["serve", "--domain", "foo.com", "--redirect", "foo.com"])
    .stderr("error: redirect domain `foo.com` is the canonical domain\n")
    .failure();
}
