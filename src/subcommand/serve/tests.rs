use {
  super::*,
  axum::{
    body,
    http::{Method, Request, header::HeaderName},
  },
  tower::ServiceExt,
};

struct TestRequestBuilder {
  body: Option<String>,
  method: Method,
  path: String,
  response_body: Body,
  response_headers: BTreeMap<String, String>,
  router: Router,
  status: StatusCode,
  token: Option<String>,
}

impl TestRequestBuilder {
  fn assert_body(mut self, body: impl AsRef<[u8]>) -> Self {
    self.response_body = Body::from(body.as_ref().to_vec());
    self
  }

  fn assert_header(mut self, name: HeaderName, value: impl Into<String>) -> Self {
    assert!(
      self
        .response_headers
        .insert(name.to_string(), value.into())
        .is_none()
    );
    self
  }

  fn assert_response(mut self, response: impl IntoResponse) -> Self {
    let (parts, body) = response.into_response().into_parts();
    self.status = parts.status;
    for (name, value) in parts.headers {
      self = self.assert_header(name.unwrap(), value.to_str().unwrap());
    }
    self.response_body = body;
    self
  }

  fn assert_static(self, path: &str) -> Self {
    self.assert_response(StaticAsset::get(path).unwrap())
  }

  fn body(mut self, body: &str) -> Self {
    self.body = Some(body.into());
    self
  }

  fn new(method: Method, path: impl Into<String>, router: Router) -> Self {
    Self {
      body: None,
      method,
      path: path.into(),
      response_body: Body::empty(),
      response_headers: BTreeMap::from([(
        header::X_CONTENT_TYPE_OPTIONS.to_string(),
        "nosniff".into(),
      )]),
      router,
      status: StatusCode::OK,
      token: None,
    }
  }

  async fn send(self) {
    let mut request = Request::builder().method(self.method).uri(self.path);

    if let Some(token) = self.token {
      request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = self
      .router
      .oneshot(
        request
          .body(if let Some(body) = self.body {
            Body::from(body)
          } else {
            Body::empty()
          })
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), self.status);

    let headers = response.headers();

    for (name, value) in self.response_headers {
      assert_eq!(headers[name], value);
    }

    let body = body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    let expected = body::to_bytes(self.response_body, usize::MAX)
      .await
      .unwrap();
    assert_eq!(body, expected);
  }

  fn status(mut self, status: StatusCode) -> Self {
    self.status = status;
    self
  }

  fn token(mut self, token: String) -> Self {
    self.token = Some(token);
    self
  }
}

struct TestServer {
  data_dir: Utf8PathBuf,
  router: Router,
  #[allow(unused)]
  tempdir: TempDir,
}

impl TestServer {
  #[track_caller]
  fn assert_file(&self, hash: Hash) {
    let contents = fs::read(self.data_dir.join("files").join(hash.to_string())).unwrap();
    assert_eq!(Hash::bytes(&contents), hash);
  }

  #[track_caller]
  fn assert_incoming_empty(&self) {
    assert_eq!(
      fs::read_dir(self.data_dir.join("incoming"))
        .unwrap()
        .count(),
      0,
    );
  }

  fn get(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::GET, path, self.router.clone())
  }

  fn new() -> Self {
    Self::with_auth(None)
  }

  fn post(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::POST, path, self.router.clone())
  }

  fn put(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::PUT, path, self.router.clone())
  }

  fn with_auth(auth_config: Option<Arc<AuthConfig>>) -> Self {
    let (tempdir, data_dir) = tempdir();

    let server = Arc::new(Server::with_data_dir(&data_dir).unwrap());

    let router = Serve::router(server, auth_config);

    Self {
      data_dir,
      router,
      tempdir,
    }
  }

  fn write_file(&self, content: &[u8]) {
    fs::write(
      self
        .data_dir
        .join("files")
        .join(Hash::bytes(content).to_string()),
      content,
    )
    .unwrap();
  }
}

#[test]
fn admin_key_requires_restrict_upload() {
  let err = Serve::try_parse_from(["filepack", "--admin-key", test::PUBLIC_KEY]).unwrap_err();
  assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
}

#[tokio::test]
async fn closed_server_forbids_uploads() {
  TestServer::with_auth(Some(Arc::new(AuthConfig {
    admin: None,
    audiences: Vec::new(),
  })))
  .put(format!("/file/{}", Hash::bytes(b"bar")))
  .body("bar")
  .status(StatusCode::FORBIDDEN)
  .assert_body("uploads forbidden")
  .send()
  .await;
}

#[test]
fn default_serve_matches_parsed() {
  assert_eq!(
    Serve::default(),
    Serve::try_parse_from(["filepack"]).unwrap(),
  );
}

fn directory(entries: &[(&str, EntryType, Hash, u64)]) -> Directory {
  Directory {
    version: Version::Zero,
    entries: entries
      .iter()
      .map(|(name, ty, hash, size)| {
        (
          name.parse().unwrap(),
          Entry {
            ty: *ty,
            hash: *hash,
            size: *size,
          },
        )
      })
      .collect(),
  }
}

#[test]
fn domain_defaults_to_hostname() {
  assert_eq!(
    Serve::default().domains().unwrap(),
    vec![System::host_name().unwrap()]
  );
}

#[test]
fn domain_flag_is_respected() {
  assert_eq!(
    Serve {
      domains: vec!["foo".into(), "bar".into()],
      ..Serve::default()
    }
    .domains()
    .unwrap(),
    vec!["foo".to_string(), "bar".to_string()],
  );
}

#[tokio::test]
async fn download_response() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"bar");
  server.write_file(b"bar");

  server
    .get(format!("/file/{hash}"))
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_DISPOSITION, "attachment")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
    .assert_header(header::CONTENT_TYPE, "application/octet-stream")
    .assert_header(header::ETAG, format!("\"{hash}\""))
    .assert_body("bar")
    .send()
    .await;
}

#[tokio::test]
async fn fallback() {
  TestServer::new()
    .get("/nonexistent")
    .assert_static("404.html")
    .status(StatusCode::NOT_FOUND)
    .send()
    .await;
}

#[tokio::test]
async fn favicon() {
  TestServer::new()
    .get("/favicon.ico")
    .assert_static("favicon.png")
    .send()
    .await;
}

#[tokio::test]
async fn files_empty() {
  TestServer::new()
    .get("/files")
    .assert_response(FilesHtml { files: Vec::new() })
    .send()
    .await;
}

#[tokio::test]
async fn files_non_empty() {
  let server = TestServer::new();

  let foo = b"foo";
  let bar = b"bar";
  let baz = b"baz";

  server.write_file(foo);
  server.write_file(bar);
  server.write_file(baz);

  fs::write(server.data_dir.join("files").join("not-a-hash"), "").unwrap();

  let mut files = vec![Hash::bytes(foo), Hash::bytes(bar), Hash::bytes(baz)];
  files.sort();

  server
    .get("/files")
    .assert_response(FilesHtml { files })
    .send()
    .await;
}

#[tokio::test]
async fn get_directory_not_found() {
  let server = TestServer::new();

  let cbor = directory(&[]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  server.write_file(&cbor);

  server
    .get(format!("/directory/{hash}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("directory {hash} not found"))
    .send()
    .await;
}

#[tokio::test]
async fn get_directory_succeeds() {
  let server = TestServer::new();

  let dir = directory(&[]);
  let cbor = dir.encode_to_vec();
  let hash = Hash::bytes(&cbor);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send().await;

  server
    .get(format!("/directory/{hash}"))
    .assert_response(DirectoryHtml {
      directory: dir,
      hash,
    })
    .send()
    .await;
}

#[tokio::test]
async fn home() {
  TestServer::new()
    .get("/")
    .assert_static("index.html")
    .send()
    .await;
}

#[tokio::test]
async fn install_script() {
  TestServer::new()
    .get("/install.sh")
    .assert_static("install.sh")
    .send()
    .await;
}

#[test]
fn ports() {
  #[track_caller]
  fn case(serve: Serve, http_port: Option<u16>, https_port: Option<u16>) {
    assert_eq!(serve.http_port(), http_port);
    assert_eq!(serve.https_port(), https_port);
  }

  case(Serve::default(), Some(80), None);
  case(
    Serve {
      https: true,
      ..Serve::default()
    },
    None,
    Some(443),
  );
  case(
    Serve {
      https_port: Some(433),
      ..Serve::default()
    },
    None,
    Some(433),
  );
  case(
    Serve {
      http: true,
      https: true,
      ..Serve::default()
    },
    Some(80),
    Some(443),
  );
  case(
    Serve {
      http_port: Some(8080),
      https_port: Some(8443),
      ..Serve::default()
    },
    Some(8080),
    Some(8443),
  );
  case(
    Serve {
      redirect_http_to_https: true,
      ..Serve::default()
    },
    Some(80),
    Some(443),
  );
}

#[test]
fn redirect_destination() {
  let domains = vec!["foo".to_string()];

  assert_eq!(Serve::redirect_destination(&domains, 443), "https://foo");
  assert_eq!(
    Serve::redirect_destination(&domains, 8443),
    "https://foo:8443",
  );
}

#[tokio::test]
async fn redirect_http_to_https() {
  async fn case(path: &str, location: &str) {
    let response = Serve::redirect_router("https://foo".into())
      .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], location);
    assert_eq!(
      response.headers()[header::X_CONTENT_TYPE_OPTIONS],
      "nosniff"
    );
  }

  case("/", "https://foo/").await;
  case("/bar", "https://foo/bar").await;
  case("/bar?baz=qux", "https://foo/bar?baz=qux").await;
}

#[tokio::test]
async fn restricted_upload_accepts_admin_token() {
  let admin = PrivateKey::generate();
  let hash = Hash::bytes(b"bar");
  let token = Token::encode(&admin, "filepack.example").unwrap();

  let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
    admin: Some(admin.public_key()),
    audiences: vec!["filepack.example".into()],
  })));

  server
    .put(format!("/file/{hash}"))
    .body("bar")
    .token(token)
    .send()
    .await;

  server.assert_file(hash);
}

#[tokio::test]
async fn restricted_upload_rejects_missing_header() {
  let admin = PrivateKey::generate();
  let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
    admin: Some(admin.public_key()),
    audiences: vec!["filepack.example".into()],
  })));

  let hash = Hash::bytes(b"bar");

  server
    .put(format!("/file/{hash}"))
    .body("bar")
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("missing authorization header")
    .send()
    .await;
}

#[tokio::test]
async fn restricted_upload_rejects_others() {
  let admin = PrivateKey::generate();
  let other = PrivateKey::generate();
  let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
    admin: Some(admin.public_key()),
    audiences: vec!["filepack.example".into()],
  })));

  let hash = Hash::bytes(b"bar");
  let token = Token::encode(&other, "filepack.example").unwrap();

  server
    .put(format!("/file/{hash}"))
    .body("bar")
    .token(token)
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("invalid authorization token")
    .send()
    .await;
}

#[tokio::test]
async fn static_files() {
  TestServer::new()
    .get("/static/index.css")
    .assert_static("index.css")
    .send()
    .await;
}

#[tokio::test]
async fn upload_creates_file() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"bar");

  server.put(format!("/file/{hash}")).body("bar").send().await;

  server.assert_file(hash);

  server.assert_incoming_empty();
}

#[tokio::test]
async fn upload_short_circuits_when_file_exists() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"bar");

  server.write_file(b"bar");

  server.put(format!("/file/{hash}")).body("bar").send().await;

  server.assert_file(hash);

  server.assert_incoming_empty();
}

#[tokio::test]
async fn upload_with_wrong_hash_fails() {
  let server = TestServer::new();

  let actual = Hash::bytes(b"bar");
  let expected = Hash::bytes(b"baz");

  server
    .put(format!("/file/{expected}"))
    .body("bar")
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "expected upload with hash {expected} but got {actual}"
    ))
    .send()
    .await;

  server.assert_incoming_empty();
}

#[tokio::test]
async fn verify_directory_decode_error() {
  let server = TestServer::new();

  let junk = b"junk";
  let hash = Hash::bytes(junk);
  server.write_file(junk);

  server
    .post(format!("/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!("failed to decode directory {hash}"))
    .send()
    .await;
}

#[tokio::test]
async fn verify_directory_file_not_found() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"foo");

  server
    .post(format!("/directory/{hash}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("file with hash {hash} not found"))
    .send()
    .await;
}

#[tokio::test]
async fn verify_directory_missing_file() {
  let server = TestServer::new();

  let missing = Hash::bytes(b"foo");
  let cbor = directory(&[("foo", EntryType::File, missing, 3)]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  server.write_file(&cbor);

  server
    .post(format!("/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {hash} references missing file {missing}"
    ))
    .send()
    .await;
}

#[tokio::test]
async fn verify_directory_rejects_missing_auth_header() {
  let admin = PrivateKey::generate();
  let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
    admin: Some(admin.public_key()),
    audiences: vec!["filepack.example".into()],
  })));

  let hash = Hash::bytes(b"foo");

  server
    .post(format!("/directory/{hash}"))
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("missing authorization header")
    .send()
    .await;
}

#[tokio::test]
async fn verify_directory_succeeds() {
  let server = TestServer::new();

  let file = b"foo";
  let file_hash = Hash::bytes(file);
  server.write_file(file);

  let child_cbor =
    directory(&[("foo", EntryType::File, file_hash, file.len() as u64)]).encode_to_vec();
  let child_hash = Hash::bytes(&child_cbor);
  server.write_file(&child_cbor);

  server.post(format!("/directory/{child_hash}")).send().await;

  let parent_cbor = directory(&[(
    "child",
    EntryType::Directory,
    child_hash,
    child_cbor.len() as u64,
  )])
  .encode_to_vec();
  let parent_hash = Hash::bytes(&parent_cbor);
  server.write_file(&parent_cbor);

  server
    .post(format!("/directory/{parent_hash}"))
    .send()
    .await;
}

#[tokio::test]
async fn verify_directory_unverified_subdirectory() {
  let server = TestServer::new();

  let child_cbor = directory(&[]).encode_to_vec();
  let child_hash = Hash::bytes(&child_cbor);
  server.write_file(&child_cbor);

  let parent_cbor = directory(&[(
    "child",
    EntryType::Directory,
    child_hash,
    child_cbor.len() as u64,
  )])
  .encode_to_vec();
  let parent_hash = Hash::bytes(&parent_cbor);
  server.write_file(&parent_cbor);

  server
    .post(format!("/directory/{parent_hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {parent_hash} references unverified subdirectory {child_hash}"
    ))
    .send()
    .await;
}
