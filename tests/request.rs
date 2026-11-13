use {super::*, pretty_assertions::assert_eq};

pub(crate) struct Request<'a> {
  body: Option<Vec<u8>>,
  child: &'a Child,
  headers: Vec<(HeaderName, String)>,
  host: Option<String>,
  method: Method,
  path: String,
  response_body: Option<Vec<u8>>,
  response_headers: BTreeMap<String, String>,
  status: StatusCode,
  version: Version,
}

impl<'a> Request<'a> {
  pub(crate) fn assert_header(mut self, name: HeaderName, value: impl Into<String>) -> Self {
    assert!(
      self
        .response_headers
        .insert(name.to_string(), value.into())
        .is_none()
    );
    self
  }

  pub(crate) fn assert_page(mut self, page: impl Page) -> Self {
    let (parts, body) = page.page(None).into_response().into_response().into_parts();

    self.status = parts.status;

    for (name, value) in &parts.headers {
      self = self.assert_header(name.clone(), value.to_str().unwrap());
    }

    self.response_body = Some(Vec::from(
      RUNTIME.block_on(body::to_bytes(body, usize::MAX)).unwrap(),
    ));

    self
  }

  pub(crate) fn body(mut self, body: impl AsRef<[u8]>) -> Self {
    self.body = Some(body.as_ref().to_vec());
    self
  }

  pub(crate) fn header(mut self, name: HeaderName, value: impl Into<String>) -> Self {
    self.headers.push((name, value.into()));
    self
  }

  pub(crate) fn host(mut self, host: &str) -> Self {
    self.host = Some(host.into());
    self
  }

  pub(crate) fn new(child: &'a Child, method: Method, path: impl Into<String>) -> Self {
    Self {
      body: None,
      child,
      headers: Vec::new(),
      host: None,
      method,
      path: path.into(),
      response_body: None,
      response_headers: BTreeMap::new(),
      status: StatusCode::OK,
      version: Version::HTTP_11,
    }
  }

  #[track_caller]
  pub(crate) fn send(self) -> Vec<u8> {
    let client = reqwest::blocking::Client::builder().redirect(reqwest::redirect::Policy::none());

    let mut client = match self.version {
      Version::HTTP_11 => client.http1_only(),
      Version::HTTP_2 => client.http2_prior_knowledge(),
      _ => unreachable!(),
    };

    let address = if let Some(host) = &self.host {
      client = client.resolve(
        host,
        format!("127.0.0.1:{}", self.child.port()).parse().unwrap(),
      );
      format!("http://{host}")
    } else {
      self.child.address()
    };

    let client = client.build().unwrap();

    let mut request = client.request(self.method, format!("{address}{}", self.path));

    for (name, value) in self.headers {
      request = request.header(name, value);
    }

    if let Some(body) = self.body {
      request = request.body(body);
    }

    let response = request.send().unwrap();

    assert_eq!(response.version(), self.version);

    assert_eq!(response.status(), self.status);

    for (name, value) in &self.response_headers {
      assert_eq!(
        response
          .headers()
          .get(name.as_str())
          .map(|value| value.to_str().unwrap()),
        Some(value.as_str()),
        "header `{name}` mismatch",
      );
    }

    let body = Vec::from(response.bytes().unwrap());

    if let Some(expected) = self.response_body {
      assert_eq!(body, expected);
    }

    body
  }

  pub(crate) fn status(mut self, status: StatusCode) -> Self {
    self.status = status;
    self
  }

  pub(crate) fn version(mut self, version: Version) -> Self {
    self.version = version;
    self
  }
}
