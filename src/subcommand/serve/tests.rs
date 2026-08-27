use {
  super::*,
  axum::{
    body,
    http::{Method, Request, header::HeaderName},
  },
  tokio::runtime::Runtime,
  tower::ServiceExt,
};

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[derive(Default)]
struct DirectoryBuilder<'a> {
  entries: BTreeMap<&'a str, DirectoryBuilderEntry<'a>>,
}

impl<'a> DirectoryBuilder<'a> {
  fn build(&self) -> Directory {
    let mut directory = Directory::new();

    for (name, entry) in &self.entries {
      match entry {
        DirectoryBuilderEntry::Directory(child) => {
          directory.insert_directory(name, &child.build());
        }
        DirectoryBuilderEntry::File(content) => {
          directory.insert_file(name, content);
        }
      }
    }

    directory
  }

  fn insert(&mut self, path: &[&'a str], content: &[u8]) {
    let (first, rest) = path.split_first().unwrap();

    if rest.is_empty() {
      assert!(
        self
          .entries
          .insert(first, DirectoryBuilderEntry::File(content.to_vec()))
          .is_none()
      );
    } else {
      let entry = self
        .entries
        .entry(first)
        .or_insert_with(|| DirectoryBuilderEntry::Directory(DirectoryBuilder::default()));

      match entry {
        DirectoryBuilderEntry::Directory(child) => child.insert(rest, content),
        DirectoryBuilderEntry::File(_) => panic!("file name `{first}` conflicts with directory"),
      }
    }
  }

  fn upload(&self, server: &TestServer) -> Hash {
    for entry in self.entries.values() {
      match entry {
        DirectoryBuilderEntry::Directory(child) => {
          child.upload(server);
        }
        DirectoryBuilderEntry::File(content) => {
          server.write_file(content);
        }
      }
    }

    let (cbor, hash) = self.build().cbor();

    server.write_file(&cbor);
    server.post(format!("/api/directory/{hash}")).send();

    hash
  }
}

enum DirectoryBuilderEntry<'a> {
  Directory(DirectoryBuilder<'a>),
  File(Vec<u8>),
}

#[derive(Default)]
struct PackageBuilder<'a> {
  root: DirectoryBuilder<'a>,
}

impl<'a> PackageBuilder<'a> {
  fn directory(&self) -> Directory {
    self.root.build()
  }

  fn file(mut self, path: &'a str, content: &[u8]) -> Self {
    let path = path.split('/').collect::<Vec<&str>>();
    self.root.insert(&path, content);
    self
  }

  fn fingerprint(&self) -> Fingerprint {
    Fingerprint(self.directory().cbor().1)
  }

  fn metadata(self, metadata: &Metadata) -> Self {
    self.file(Metadata::CBOR_FILENAME, &metadata.encode_to_vec())
  }

  fn new() -> Self {
    Self::default()
  }

  fn upload(self, server: &TestServer) -> Fingerprint {
    let fingerprint = Fingerprint(self.root.upload(server));

    server.post(format!("/api/package/{fingerprint}")).send();

    fingerprint
  }
}

struct TestRequestBuilder {
  absent_headers: BTreeSet<String>,
  body: Option<Vec<u8>>,
  method: Method,
  path: String,
  range: Option<&'static str>,
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

  fn assert_header_absent(mut self, name: HeaderName) -> Self {
    assert!(self.absent_headers.insert(name.to_string()));
    self
  }

  fn assert_page(self, page: impl Page) -> Self {
    self.assert_response(page.page(None))
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

  fn body(mut self, body: impl AsRef<[u8]>) -> Self {
    self.body = Some(body.as_ref().to_vec());
    self
  }

  fn new(method: Method, path: impl Into<String>, router: Router) -> Self {
    Self {
      absent_headers: BTreeSet::new(),
      body: None,
      method,
      path: path.into(),
      range: None,
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

  fn range(mut self, range: &'static str) -> Self {
    self.range = Some(range);
    self
  }

  fn send(self) {
    RUNTIME.block_on(async move {
      let mut request = Request::builder().method(self.method).uri(self.path);

      if let Some(token) = self.token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
      }

      if let Some(range) = self.range {
        request = request.header(header::RANGE, range);
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

      for name in self.absent_headers {
        assert!(
          !headers.contains_key(name.as_str()),
          "unexpected header {name}"
        );
      }

      let body = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
      let expected = body::to_bytes(self.response_body, usize::MAX)
        .await
        .unwrap();
      assert_eq!(body, expected);
    });
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

  fn builder() -> TestServerBuilder {
    TestServerBuilder {
      auth_config: None,
      mounts: HashSet::new(),
      url: None,
    }
  }

  fn delete(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::DELETE, path, self.router.clone())
  }

  fn get(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::GET, path, self.router.clone())
  }

  fn new() -> Self {
    Self::builder().build()
  }

  fn post(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::POST, path, self.router.clone())
  }

  fn put(&self, path: impl Into<String>) -> TestRequestBuilder {
    TestRequestBuilder::new(Method::PUT, path, self.router.clone())
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

struct TestServerBuilder {
  auth_config: Option<Arc<AuthConfig>>,
  mounts: HashSet<Fingerprint>,
  url: Option<Url>,
}

impl TestServerBuilder {
  fn auth_config(mut self, auth_config: AuthConfig) -> Self {
    self.auth_config = Some(Arc::new(auth_config));
    self
  }

  fn build(self) -> TestServer {
    let (tempdir, data_dir) = tempdir();

    let server = Arc::new(Server::with_data_dir(&data_dir).unwrap());

    let router = Serve::router(
      server,
      self.auth_config,
      None,
      Arc::new(ServerConfig {
        mounts: self.mounts,
        url: self.url,
      }),
    );

    TestServer {
      data_dir,
      router,
      tempdir,
    }
  }

  fn mount(mut self, fingerprint: Fingerprint) -> Self {
    self.mounts.insert(fingerprint);
    self
  }

  fn url(mut self, url: Url) -> Self {
    self.url = Some(url);
    self
  }
}

#[test]
fn admin_key_requires_restrict_writes() {
  let err = Serve::try_parse_from(["filepack", "--admin-key", test::PUBLIC_KEY]).unwrap_err();
  assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
}

#[test]
fn api_packages_returns_package_fingerprints() {
  let server = TestServer::new();

  server
    .get("/api/packages")
    .assert_body(
      api::packages::Response {
        packages: BTreeSet::new().into(),
      }
      .encode_to_vec(),
    )
    .send();

  let foo = PackageBuilder::new().file("foo", b"foo").upload(&server);
  let bar = PackageBuilder::new().file("bar", b"bar").upload(&server);

  server
    .get("/api/packages")
    .assert_body(
      api::packages::Response {
        packages: BTreeSet::from([foo, bar]).into(),
      }
      .encode_to_vec(),
    )
    .send();
}

#[test]
fn artwork_missing() {
  let server = TestServer::new();

  let artwork = b"foo";
  server.write_file(artwork);

  let metadata = Metadata {
    artwork: Some(Image::test("cover.png")),
    ..Metadata::default()
  };
  let metadata_cbor = metadata.encode_to_vec();
  server.write_file(&metadata_cbor);

  let (cbor, hash) = Directory::new().insert_file("cover.png,", artwork).cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/package/{fingerprint}")).send();

  let mut corrupt = Directory::new();
  corrupt.insert_file(Metadata::CBOR_FILENAME, &metadata_cbor);

  let corrupt = corrupt.encode_to_vec();
  fs::write(
    server.data_dir.join("files").join(hash.to_string()),
    &corrupt,
  )
  .unwrap();

  server
    .get(format!("/artwork/{fingerprint}"))
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .assert_body(format!(
      "file `cover.png` missing from package {fingerprint}",
    ))
    .send();
}

#[test]
fn artwork_not_found_without_artwork() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata::default())
    .upload(&server);

  server
    .get(format!("/artwork/{fingerprint}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} artwork not found"))
    .send();
}

#[test]
fn artwork_package_not_found() {
  let server = TestServer::new();

  let fingerprint = Fingerprint(Hash::bytes(b"foo"));

  server
    .get(format!("/artwork/{fingerprint}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not found"))
    .send();
}

#[test]
fn artwork_response() {
  #[track_caller]
  fn case(filename: &str, content_type: &str) {
    let server = TestServer::new();

    let artwork = b"foo";
    let artwork_hash = Hash::bytes(artwork);
    server.write_file(artwork);

    let metadata = Metadata {
      artwork: Some(Image::test(filename)),
      ..Metadata::default()
    };
    let metadata_cbor = metadata.encode_to_vec();
    server.write_file(&metadata_cbor);

    let (cbor, hash) = Directory::new()
      .insert_file(filename, artwork)
      .insert_file(Metadata::CBOR_FILENAME, &metadata_cbor)
      .cbor();

    let fingerprint = Fingerprint(hash);
    server.write_file(&cbor);

    server.post(format!("/api/directory/{hash}")).send();
    server.post(format!("/api/package/{fingerprint}")).send();

    server
      .get(format!("/artwork/{fingerprint}"))
      .assert_header(header::ACCEPT_RANGES, "bytes")
      .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
      .assert_header(header::CONTENT_LENGTH, "3")
      .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
      .assert_header(header::CONTENT_TYPE, content_type)
      .assert_header(header::ETAG, format!("\"{artwork_hash}\""))
      .assert_body(artwork)
      .send();

    server
      .get(format!("/artwork/{fingerprint}"))
      .range("bytes=1-2")
      .status(StatusCode::PARTIAL_CONTENT)
      .assert_header(header::ACCEPT_RANGES, "bytes")
      .assert_header(header::CONTENT_RANGE, "bytes 1-2/3")
      .assert_header(header::CONTENT_LENGTH, "2")
      .assert_body("oo")
      .send();
  }

  case("cover.png", "image/png");
  case("cover.jpg", "image/jpeg");
}

#[test]
fn artwork_thumbnail_response() {
  let server = TestServer::new();

  let artwork: &[u8] = b"foo";
  let thumbnail: &[u8] = b"barbar";

  let with_thumbnail = PackageBuilder::new()
    .metadata(&Metadata {
      artwork: Some(Image::test("cover.png")),
      thumbnails: Some(
        [(
          "cover.png".parse().unwrap(),
          Image::test("thumbnails/cover.jpg"),
        )]
        .into(),
      ),
      ..default()
    })
    .file("cover.png", artwork)
    .file("thumbnails/cover.jpg", thumbnail)
    .upload(&server);

  server
    .get(format!("/artwork/{with_thumbnail}/thumbnail"))
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "6")
    .assert_header(header::CONTENT_TYPE, "image/jpeg")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(thumbnail)))
    .assert_body(thumbnail)
    .send();

  let without_thumbnail = PackageBuilder::new()
    .metadata(&Metadata {
      artwork: Some(Image::test("cover.png")),
      ..default()
    })
    .file("cover.png", artwork)
    .upload(&server);

  server
    .get(format!("/artwork/{without_thumbnail}/thumbnail"))
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_TYPE, "image/png")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(artwork)))
    .assert_body(artwork)
    .send();
}

#[test]
fn closed_server_forbids_writes() {
  TestServer::builder()
    .auth_config(AuthConfig {
      admin: None,
      audience: None,
    })
    .build()
    .put(format!("/file/{}", Hash::bytes(b"bar")))
    .body("bar")
    .status(StatusCode::FORBIDDEN)
    .assert_body("writes forbidden")
    .send();
}

#[test]
fn default_serve_matches_parsed() {
  assert_eq!(
    Serve::default(),
    Serve::try_parse_from(["filepack"]).unwrap(),
  );
}

#[test]
fn delete_package_not_found() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new().fingerprint();

  server
    .delete(format!("/api/package/{fingerprint}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not found"))
    .send();
}

#[test]
fn delete_package_rejects_missing_auth_header() {
  let admin = PrivateKey::generate();
  let server = TestServer::builder()
    .auth_config(AuthConfig {
      admin: Some(admin.public_key()),
      audience: Some("filepack.example".into()),
    })
    .build();

  let fingerprint = PackageBuilder::new().fingerprint();

  server
    .delete(format!("/api/package/{fingerprint}"))
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("missing authorization header")
    .send();
}

#[test]
fn delete_package_removes_package() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new().file("foo", b"foo").upload(&server);

  server.delete(format!("/api/package/{fingerprint}")).send();

  server
    .get("/api/packages")
    .assert_body(
      api::packages::Response {
        packages: BTreeSet::new().into(),
      }
      .encode_to_vec(),
    )
    .send();
}

#[test]
fn domain_required_for_canonical_domain_options() {
  #[track_caller]
  fn case(args: &[&str]) {
    let err = Serve::try_parse_from(["filepack"].iter().chain(args)).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
  }

  case(&["--https"]);
  case(&["--https-port", "443"]);
  case(&["--redirect", "bar"]);
  case(&["--redirect-http-to-https"]);
}

#[test]
fn domains_include_redirects() {
  assert_eq!(
    Serve {
      domain: Some("foo".into()),
      redirects: vec!["bar".into()],
      ..Serve::default()
    }
    .domains(),
    vec!["foo".to_string(), "bar".to_string()],
  );
}

#[test]
fn download_range() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"foobarbaz");
  server.write_file(b"foobarbaz");

  server
    .get(format!("/file/{hash}"))
    .range("bytes=0-2")
    .status(StatusCode::PARTIAL_CONTENT)
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CONTENT_DISPOSITION, "attachment")
    .assert_header(header::CONTENT_RANGE, "bytes 0-2/9")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_body("foo")
    .send();
}

#[test]
fn download_response() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"bar");
  server.write_file(b"bar");

  server
    .get(format!("/file/{hash}"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_DISPOSITION, "attachment")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
    .assert_header(header::CONTENT_TYPE, "application/octet-stream")
    .assert_header(header::ETAG, format!("\"{hash}\""))
    .assert_body("bar")
    .send();
}

#[test]
fn fallback() {
  TestServer::new()
    .get("/nonexistent")
    .assert_static("404.html")
    .status(StatusCode::NOT_FOUND)
    .send();
}

#[test]
fn favicon() {
  TestServer::new()
    .get("/favicon.ico")
    .assert_static("favicon.png")
    .send();
}

#[test]
fn file_with_path_inline() {
  let server = TestServer::new();

  server.write_file(b"foo");

  server
    .get(format!("/file/{}/bar.png", Hash::bytes(b"foo")))
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_TYPE, "image/png")
    .assert_header_absent(header::CONTENT_DISPOSITION)
    .assert_body("foo")
    .send();
}

#[test]
fn file_with_path_markdown() {
  let server = TestServer::new();

  server.write_file(b"foo");

  server
    .get(format!("/file/{}/bar.md", Hash::bytes(b"foo")))
    .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
    .assert_header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
    .assert_header_absent(header::CONTENT_DISPOSITION)
    .assert_body("foo")
    .send();
}

#[test]
fn file_with_path_nested() {
  let server = TestServer::new();

  server.write_file(b"foo");

  server
    .get(format!("/file/{}/bar/baz.png", Hash::bytes(b"foo")))
    .assert_header(header::CONTENT_TYPE, "image/png")
    .assert_body("foo")
    .send();
}

#[test]
fn file_with_path_redirect() {
  let hash = Hash::bytes(b"foo");

  TestServer::new()
    .get(format!("/file/{hash}/bar.txt"))
    .status(StatusCode::TEMPORARY_REDIRECT)
    .assert_header(header::LOCATION, format!("/file/{hash}"))
    .send();
}

#[test]
fn files_empty() {
  TestServer::new()
    .get("/files")
    .assert_page(FilesHtml { files: Vec::new() })
    .send();
}

#[test]
fn files_non_empty() {
  let server = TestServer::new();

  server.write_file(b"foo");
  server.write_file(b"bar");
  server.write_file(b"baz");

  fs::write(server.data_dir.join("files").join("not-a-hash"), "").unwrap();

  let mut files = vec![
    Hash::bytes(b"foo"),
    Hash::bytes(b"bar"),
    Hash::bytes(b"baz"),
  ];
  files.sort();

  server.get("/files").assert_page(FilesHtml { files }).send();
}

#[test]
fn fingerprint_redirects_to_package() {
  TestServer::new()
    .get(format!("/{}", test::FINGERPRINT))
    .status(StatusCode::PERMANENT_REDIRECT)
    .assert_header(header::LOCATION, format!("/package/{}", test::FINGERPRINT))
    .send();
}

#[test]
fn gc_empty_server_removes_nothing() {
  TestServer::new()
    .post("/api/gc")
    .assert_body(
      api::gc::Response {
        bytes: 0,
        directories: BTreeSet::new().into(),
        files: BTreeSet::new().into(),
      }
      .encode_to_vec(),
    )
    .send();
}

#[test]
fn gc_ignores_non_hash_filenames() {
  let server = TestServer::new();

  fs::write(server.data_dir.join("files").join("not-a-hash"), "").unwrap();

  server
    .post("/api/gc")
    .assert_body(
      api::gc::Response {
        bytes: 0,
        directories: BTreeSet::new().into(),
        files: BTreeSet::new().into(),
      }
      .encode_to_vec(),
    )
    .send();

  assert!(
    server
      .data_dir
      .join("files")
      .join("not-a-hash")
      .try_exists()
      .unwrap()
  );
}

#[test]
fn gc_rejects_missing_auth_header() {
  let admin = PrivateKey::generate();
  let server = TestServer::builder()
    .auth_config(AuthConfig {
      admin: Some(admin.public_key()),
      audience: Some("filepack.example".into()),
    })
    .build();

  server
    .post("/api/gc")
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("missing authorization header")
    .send();
}

#[test]
fn gc_removes_unreachable_and_retains_reachable_data() {
  let server = TestServer::new();

  let retained = PackageBuilder::new().file("foo", b"foo");

  let retained_hash = retained.directory().cbor().1;

  retained.upload(&server);

  let mut subdirectory = Directory::new();
  subdirectory.insert_file("baz", b"baz");

  let (subdirectory_cbor, subdirectory_hash) = subdirectory.cbor();

  let package = PackageBuilder::new()
    .file("bar/baz", b"baz")
    .file("foo", b"foo");

  let (root_cbor, root_hash) = package.directory().cbor();

  let fingerprint = package.upload(&server);

  server.delete(format!("/api/package/{fingerprint}")).send();

  server
    .post("/api/gc")
    .assert_body(
      api::gc::Response {
        bytes: root_cbor.len().into_u64() + subdirectory_cbor.len().into_u64() + 3,
        directories: BTreeSet::from([root_hash, subdirectory_hash]).into(),
        files: BTreeSet::from([root_hash, subdirectory_hash, Hash::bytes(b"baz")]).into(),
      }
      .encode_to_vec(),
    )
    .send();

  server.assert_file(retained_hash);
  server.assert_file(Hash::bytes(b"foo"));

  assert_eq!(
    fs::read_dir(server.data_dir.join("files")).unwrap().count(),
    2,
  );
}

#[test]
fn get_directory_not_found() {
  let server = TestServer::new();

  let (cbor, hash) = Directory::new().cbor();
  server.write_file(&cbor);

  server
    .get(format!("/directory/{hash}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("directory {hash} not found"))
    .send();
}

#[test]
fn get_directory_succeeds() {
  let server = TestServer::new();

  let directory = Directory::new();
  let (cbor, hash) = directory.cbor();
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();

  server
    .get(format!("/directory/{hash}"))
    .assert_page(DirectoryHtml { directory, hash })
    .send();
}

#[test]
fn get_package_not_found() {
  let server = TestServer::new();

  let fingerprint = Fingerprint(Hash::bytes(b"foo"));

  server
    .get(format!("/package/{fingerprint}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not found"))
    .send();
}

#[test]
fn get_package_with_metadata() {
  let server = TestServer::new();

  let readme = b"foo";
  server.write_file(readme);

  let colophon = b"bar";
  server.write_file(colophon);

  let metadata = Metadata {
    artwork: None,
    creator: None,
    description: None,
    homepage: None,
    language: None,
    media: None,
    package: Some(Package {
      colophon: Some("COLOPHON.md".parse().unwrap()),
      creator: None,
      description: None,
      homepage: None,
      time: None,
      title: None,
    }),
    readme: Some("README.md".parse().unwrap()),
    thumbnails: None,
    time: None,
    title: Some("foo".parse().unwrap()),
  };
  let metadata_cbor = metadata.encode_to_vec();
  server.write_file(&metadata_cbor);

  let mut directory = Directory::new();
  directory
    .insert_file("COLOPHON.md", colophon)
    .insert_file("README.md", readme)
    .insert_file(Metadata::CBOR_FILENAME, &metadata_cbor);
  let (cbor, hash) = directory.cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/package/{fingerprint}")).send();

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      colophon: Some(Hash::bytes(colophon)),
      directory,
      fingerprint,
      metadata: Some(metadata),
      mounted: false,
      readme: Some(Hash::bytes(readme)),
      totals: Totals {
        directories: 0,
        directory_size: 0,
        file_size: metadata_cbor.len().into_u64() + 6,
        files: 3,
      },
    })
    .send();
}

#[test]
fn get_package_without_metadata() {
  let server = TestServer::new();

  let directory = Directory::new();
  let (cbor, hash) = directory.cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/package/{fingerprint}")).send();

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      colophon: None,
      directory,
      fingerprint,
      metadata: None,
      mounted: false,
      readme: None,
      totals: Totals::default(),
    })
    .send();
}

#[test]
fn home() {
  TestServer::new()
    .get("/")
    .assert_static("index.html")
    .send();
}

#[test]
fn install_script() {
  TestServer::new()
    .get("/install.sh")
    .assert_static("install.sh")
    .send();
}

#[test]
fn malformed_fingerprint_returns_error() {
  TestServer::new()
    .get("/package1invalid")
    .status(StatusCode::BAD_REQUEST)
    .assert_body("failed to decode bech32 package fingerprint")
    .send();
}

#[test]
fn media_audio_item_file_missing() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Audio {
      items: tracks(&["foo.flac"]),
    }),
    ..default()
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.flac", b"foo")
    .upload(&server);

  let metadata_cbor = metadata.encode_to_vec();

  let (cbor, _hash) = Directory::new()
    .insert_file(Metadata::CBOR_FILENAME, &metadata_cbor)
    .cbor();

  let hash = Hash::from(fingerprint);
  fs::write(server.data_dir.join("files").join(hash.to_string()), &cbor).unwrap();

  server
    .get(format!("/media/audio/{fingerprint}/item/1"))
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .assert_body(format!(
      "file `foo.flac` missing from package {fingerprint}"
    ))
    .send();
}

#[test]
fn media_audio_item_out_of_range() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Audio {
        items: tracks(&["foo.flac", "bar.flac"]),
      }),
      ..default()
    })
    .file("foo.flac", b"foo")
    .file("bar.flac", b"bar")
    .upload(&server);

  server
    .get(format!("/media/audio/{fingerprint}/item/3"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "track 3 does not exist, package {fingerprint} has 2 tracks"
    ))
    .send();

  server
    .get(format!("/media/audio/{fingerprint}/item/0"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(
      "Invalid URL: Cannot parse `item` with value `0`: number would be zero for non-zero type",
    )
    .send();
}

#[test]
fn media_audio_item_package_not_found() {
  let server = TestServer::new();

  let fingerprint = Fingerprint(Hash::bytes(b"foo"));

  server
    .get(format!("/media/audio/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not found"))
    .send();
}

#[test]
fn media_audio_item_package_without_media() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      title: Some("foo".parse().unwrap()),
      ..default()
    })
    .upload(&server);

  server
    .get(format!("/media/audio/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "package {fingerprint} does not have media metadata"
    ))
    .send();
}

#[test]
fn media_audio_item_package_without_metadata() {
  let server = TestServer::new();

  let (cbor, hash) = Directory::new().cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/package/{fingerprint}")).send();

  server
    .get(format!("/media/audio/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} does not have metadata"))
    .send();
}

#[test]
fn media_audio_item_ranges() {
  #[track_caller]
  fn case(
    server: &TestServer,
    fingerprint: Fingerprint,
    range: &'static str,
    status: StatusCode,
    content_range: &str,
    body: &[u8],
  ) {
    let request = server
      .get(format!("/media/audio/{fingerprint}/item/1"))
      .range(range)
      .status(status)
      .assert_header(header::ACCEPT_RANGES, "bytes")
      .assert_header(header::CONTENT_RANGE, content_range)
      .assert_header(header::CONTENT_LENGTH, body.len().to_string())
      .assert_body(body);

    if status == StatusCode::RANGE_NOT_SATISFIABLE {
      request.assert_header_absent(header::CACHE_CONTROL).send();
    } else {
      request
        .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .send();
    }
  }

  let server = TestServer::new();

  let audio: &[u8] = b"foobarbaz";

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Audio {
        items: tracks(&["foo.flac"]),
      }),
      ..default()
    })
    .file("foo.flac", audio)
    .upload(&server);

  case(
    &server,
    fingerprint,
    "bytes=2-5",
    StatusCode::PARTIAL_CONTENT,
    "bytes 2-5/9",
    b"obar",
  );

  case(
    &server,
    fingerprint,
    "bytes=3-",
    StatusCode::PARTIAL_CONTENT,
    "bytes 3-8/9",
    b"barbaz",
  );

  case(
    &server,
    fingerprint,
    "bytes=-3",
    StatusCode::PARTIAL_CONTENT,
    "bytes 6-8/9",
    b"baz",
  );

  case(
    &server,
    fingerprint,
    "bytes=100-200",
    StatusCode::RANGE_NOT_SATISFIABLE,
    "bytes */9",
    b"",
  );
}

#[test]
fn media_audio_item_response() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";
  let bar: &[u8] = b"barbar";

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Audio {
        items: tracks(&["foo.flac", "bar.mp3"]),
      }),
      ..default()
    })
    .file("foo.flac", foo)
    .file("bar.mp3", bar)
    .upload(&server);

  server
    .get(format!("/media/audio/{fingerprint}/item/1"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_TYPE, "audio/flac")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(foo)))
    .assert_body(foo)
    .send();

  server
    .get(format!("/media/audio/{fingerprint}/item/2"))
    .assert_header(header::CONTENT_LENGTH, "6")
    .assert_header(header::CONTENT_TYPE, "audio/mpeg")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(bar)))
    .assert_body(bar)
    .send();
}

#[test]
fn media_document_item_out_of_range() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Document {
        items: vec![Item::test("foo.pdf")],
      }),
      ..default()
    })
    .file("foo.pdf", b"foo")
    .upload(&server);

  server
    .get(format!("/media/document/{fingerprint}/item/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "document 2 does not exist, package {fingerprint} has 1 document"
    ))
    .send();
}

#[test]
fn media_document_item_response() {
  let server = TestServer::new();

  let pdf: &[u8] = b"%PDF-1.7\n";

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Document {
        items: vec![Item::test("foo.pdf")],
      }),
      ..default()
    })
    .file("foo.pdf", pdf)
    .upload(&server);

  server
    .get(format!("/media/document/{fingerprint}/item/1"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, pdf.len().to_string())
    .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
    .assert_header(header::CONTENT_TYPE, "application/pdf")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(pdf)))
    .assert_header_absent(header::CONTENT_DISPOSITION)
    .assert_body(pdf)
    .send();
}

#[test]
fn media_image_item_out_of_range() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png")],
      }),
      ..default()
    })
    .file("foo.png", foo)
    .upload(&server);

  server
    .get(format!("/media/image/{fingerprint}/item/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "image 2 does not exist, package {fingerprint} has 1 image"
    ))
    .send();
}

#[test]
fn media_image_item_response() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";
  let bar: &[u8] = b"barbar";

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png"), Item::test("bar.jpg")],
      }),
      ..default()
    })
    .file("foo.png", foo)
    .file("bar.jpg", bar)
    .upload(&server);

  server
    .get(format!("/media/image/{fingerprint}/item/1"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
    .assert_header(header::CONTENT_TYPE, "image/png")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(foo)))
    .assert_body(foo)
    .send();

  server
    .get(format!("/media/image/{fingerprint}/item/2"))
    .assert_header(header::CONTENT_LENGTH, "6")
    .assert_header(header::CONTENT_TYPE, "image/jpeg")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(bar)))
    .assert_body(bar)
    .send();
}

#[test]
fn media_image_item_thumbnail_response() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";
  let bar: &[u8] = b"barbar";
  let thumbnail: &[u8] = b"bazbazbaz";

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png"), Item::test("bar.jpg")],
      }),
      thumbnails: Some(
        [(
          "foo.png".parse().unwrap(),
          Image::test("thumbnails/foo.jpg"),
        )]
        .into(),
      ),
      ..default()
    })
    .file("foo.png", foo)
    .file("bar.jpg", bar)
    .file("thumbnails/foo.jpg", thumbnail)
    .upload(&server);

  server
    .get(format!("/media/image/{fingerprint}/item/1/thumbnail"))
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "9")
    .assert_header(header::CONTENT_TYPE, "image/jpeg")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(thumbnail)))
    .assert_body(thumbnail)
    .send();

  server
    .get(format!("/media/image/{fingerprint}/item/2/thumbnail"))
    .assert_header(header::CONTENT_LENGTH, "6")
    .assert_header(header::CONTENT_TYPE, "image/jpeg")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(bar)))
    .assert_body(bar)
    .send();

  server
    .get(format!("/media/image/{fingerprint}/item/3/thumbnail"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "image 3 does not exist, package {fingerprint} has 2 images"
    ))
    .send();
}

#[test]
fn media_type_mismatch() {
  #[track_caller]
  fn case(server: &TestServer, path: String, body: String) {
    server
      .get(path)
      .status(StatusCode::NOT_FOUND)
      .assert_body(body)
      .send();
  }

  let server = TestServer::new();

  let foo: &[u8] = b"foo";

  let audio = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Audio {
        items: tracks(&["foo.flac"]),
      }),
      ..default()
    })
    .file("foo.flac", foo)
    .upload(&server);

  let image = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Image {
        items: vec![Item::test("foo.png")],
      }),
      ..default()
    })
    .file("foo.png", foo)
    .upload(&server);

  case(
    &server,
    format!("/media/image/{audio}/item/1"),
    format!("expected media type image but package {audio} is audio"),
  );

  case(
    &server,
    format!("/media/image/{audio}/item/1/thumbnail"),
    format!("expected media type image but package {audio} is audio"),
  );

  case(
    &server,
    format!("/media/audio/{image}/item/1"),
    format!("expected media type audio but package {image} is image"),
  );
}

#[test]
fn media_video_item_out_of_range() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Video {
        items: vec![Item::test("foo.mp4")],
      }),
      ..default()
    })
    .file("foo.mp4", b"foo")
    .upload(&server);

  server
    .get(format!("/media/video/{fingerprint}/item/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "video 2 does not exist, package {fingerprint} has 1 video"
    ))
    .send();
}

#[test]
fn media_video_item_placeholder_response() {
  let server = TestServer::new();

  let mut video = Item::<Video>::test("foo.mp4");
  video.content.placeholder = Some(Image::test("bar.png"));

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Video {
        items: vec![video, Item::test("baz.mp4")],
      }),
      ..default()
    })
    .file("foo.mp4", b"foo")
    .file("bar.png", b"bar")
    .file("baz.mp4", b"baz")
    .upload(&server);

  server
    .get(format!("/media/video/{fingerprint}/item/1/placeholder"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_TYPE, "image/png")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(b"bar")))
    .assert_body(b"bar")
    .send();

  server
    .get(format!("/media/video/{fingerprint}/item/2/placeholder"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "video 2 in package {fingerprint} does not have a placeholder image"
    ))
    .send();

  server
    .get(format!("/media/video/{fingerprint}/item/3/placeholder"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "video 3 does not exist, package {fingerprint} has 2 videos"
    ))
    .send();
}

#[test]
fn media_video_item_response() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Video {
        items: vec![Item::test("foo.mp4"), Item::test("bar.mp4")],
      }),
      ..default()
    })
    .file("foo.mp4", b"foo")
    .file("bar.mp4", b"bar")
    .upload(&server);

  server
    .get(format!("/media/video/{fingerprint}/item/1"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_TYPE, "video/mp4")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(b"foo")))
    .assert_body(b"foo")
    .send();

  server
    .get(format!("/media/video/{fingerprint}/item/2"))
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_TYPE, "video/mp4")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(b"bar")))
    .assert_body(b"bar")
    .send();
}

#[test]
fn media_webm_item_response() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Video {
        items: vec![Item::test("foo.webm")],
      }),
      ..default()
    })
    .file("foo.webm", b"foo")
    .upload(&server);

  server
    .get(format!("/media/video/{fingerprint}/item/1"))
    .assert_header(header::CONTENT_TYPE, "video/webm")
    .assert_body(b"foo")
    .send();
}

#[test]
fn missing_rejects_unsorted_hashes() {
  let mut hashes = BTreeSet::from([Hash::bytes(b"foo"), Hash::bytes(b"bar")])
    .into_iter()
    .collect::<Vec<_>>();

  hashes.reverse();

  let mut encoder = Encoder::new();
  let mut map = encoder.map::<u64>(1);
  map.item(0, hashes);
  drop(map);

  TestServer::new()
    .post("/api/missing")
    .body(encoder.finish())
    .status(StatusCode::BAD_REQUEST)
    .assert_body("failed to decode request body")
    .send();
}

#[test]
fn missing_returns_missing_hashes() {
  let server = TestServer::new();

  let present = Hash::bytes(b"bar");
  let absent = Hash::bytes(b"baz");

  server.write_file(b"bar");

  server
    .post("/api/missing")
    .body(
      api::missing::Request {
        hashes: BTreeSet::from([present, absent]).into(),
      }
      .encode_to_vec(),
    )
    .assert_body(
      api::missing::Response {
        hashes: BTreeSet::from([absent]).into(),
      }
      .encode_to_vec(),
    )
    .send();
}

#[test]
fn mount_file() {
  let metadata = Metadata {
    media: Some(Media::Web),
    ..default()
  };

  let package = PackageBuilder::new()
    .metadata(&metadata)
    .file("static/foo.css", b"bar")
    .file("static/index.html", b"foo");

  let server = TestServer::builder().mount(package.fingerprint()).build();

  let fingerprint = package.upload(&server);

  server
    .get(format!("/mount/{fingerprint}/foo.css"))
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_TYPE, "text/css")
    .assert_header_absent(header::CONTENT_SECURITY_POLICY)
    .assert_header_absent(header::CONTENT_DISPOSITION)
    .assert_body("bar")
    .send();
}

#[test]
fn mount_file_invalid_path() {
  let metadata = Metadata {
    media: Some(Media::Web),
    ..default()
  };

  let package = PackageBuilder::new()
    .metadata(&metadata)
    .file("static/index.html", b"foo");

  let server = TestServer::builder().mount(package.fingerprint()).build();

  let fingerprint = package.upload(&server);

  server
    .get(format!("/mount/{fingerprint}/%2e%2e/foo"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(
      "Invalid URL: Cannot parse `path` with value `../foo`: path contains invalid component `..`",
    )
    .send();
}

#[test]
fn mount_file_nested() {
  let metadata = Metadata {
    media: Some(Media::Web),
    ..default()
  };

  let package = PackageBuilder::new()
    .metadata(&metadata)
    .file("static/index.html", b"foo")
    .file("static/foo/bar.txt", b"baz");

  let server = TestServer::builder().mount(package.fingerprint()).build();

  let fingerprint = package.upload(&server);

  server
    .get(format!("/mount/{fingerprint}/foo/bar.txt"))
    .assert_header(header::CONTENT_TYPE, "text/plain")
    .assert_body("baz")
    .send();
}

#[test]
fn mount_file_not_found() {
  let metadata = Metadata {
    media: Some(Media::Web),
    ..default()
  };

  let package = PackageBuilder::new()
    .metadata(&metadata)
    .file("static/index.html", b"foo");

  let server = TestServer::builder().mount(package.fingerprint()).build();

  let fingerprint = package.upload(&server);

  server
    .get(format!("/mount/{fingerprint}/foo"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "file `static/foo` not found in package {fingerprint}"
    ))
    .send();
}

#[test]
fn mount_file_not_mounted() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Web),
      ..default()
    })
    .file("static/index.html", b"foo")
    .upload(&server);

  server
    .get(format!("/mount/{fingerprint}/index.html"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not mounted"))
    .send();
}

#[test]
fn mount_redirect() {
  TestServer::new()
    .get(format!("/mount/{}", test::FINGERPRINT))
    .status(StatusCode::PERMANENT_REDIRECT)
    .assert_header(header::LOCATION, format!("/mount/{}/", test::FINGERPRINT))
    .send();
}

#[test]
fn mount_serves_index_html() {
  let metadata = Metadata {
    media: Some(Media::Web),
    ..default()
  };

  let package = PackageBuilder::new()
    .metadata(&metadata)
    .file("static/index.html", b"foo");

  let server = TestServer::builder().mount(package.fingerprint()).build();

  let fingerprint = package.upload(&server);

  server
    .get(format!("/mount/{fingerprint}/"))
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_TYPE, "text/html")
    .assert_header_absent(header::CONTENT_SECURITY_POLICY)
    .assert_header_absent(header::CONTENT_DISPOSITION)
    .assert_body("foo")
    .send();
}

#[test]
fn non_fingerprint_bech32_falls_through() {
  TestServer::new()
    .get(format!("/{}", test::PUBLIC_KEY))
    .assert_static("404.html")
    .status(StatusCode::NOT_FOUND)
    .send();
}

#[test]
fn package_item_audio() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Audio {
      items: tracks(&["foo.flac"]),
    }),
    ..default()
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.flac", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .assert_page(AudioHtml {
      audio: 0,
      fingerprint,
      metadata,
    })
    .send();
}

#[test]
fn package_item_audio_out_of_range() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Audio {
        items: tracks(&["foo.flac"]),
      }),
      ..default()
    })
    .file("foo.flac", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "track 2 does not exist, package {fingerprint} has 1 track"
    ))
    .send();
}

#[test]
fn package_item_document() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Document {
      items: vec![Item::test("foo.pdf")],
    }),
    ..default()
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.pdf", b"%PDF-1.7\n")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .assert_page(DocumentHtml {
      document: 0,
      fingerprint,
      metadata,
    })
    .send();
}

#[test]
fn package_item_image() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Image {
      items: vec![Item {
        content: Image {
          alpha: false,
          bit_depth: 8,
          chroma_subsampling: None,
          color_type: ColorType::Rgb,
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          orientation: Orientation::new(),
          path: "foo.png".parse().unwrap(),
          ty: ImageType::Png,
        },
        title: None,
      }],
    }),
    ..default()
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.png", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .assert_page(ImageHtml {
      fingerprint,
      image: 0,
      metadata,
    })
    .send();
}

#[test]
fn package_item_image_out_of_range() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Image {
        items: vec![Item {
          content: Image {
            alpha: false,
            bit_depth: 8,
            chroma_subsampling: None,
            color_type: ColorType::Rgb,
            dimensions: Dimensions {
              height: 1,
              width: 1,
            },
            orientation: Orientation::new(),
            path: "foo.png".parse().unwrap(),
            ty: ImageType::Png,
          },
          title: None,
        }],
      }),
      ..default()
    })
    .file("foo.png", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "image 2 does not exist, package {fingerprint} has 1 image"
    ))
    .send();
}

#[test]
fn package_item_package_not_found() {
  let server = TestServer::new();

  let fingerprint = Fingerprint(Hash::bytes(b"foo"));

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not found"))
    .send();
}

#[test]
fn package_item_video() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Video {
      items: vec![Item {
        content: Video {
          placeholder: None,
          duration: 0,
          path: "foo.mp4".parse().unwrap(),
          tracks: vec![
            Track {
              codec: Codec::H264,
              info: TrackInfo::Video {
                bit_depth: 8,
                chroma_subsampling: ChromaSubsampling::Yuv420,
                dimensions: Dimensions {
                  height: 1,
                  width: 2,
                },
                frames: 0,
                orientation: Orientation::new(),
              },
              size: 0,
            },
            Track {
              codec: Codec::Aac,
              info: TrackInfo::Audio {
                channels: 2,
                sample_rate: 44100,
              },
              size: 0,
            },
          ],
          ty: VideoType::Mp4,
        },
        title: None,
      }],
    }),
    ..default()
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.mp4", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .assert_page(VideoHtml {
      fingerprint,
      metadata,
      video: 0,
    })
    .send();
}

#[test]
fn package_item_video_out_of_range() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Video {
        items: vec![Item::test("foo.mp4")],
      }),
      ..default()
    })
    .file("foo.mp4", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "video 2 does not exist, package {fingerprint} has 1 video"
    ))
    .send();
}

#[test]
fn package_item_web() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      media: Some(Media::Web),
      ..default()
    })
    .file("static/index.html", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body("media type web does not have items")
    .send();
}

#[test]
fn package_item_without_media() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      title: Some("foo".parse().unwrap()),
      ..default()
    })
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "package {fingerprint} does not have media metadata"
    ))
    .send();
}

#[test]
fn package_item_without_metadata() {
  let server = TestServer::new();

  let (cbor, hash) = Directory::new().cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/package/{fingerprint}")).send();

  server
    .get(format!("/package/{fingerprint}/item/1"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} does not have metadata"))
    .send();
}

#[test]
fn package_media() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Audio {
      items: tracks(&["foo.flac"]),
    }),
    ..default()
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.flac", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/media"))
    .assert_page(MediaHtml {
      fingerprint,
      metadata,
    })
    .send();
}

#[test]
fn package_media_without_media() {
  let server = TestServer::new();

  let fingerprint = PackageBuilder::new()
    .metadata(&Metadata {
      title: Some("foo".parse().unwrap()),
      ..default()
    })
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}/media"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "package {fingerprint} does not have media metadata"
    ))
    .send();
}

#[test]
fn package_page_og_image() {
  let url = "https://example.com".parse::<Url>().unwrap();

  let server = TestServer::builder().url(url.clone()).build();

  let artwork = b"foo";
  server.write_file(artwork);

  let metadata = Metadata {
    artwork: Some(Image::test("bar.png")),
    ..Metadata::default()
  };
  let metadata_cbor = metadata.encode_to_vec();
  server.write_file(&metadata_cbor);

  let mut directory = Directory::new();
  directory
    .insert_file("bar.png", artwork)
    .insert_file(Metadata::CBOR_FILENAME, &metadata_cbor);
  let (cbor, hash) = directory.cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/package/{fingerprint}")).send();

  server
    .get(format!("/package/{fingerprint}"))
    .assert_response(
      PackageHtml {
        colophon: None,
        directory,
        fingerprint,
        metadata: Some(metadata),
        mounted: false,
        readme: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: metadata_cbor.len().into_u64() + 3,
          files: 2,
        },
      }
      .page(Some(url)),
    )
    .send();
}

#[test]
fn package_page_renders_audio_media() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Audio {
      items: vec![
        Item {
          content: Audio {
            album: "qux".parse().unwrap(),
            artist: "baz".parse().unwrap(),
            channels: 2,
            disc: 1,
            discs: 1,
            path: "foo.flac".parse().unwrap(),
            sample_bits: Some(16),
            sample_rate: 44100,
            samples: 9_922_500,
            size: 0,
            track: 1,
            tracks: 2,
            ty: AudioType::Flac,
          },
          title: Some("foo".parse().unwrap()),
        },
        Item {
          content: Audio {
            album: "qux".parse().unwrap(),
            artist: "baz".parse().unwrap(),
            channels: 2,
            disc: 1,
            discs: 1,
            path: "bar.flac".parse().unwrap(),
            sample_bits: Some(16),
            sample_rate: 44100,
            samples: 44100,
            size: 0,
            track: 2,
            tracks: 2,
            ty: AudioType::Flac,
          },
          title: Some("bar".parse().unwrap()),
        },
      ],
    }),
    ..default()
  };

  let totals = Totals {
    directories: 0,
    directory_size: 0,
    file_size: metadata.encode_to_vec().len().into_u64() + 6,
    files: 3,
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.flac", b"foo")
    .file("bar.flac", b"bar")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint,
      metadata: Some(metadata),
      mounted: false,
      readme: None,
      totals,
    })
    .send();
}

#[test]
fn package_page_renders_image_media() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Image {
      items: vec![Item {
        content: Image {
          alpha: false,
          bit_depth: 8,
          chroma_subsampling: None,
          color_type: ColorType::Rgb,
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          orientation: Orientation::new(),
          path: "foo.png".parse().unwrap(),
          ty: ImageType::Png,
        },
        title: None,
      }],
    }),
    ..default()
  };

  let totals = Totals {
    directories: 0,
    directory_size: 0,
    file_size: metadata.encode_to_vec().len().into_u64() + 3,
    files: 2,
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.png", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint,
      metadata: Some(metadata),
      mounted: false,
      readme: None,
      totals,
    })
    .send();
}

#[test]
fn package_page_renders_video_media() {
  let server = TestServer::new();

  let metadata = Metadata {
    media: Some(Media::Video {
      items: vec![Item {
        content: Video {
          placeholder: None,
          duration: 0,
          path: "foo.mp4".parse().unwrap(),
          tracks: vec![
            Track {
              codec: Codec::H264,
              info: TrackInfo::Video {
                bit_depth: 8,
                chroma_subsampling: ChromaSubsampling::Yuv420,
                dimensions: Dimensions {
                  height: 1,
                  width: 2,
                },
                frames: 0,
                orientation: Orientation::new(),
              },
              size: 0,
            },
            Track {
              codec: Codec::Aac,
              info: TrackInfo::Audio {
                channels: 2,
                sample_rate: 44100,
              },
              size: 0,
            },
          ],
          ty: VideoType::Mp4,
        },
        title: None,
      }],
    }),
    ..default()
  };

  let totals = Totals {
    directories: 0,
    directory_size: 0,
    file_size: metadata.encode_to_vec().len().into_u64() + 3,
    files: 2,
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.mp4", b"foo")
    .upload(&server);

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint,
      metadata: Some(metadata),
      mounted: false,
      readme: None,
      totals,
    })
    .send();
}

#[test]
fn package_page_web() {
  let metadata = Metadata {
    media: Some(Media::Web),
    ..default()
  };

  let metadata_cbor_len = metadata.encode_to_vec().len().into_u64();

  let package = PackageBuilder::new()
    .metadata(&metadata)
    .file("static/index.html", b"foo");

  let directory = package.directory();

  let mut static_directory = Directory::new();
  static_directory.insert_file("index.html", b"foo");
  let static_cbor_len = static_directory.cbor().0.len().into_u64();

  let server = TestServer::builder().mount(package.fingerprint()).build();

  let fingerprint = package.upload(&server);

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      colophon: None,
      directory,
      fingerprint,
      metadata: Some(metadata),
      mounted: true,
      readme: None,
      totals: Totals {
        directories: 1,
        directory_size: static_cbor_len,
        file_size: metadata_cbor_len + 3,
        files: 2,
      },
    })
    .send();
}

#[test]
fn packages_empty() {
  TestServer::new()
    .get("/packages")
    .assert_page(PackagesHtml {
      packages: Vec::new(),
      view: View::List,
    })
    .send();
}

#[test]
fn packages_grid() {
  let server = TestServer::new();

  let metadata = Metadata {
    artwork: Some(Image::test("foo.png")),
    ..default()
  };

  let totals = Totals {
    directories: 0,
    directory_size: 0,
    file_size: metadata.encode_to_vec().len().into_u64() + 3,
    files: 2,
  };

  let fingerprint = PackageBuilder::new()
    .metadata(&metadata)
    .file("foo.png", b"bar")
    .upload(&server);

  server
    .get("/packages?view=grid")
    .assert_page(PackagesHtml {
      packages: vec![(fingerprint, Some(metadata), totals)],
      view: View::Grid,
    })
    .send();
}

#[test]
fn packages_include_creators_and_titles() {
  let server = TestServer::new();

  let metadata = Metadata {
    creator: Some("foo".parse().unwrap()),
    title: Some("bar".parse().unwrap()),
    ..default()
  };

  let totals = Totals {
    directories: 0,
    directory_size: 0,
    file_size: metadata.encode_to_vec().len().into_u64(),
    files: 1,
  };

  let fingerprint = PackageBuilder::new().metadata(&metadata).upload(&server);

  server
    .get("/packages")
    .assert_page(PackagesHtml {
      packages: vec![(fingerprint, Some(metadata), totals)],
      view: View::List,
    })
    .send();
}

#[test]
fn packages_invalid_view() {
  TestServer::new()
    .get("/packages?view=foo")
    .status(StatusCode::BAD_REQUEST)
    .assert_body(
      "Failed to deserialize query string: view: unknown variant `foo`, expected `grid` or `list`",
    )
    .send();
}

#[test]
fn packages_non_empty() {
  let server = TestServer::new();

  let mut packages = Vec::new();

  for content in [b"foo".as_slice(), b"bar", b"baz"] {
    server.write_file(content);
    let (cbor, hash) = Directory::new().insert_file("file", content).cbor();
    let fingerprint = Fingerprint(hash);
    server.write_file(&cbor);
    server.post(format!("/api/directory/{hash}")).send();
    server.post(format!("/api/package/{fingerprint}")).send();
    packages.push((
      fingerprint,
      None,
      Totals {
        directories: 0,
        directory_size: 0,
        file_size: 3,
        files: 1,
      },
    ));
  }

  packages.sort_by_key(|&(fingerprint, ..)| fingerprint);

  server
    .get("/packages")
    .assert_page(PackagesHtml {
      packages,
      view: View::List,
    })
    .send();
}

#[test]
fn packages_sorted_by_title_then_fingerprint() {
  let server = TestServer::new();

  let mut packages = Vec::new();

  for title in [Some("Baz"), None, Some("bar")] {
    let metadata = Metadata {
      title: title.map(|title| title.parse().unwrap()),
      ..default()
    };

    let totals = Totals {
      directories: 0,
      directory_size: 0,
      file_size: metadata.encode_to_vec().len().into_u64(),
      files: 1,
    };

    let fingerprint = PackageBuilder::new().metadata(&metadata).upload(&server);

    packages.push((fingerprint, Some(metadata), totals));
  }

  let (baz, untitled, bar) = (
    packages[0].clone(),
    packages[1].clone(),
    packages[2].clone(),
  );

  assert!(baz.0 < bar.0);
  assert!(untitled.0 < bar.0);

  server
    .get("/packages")
    .assert_page(PackagesHtml {
      packages: vec![bar, baz, untitled],
      view: View::List,
    })
    .send();
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
fn redirect_omits_default_ports() {
  assert_eq!(
    Serve {
      domain: Some("foo".into()),
      ..Serve::default()
    }
    .redirect_url()
    .as_str(),
    "http://foo/",
  );

  assert_eq!(
    Serve {
      domain: Some("foo".into()),
      https: true,
      ..Serve::default()
    }
    .redirect_url()
    .as_str(),
    "https://foo/",
  );
}

#[test]
fn restricted_write_accepts_admin_token() {
  let admin = PrivateKey::generate();
  let hash = Hash::bytes(b"bar");
  let token = Token::encode(&admin, "filepack.example").unwrap();

  let server = TestServer::builder()
    .auth_config(AuthConfig {
      admin: Some(admin.public_key()),
      audience: Some("filepack.example".into()),
    })
    .build();

  server
    .put(format!("/file/{hash}"))
    .body("bar")
    .token(token)
    .send();

  server.assert_file(hash);
}

#[test]
fn restricted_write_rejects_missing_header() {
  let admin = PrivateKey::generate();
  let server = TestServer::builder()
    .auth_config(AuthConfig {
      admin: Some(admin.public_key()),
      audience: Some("filepack.example".into()),
    })
    .build();

  let hash = Hash::bytes(b"bar");

  server
    .put(format!("/file/{hash}"))
    .body("bar")
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("missing authorization header")
    .send();
}

#[test]
fn restricted_write_rejects_others() {
  let admin = PrivateKey::generate();
  let other = PrivateKey::generate();
  let server = TestServer::builder()
    .auth_config(AuthConfig {
      admin: Some(admin.public_key()),
      audience: Some("filepack.example".into()),
    })
    .build();

  let hash = Hash::bytes(b"bar");
  let token = Token::encode(&other, "filepack.example").unwrap();

  server
    .put(format!("/file/{hash}"))
    .body("bar")
    .token(token)
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("invalid authorization token")
    .send();
}

#[test]
fn server_config() {
  #[track_caller]
  fn case(serve: Serve, url: Option<&str>) {
    assert_eq!(
      serve.server_config().url,
      url.map(|url| url.parse().unwrap()),
    );
  }

  case(Serve::default(), None);
  case(
    Serve {
      domain: Some("foo".into()),
      ..Serve::default()
    },
    Some("http://foo/"),
  );
  case(
    Serve {
      domain: Some("foo".into()),
      https: true,
      ..Serve::default()
    },
    Some("https://foo/"),
  );

  let fingerprint = Fingerprint(Hash::bytes(b"foo"));

  assert_eq!(
    Serve {
      mounts: vec![fingerprint],
      ..Serve::default()
    }
    .server_config()
    .mounts,
    HashSet::from([fingerprint]),
  );
}

#[test]
fn static_files() {
  TestServer::new()
    .get("/static/index.css")
    .assert_static("index.css")
    .send();
}

fn tracks(filenames: &[&str]) -> Vec<Item<Audio>> {
  filenames
    .iter()
    .enumerate()
    .map(|(i, filename)| {
      let mut audio = Item::<Audio>::test(filename);
      audio.content.disc = 1;
      audio.content.discs = 1;
      audio.content.track = i.into_u64() + 1;
      audio.content.tracks = filenames.len().into_u64();
      audio
    })
    .collect()
}

#[test]
fn upload_creates_file() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"bar");

  server.put(format!("/file/{hash}")).body("bar").send();

  server.assert_file(hash);

  server.assert_incoming_empty();
}

#[test]
fn upload_short_circuits_when_file_exists() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"bar");

  server.write_file(b"bar");

  server.put(format!("/file/{hash}")).body("bar").send();

  server.assert_file(hash);

  server.assert_incoming_empty();
}

#[test]
fn upload_with_wrong_hash_fails() {
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
    .send();

  server.assert_incoming_empty();
}

#[test]
fn uppercase_fingerprint_redirects_to_lowercase_package() {
  TestServer::new()
    .get(format!("/{}", test::FINGERPRINT.to_uppercase()))
    .status(StatusCode::PERMANENT_REDIRECT)
    .assert_header(header::LOCATION, format!("/package/{}", test::FINGERPRINT))
    .send();
}

#[test]
fn verify_directory_decode_error() {
  let server = TestServer::new();

  let junk = b"junk";
  let hash = Hash::bytes(junk);
  server.write_file(junk);

  server
    .post(format!("/api/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!("failed to decode directory {hash}"))
    .send();
}

#[test]
fn verify_directory_entry_size_mismatch() {
  let server = TestServer::new();

  let contents = b"bar";
  server.write_file(contents);

  let (cbor, hash) = Directory::new()
    .insert_entry("foo", Entry::file(Hash::bytes(contents), 4))
    .cbor();
  server.write_file(&cbor);

  server
    .post(format!("/api/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {hash} entry `foo` size mismatch, expected 4 but found 3"
    ))
    .send();
}

#[test]
fn verify_directory_file_not_found() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"foo");

  server
    .post(format!("/api/directory/{hash}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("file with hash {hash} not found"))
    .send();
}

#[test]
fn verify_directory_idempotent() {
  let server = TestServer::new();

  let directory = Directory::new();
  let (cbor, hash) = directory.cbor();
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();
  server.post(format!("/api/directory/{hash}")).send();

  server
    .get(format!("/directory/{hash}"))
    .assert_page(DirectoryHtml { directory, hash })
    .send();
}

#[test]
fn verify_directory_missing_file() {
  let server = TestServer::new();

  let missing = b"foo";

  let (cbor, hash) = Directory::new().insert_file("foo", missing).cbor();
  server.write_file(&cbor);

  server
    .post(format!("/api/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {hash} references missing file entry `foo` with hash {}",
      Hash::bytes(missing),
    ))
    .send();
}

#[test]
fn verify_directory_missing_subdirectory() {
  let server = TestServer::new();

  let child = Directory::new();
  let (_, child_hash) = child.cbor();

  let (parent_cbor, parent_hash) = Directory::new().insert_directory("child", &child).cbor();
  server.write_file(&parent_cbor);

  server
    .post(format!("/api/directory/{parent_hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {parent_hash} references missing directory entry `child` with hash {child_hash}"
    ))
    .send();
}

#[test]
fn verify_directory_rejects_missing_auth_header() {
  let admin = PrivateKey::generate();
  let server = TestServer::builder()
    .auth_config(AuthConfig {
      admin: Some(admin.public_key()),
      audience: Some("filepack.example".into()),
    })
    .build();

  let hash = Hash::bytes(b"foo");

  server
    .post(format!("/api/directory/{hash}"))
    .status(StatusCode::UNAUTHORIZED)
    .assert_body("missing authorization header")
    .send();
}

#[test]
fn verify_directory_subdirectory_totals_mismatch() {
  let server = TestServer::new();

  let child = Directory::new();
  let (child_cbor, child_hash) = child.cbor();
  server.write_file(&child_cbor);

  server.post(format!("/api/directory/{child_hash}")).send();

  let (parent_cbor, parent_hash) = Directory::new()
    .insert_entry(
      "child",
      Entry::directory(
        child_hash,
        child_cbor.len().into_u64(),
        Totals {
          directories: 0,
          directory_size: 0,
          file_size: 0,
          files: 1,
        },
      ),
    )
    .cbor();
  server.write_file(&parent_cbor);

  server
    .post(format!("/api/directory/{parent_hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {parent_hash} entry `child` totals error: totals mismatch, found 0 bytes in 0 \
      files and 0 bytes in 0 directories but expected 0 bytes in 1 file and 0 bytes in 0 \
      directories"
    ))
    .send();
}

#[test]
fn verify_directory_succeeds() {
  let server = TestServer::new();

  let file = b"foo";
  server.write_file(file);

  let mut child = Directory::new();
  let (child_cbor, child_hash) = child.insert_file("foo", file).cbor();

  server.write_file(&child_cbor);

  server.post(format!("/api/directory/{child_hash}")).send();

  server
    .get(format!("/directory/{child_hash}"))
    .assert_page(DirectoryHtml {
      directory: child.clone(),
      hash: child_hash,
    })
    .send();

  let mut parent = Directory::new();
  parent.insert_directory("child", &child);

  let (parent_cbor, parent_hash) = parent.cbor();
  server.write_file(&parent_cbor);

  server.post(format!("/api/directory/{parent_hash}")).send();

  server
    .get(format!("/directory/{parent_hash}"))
    .assert_page(DirectoryHtml {
      directory: parent,
      hash: parent_hash,
    })
    .send();
}

#[test]
fn verify_directory_totals_overflow() {
  let server = TestServer::new();

  let file = Hash::bytes(b"foo");

  let mut directory = Directory::new();
  directory
    .insert_entry("bar", Entry::file(file, u64::MAX))
    .insert_entry("baz", Entry::file(file, 1));

  let (cbor, hash) = directory.cbor();
  server.write_file(&cbor);

  server
    .post(format!("/api/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!("directory {hash} totals error"))
    .send();
}

#[test]
fn verify_directory_unverified_subdirectory() {
  let server = TestServer::new();

  let child = Directory::new();
  let (child_cbor, child_hash) = child.cbor();
  server.write_file(&child_cbor);

  let (parent_cbor, parent_hash) = Directory::new().insert_directory("child", &child).cbor();
  server.write_file(&parent_cbor);

  server
    .post(format!("/api/directory/{parent_hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "directory {parent_hash} references unverified subdirectory {child_hash}"
    ))
    .send();
}

#[test]
fn verify_package_metadata_decode_error() {
  let server = TestServer::new();

  let junk = b"foo";
  server.write_file(junk);

  let (cbor, hash) = Directory::new()
    .insert_file(Metadata::CBOR_FILENAME, junk)
    .cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();

  server
    .post(format!("/api/package/{fingerprint}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "failed to decode metadata for package {fingerprint}"
    ))
    .send();
}

#[test]
fn verify_package_metadata_references_missing_file() {
  let server = TestServer::new();

  let metadata = Metadata {
    artwork: Some(Image::test("cover.png")),
    ..default()
  }
  .encode_to_vec();
  server.write_file(&metadata);

  let (cbor, hash) = Directory::new()
    .insert_file(Metadata::CBOR_FILENAME, &metadata)
    .cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();

  server
    .post(format!("/api/package/{fingerprint}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "package {fingerprint} metadata references missing file `cover.png`"
    ))
    .send();
}

#[test]
fn verify_package_metadata_references_present_file() {
  let server = TestServer::new();

  let artwork = b"artwork";
  server.write_file(artwork);

  let metadata = Metadata {
    artwork: Some(Image::test("cover.png")),
    ..default()
  }
  .encode_to_vec();
  server.write_file(&metadata);

  let (cbor, hash) = Directory::new()
    .insert_file("cover.png", artwork)
    .insert_file(Metadata::CBOR_FILENAME, &metadata)
    .cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/api/directory/{hash}")).send();

  server.post(format!("/api/package/{fingerprint}")).send();
}

#[test]
fn verify_package_unverified() {
  let server = TestServer::new();

  let (cbor, hash) = Directory::new().cbor();
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server
    .post(format!("/api/package/{fingerprint}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "package {fingerprint} root directory is unverified"
    ))
    .send();
}
