use super::*;

#[test]
fn http1_is_supported() {
  let server = Test::new().serve().spawn();

  server
    .get(format!("/file/{}", Hash::bytes(b"")))
    .version(Version::HTTP_11)
    .status(StatusCode::NOT_FOUND)
    .send();

  server.terminate().success();
}

#[test]
fn http2_is_supported() {
  let server = Test::new().serve().spawn();

  server
    .get(format!("/file/{}", Hash::bytes(b"")))
    .version(Version::HTTP_2)
    .status(StatusCode::NOT_FOUND)
    .send();

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
      "foo",
      "--redirect",
      "bar",
    ])
    .spawn();

  server
    .get("/baz?qux=quux")
    .header(header::HOST, "bar")
    .status(StatusCode::PERMANENT_REDIRECT)
    .assert_header(header::LOCATION, "http://foo:0/baz?qux=quux")
    .send();

  server
    .get("/baz?qux=quux")
    .host("bar")
    .version(Version::HTTP_2)
    .status(StatusCode::PERMANENT_REDIRECT)
    .assert_header(header::LOCATION, "http://foo:0/baz?qux=quux")
    .send();

  server.get("/").header(header::HOST, "foo").send();

  server.terminate().success();
}

#[test]
fn redirect_http_to_https() {
  #[track_caller]
  fn case(server: &Child, path: &str, location: &str) {
    server
      .get(path)
      .status(StatusCode::PERMANENT_REDIRECT)
      .assert_header(header::LOCATION, location)
      .assert_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
      .send();
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
    ])
    .spawn();

  case(&server, "/", "https://foo.com:0/");
  case(&server, "/bar", "https://foo.com:0/bar");
  case(&server, "/bar?baz=qux", "https://foo.com:0/bar?baz=qux");

  server.terminate().success();
}

#[test]
fn redirect_rejects_canonical_domain() {
  Test::new()
    .args(["serve", "--domain", "foo", "--redirect", "foo"])
    .stderr("error: redirect domain `foo` is the canonical domain\n")
    .failure();
}
