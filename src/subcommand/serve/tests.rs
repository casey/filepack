use {
  super::*,
  axum::{
    body,
    http::{Method, Request, header::HeaderName},
  },
  templates::Page,
  tokio::runtime::Runtime,
  tower::ServiceExt,
};

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

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
    self.assert_response(PageHtml::from(page))
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

#[test]
fn artwork_missing() {
  let server = TestServer::new();

  let artwork = b"foo";
  let artwork_hash = Hash::bytes(artwork);
  server.write_file(artwork);

  let metadata = Metadata {
    artwork: Some("cover.png".parse().unwrap()),
    ..Metadata::default()
  };
  let metadata_cbor = metadata.encode_to_vec();
  let metadata_hash = Hash::bytes(&metadata_cbor);
  server.write_file(&metadata_cbor);

  let metadata_entry = (
    Metadata::CBOR_FILENAME,
    EntryType::File,
    metadata_hash,
    metadata_cbor.len().into_u64(),
  );

  let cbor = directory(&[
    (
      "cover.png",
      EntryType::File,
      artwork_hash,
      artwork.len().into_u64(),
    ),
    metadata_entry,
  ])
  .encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/package/{fingerprint}")).send();

  let corrupt = directory(&[metadata_entry]).encode_to_vec();
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

  let cbor = directory(&[]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/package/{fingerprint}")).send();

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
      artwork: Some(filename.parse().unwrap()),
      ..Metadata::default()
    };
    let metadata_cbor = metadata.encode_to_vec();
    let metadata_hash = Hash::bytes(&metadata_cbor);
    server.write_file(&metadata_cbor);

    let cbor = directory(&[
      (
        filename,
        EntryType::File,
        artwork_hash,
        artwork.len().into_u64(),
      ),
      (
        Metadata::CBOR_FILENAME,
        EntryType::File,
        metadata_hash,
        metadata_cbor.len().into_u64(),
      ),
    ])
    .encode_to_vec();
    let hash = Hash::bytes(&cbor);
    let fingerprint = Fingerprint(hash);
    server.write_file(&cbor);

    server.post(format!("/directory/{hash}")).send();
    server.post(format!("/package/{fingerprint}")).send();

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
fn closed_server_forbids_uploads() {
  TestServer::with_auth(Some(Arc::new(AuthConfig {
    admin: None,
    audiences: Vec::new(),
  })))
  .put(format!("/file/{}", Hash::bytes(b"bar")))
  .body("bar")
  .status(StatusCode::FORBIDDEN)
  .assert_body("uploads forbidden")
  .send();
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
fn files_empty() {
  TestServer::new()
    .get("/files")
    .assert_page(FilesHtml { files: Vec::new() })
    .send();
}

#[test]
fn files_non_empty() {
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

  server.get("/files").assert_page(FilesHtml { files }).send();
}

#[test]
fn get_directory_not_found() {
  let server = TestServer::new();

  let cbor = directory(&[]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
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

  let dir = directory(&[]);
  let cbor = dir.encode_to_vec();
  let hash = Hash::bytes(&cbor);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();

  server
    .get(format!("/directory/{hash}"))
    .assert_page(DirectoryHtml {
      directory: dir,
      hash,
    })
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

  let metadata = Metadata {
    artwork: None,
    creator: None,
    date: None,
    description: None,
    homepage: None,
    language: None,
    media: None,
    package: None,
    readme: None,
    title: Some("foo".parse().unwrap()),
  };
  let metadata_cbor = metadata.encode_to_vec();
  let metadata_hash = Hash::bytes(&metadata_cbor);
  server.write_file(&metadata_cbor);

  let cbor = directory(&[(
    Metadata::CBOR_FILENAME,
    EntryType::File,
    metadata_hash,
    metadata_cbor.len().into_u64(),
  )])
  .encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/package/{fingerprint}")).send();

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      fingerprint,
      metadata: Some(metadata),
    })
    .send();
}

#[test]
fn get_package_without_metadata() {
  let server = TestServer::new();

  let cbor = directory(&[]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/package/{fingerprint}")).send();

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      fingerprint,
      metadata: None,
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
fn media_audio_track_file_missing() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";

  let metadata = Metadata {
    media: Some(Media::Audio {
      tracks: vec!["foo.flac".parse().unwrap()],
    }),
    ..default()
  };

  let fingerprint = package(&server, &metadata, &[("foo.flac", foo)]);

  let metadata_cbor = metadata.encode_to_vec();
  let corrupt = directory(&[(
    Metadata::CBOR_FILENAME,
    EntryType::File,
    Hash::bytes(&metadata_cbor),
    metadata_cbor.len().into_u64(),
  )])
  .encode_to_vec();

  let hash = Hash::from(fingerprint);
  fs::write(
    server.data_dir.join("files").join(hash.to_string()),
    &corrupt,
  )
  .unwrap();

  server
    .get(format!("/media/audio/{fingerprint}/track/0"))
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .assert_body(format!(
      "file `foo.flac` missing from package {fingerprint}"
    ))
    .send();
}

#[test]
fn media_audio_track_out_of_range() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";
  let bar: &[u8] = b"bar";

  let fingerprint = package(
    &server,
    &Metadata {
      media: Some(Media::Audio {
        tracks: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
      }),
      ..default()
    },
    &[("foo.flac", foo), ("bar.flac", bar)],
  );

  server
    .get(format!("/media/audio/{fingerprint}/track/2"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "track 2 does not exist, package {fingerprint} has 2 tracks"
    ))
    .send();
}

#[test]
fn media_audio_track_package_not_found() {
  let server = TestServer::new();

  let fingerprint = Fingerprint(Hash::bytes(b"foo"));

  server
    .get(format!("/media/audio/{fingerprint}/track/0"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} not found"))
    .send();
}

#[test]
fn media_audio_track_package_without_media() {
  let server = TestServer::new();

  let fingerprint = package(
    &server,
    &Metadata {
      title: Some("foo".parse().unwrap()),
      ..default()
    },
    &[],
  );

  server
    .get(format!("/media/audio/{fingerprint}/track/0"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!(
      "package {fingerprint} does not have media metadata"
    ))
    .send();
}

#[test]
fn media_audio_track_package_without_metadata() {
  let server = TestServer::new();

  let cbor = directory(&[]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/package/{fingerprint}")).send();

  server
    .get(format!("/media/audio/{fingerprint}/track/0"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("package {fingerprint} does not have metadata"))
    .send();
}

#[test]
fn media_audio_track_ranges() {
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
      .get(format!("/media/audio/{fingerprint}/track/0"))
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

  let track: &[u8] = b"foobarbaz";

  let fingerprint = package(
    &server,
    &Metadata {
      media: Some(Media::Audio {
        tracks: vec!["foo.flac".parse().unwrap()],
      }),
      ..default()
    },
    &[("foo.flac", track)],
  );

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
fn media_audio_track_response() {
  let server = TestServer::new();

  let foo: &[u8] = b"foo";
  let bar: &[u8] = b"barbar";

  let fingerprint = package(
    &server,
    &Metadata {
      media: Some(Media::Audio {
        tracks: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
      }),
      ..default()
    },
    &[("foo.flac", foo), ("bar.flac", bar)],
  );

  server
    .get(format!("/media/audio/{fingerprint}/track/0"))
    .assert_header(header::ACCEPT_RANGES, "bytes")
    .assert_header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
    .assert_header(header::CONTENT_LENGTH, "3")
    .assert_header(header::CONTENT_SECURITY_POLICY, "sandbox")
    .assert_header(header::CONTENT_TYPE, "audio/flac")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(foo)))
    .assert_body(foo)
    .send();

  server
    .get(format!("/media/audio/{fingerprint}/track/1"))
    .assert_header(header::CONTENT_LENGTH, "6")
    .assert_header(header::CONTENT_TYPE, "audio/flac")
    .assert_header(header::ETAG, format!("\"{}\"", Hash::bytes(bar)))
    .assert_body(bar)
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
    .post("/missing")
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
    .post("/missing")
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

#[track_caller]
fn package(server: &TestServer, metadata: &Metadata, files: &[(&str, &[u8])]) -> Fingerprint {
  let metadata_cbor = metadata.encode_to_vec();
  let metadata_hash = Hash::bytes(&metadata_cbor);
  server.write_file(&metadata_cbor);

  let mut entries = files
    .iter()
    .map(|&(name, content)| {
      server.write_file(content);
      (
        name,
        EntryType::File,
        Hash::bytes(content),
        content.len().into_u64(),
      )
    })
    .collect::<Vec<(&str, EntryType, Hash, u64)>>();

  entries.push((
    Metadata::CBOR_FILENAME,
    EntryType::File,
    metadata_hash,
    metadata_cbor.len().into_u64(),
  ));

  let cbor = directory(&entries).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/package/{fingerprint}")).send();

  fingerprint
}

#[test]
fn packages_empty() {
  TestServer::new()
    .get("/packages")
    .assert_page(PackagesHtml {
      packages: Vec::new(),
    })
    .send();
}

#[test]
fn packages_include_titles() {
  let server = TestServer::new();

  let fingerprint = package(
    &server,
    &Metadata {
      title: Some("foo".parse().unwrap()),
      ..default()
    },
    &[],
  );

  server
    .get("/packages")
    .assert_page(PackagesHtml {
      packages: vec![(fingerprint, Some("foo".parse().unwrap()))],
    })
    .send();
}

#[test]
fn packages_non_empty() {
  let server = TestServer::new();

  let mut packages = Vec::new();

  for content in [b"foo".as_slice(), b"bar", b"baz"] {
    server.write_file(content);
    let cbor = directory(&[(
      "file",
      EntryType::File,
      Hash::bytes(content),
      content.len().into_u64(),
    )])
    .encode_to_vec();
    let hash = Hash::bytes(&cbor);
    let fingerprint = Fingerprint(hash);
    server.write_file(&cbor);
    server.post(format!("/directory/{hash}")).send();
    server.post(format!("/package/{fingerprint}")).send();
    packages.push((fingerprint, None));
  }

  packages.sort();

  server
    .get("/packages")
    .assert_page(PackagesHtml { packages })
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
fn redirect_destination() {
  let domains = vec!["foo".to_string()];

  assert_eq!(Serve::redirect_destination(&domains, 443), "https://foo");
  assert_eq!(
    Serve::redirect_destination(&domains, 8443),
    "https://foo:8443",
  );
}

#[test]
fn redirect_http_to_https() {
  fn case(path: &str, location: &str) {
    let response = RUNTIME
      .block_on(
        Serve::redirect_router("https://foo".into())
          .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap()),
      )
      .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], location);
    assert_eq!(
      response.headers()[header::X_CONTENT_TYPE_OPTIONS],
      "nosniff"
    );
  }

  case("/", "https://foo/");
  case("/bar", "https://foo/bar");
  case("/bar?baz=qux", "https://foo/bar?baz=qux");
}

#[test]
fn restricted_upload_accepts_admin_token() {
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
    .send();

  server.assert_file(hash);
}

#[test]
fn restricted_upload_rejects_missing_header() {
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
    .send();
}

#[test]
fn restricted_upload_rejects_others() {
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
    .send();
}

#[test]
fn static_files() {
  TestServer::new()
    .get("/static/index.css")
    .assert_static("index.css")
    .send();
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
fn verify_directory_decode_error() {
  let server = TestServer::new();

  let junk = b"junk";
  let hash = Hash::bytes(junk);
  server.write_file(junk);

  server
    .post(format!("/directory/{hash}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!("failed to decode directory {hash}"))
    .send();
}

#[test]
fn verify_directory_file_not_found() {
  let server = TestServer::new();

  let hash = Hash::bytes(b"foo");

  server
    .post(format!("/directory/{hash}"))
    .status(StatusCode::NOT_FOUND)
    .assert_body(format!("file with hash {hash} not found"))
    .send();
}

#[test]
fn verify_directory_idempotent() {
  let server = TestServer::new();

  let dir = directory(&[]);
  let cbor = dir.encode_to_vec();
  let hash = Hash::bytes(&cbor);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();
  server.post(format!("/directory/{hash}")).send();

  server
    .get(format!("/directory/{hash}"))
    .assert_page(DirectoryHtml {
      directory: dir,
      hash,
    })
    .send();
}

#[test]
fn verify_directory_missing_file() {
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
    .send();
}

#[test]
fn verify_directory_rejects_missing_auth_header() {
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
    .send();
}

#[test]
fn verify_directory_succeeds() {
  let server = TestServer::new();

  let file = b"foo";
  let file_hash = Hash::bytes(file);
  server.write_file(file);

  let child = directory(&[("foo", EntryType::File, file_hash, file.len().into_u64())]);
  let child_cbor = child.encode_to_vec();
  let child_hash = Hash::bytes(&child_cbor);
  server.write_file(&child_cbor);

  server.post(format!("/directory/{child_hash}")).send();

  server
    .get(format!("/directory/{child_hash}"))
    .assert_page(DirectoryHtml {
      directory: child,
      hash: child_hash,
    })
    .send();

  let parent = directory(&[(
    "child",
    EntryType::Directory,
    child_hash,
    child_cbor.len().into_u64(),
  )]);
  let parent_cbor = parent.encode_to_vec();
  let parent_hash = Hash::bytes(&parent_cbor);
  server.write_file(&parent_cbor);

  server.post(format!("/directory/{parent_hash}")).send();

  server
    .get(format!("/directory/{parent_hash}"))
    .assert_page(DirectoryHtml {
      directory: parent,
      hash: parent_hash,
    })
    .send();
}

#[test]
fn verify_directory_unverified_subdirectory() {
  let server = TestServer::new();

  let child_cbor = directory(&[]).encode_to_vec();
  let child_hash = Hash::bytes(&child_cbor);
  server.write_file(&child_cbor);

  let parent_cbor = directory(&[(
    "child",
    EntryType::Directory,
    child_hash,
    child_cbor.len().into_u64(),
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
    .send();
}

#[test]
fn verify_package_metadata_decode_error() {
  let server = TestServer::new();

  let junk = b"not cbor";
  let metadata_hash = Hash::bytes(junk);
  server.write_file(junk);

  let cbor = directory(&[(
    Metadata::CBOR_FILENAME,
    EntryType::File,
    metadata_hash,
    junk.len().into_u64(),
  )])
  .encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();

  server
    .post(format!("/package/{fingerprint}"))
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
    artwork: Some("cover.png".parse().unwrap()),
    ..default()
  }
  .encode_to_vec();
  let metadata_hash = Hash::bytes(&metadata);
  server.write_file(&metadata);

  let cbor = directory(&[(
    Metadata::CBOR_FILENAME,
    EntryType::File,
    metadata_hash,
    metadata.len().into_u64(),
  )])
  .encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();

  server
    .post(format!("/package/{fingerprint}"))
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
  let artwork_hash = Hash::bytes(artwork);
  server.write_file(artwork);

  let metadata = Metadata {
    artwork: Some("cover.png".parse().unwrap()),
    ..default()
  }
  .encode_to_vec();
  let metadata_hash = Hash::bytes(&metadata);
  server.write_file(&metadata);

  let cbor = directory(&[
    (
      "cover.png",
      EntryType::File,
      artwork_hash,
      artwork.len().into_u64(),
    ),
    (
      Metadata::CBOR_FILENAME,
      EntryType::File,
      metadata_hash,
      metadata.len().into_u64(),
    ),
  ])
  .encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server.post(format!("/directory/{hash}")).send();

  server.post(format!("/package/{fingerprint}")).send();
}

#[test]
fn verify_package_unverified() {
  let server = TestServer::new();

  let cbor = directory(&[]).encode_to_vec();
  let hash = Hash::bytes(&cbor);
  let fingerprint = Fingerprint(hash);
  server.write_file(&cbor);

  server
    .post(format!("/package/{fingerprint}"))
    .status(StatusCode::BAD_REQUEST)
    .assert_body(format!(
      "package {fingerprint} root directory is unverified"
    ))
    .send();
}
