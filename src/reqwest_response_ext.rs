use super::*;

pub(crate) trait ReqwestResponseExt: Sized {
  fn check_status(self) -> Result<Self>;

  fn deco<T: Decode>(self) -> Result<T>;

  fn found(self) -> Result<bool>;
}

impl ReqwestResponseExt for reqwest::blocking::Response {
  fn check_status(self) -> Result<Self> {
    let status = self.status();

    if !status.is_success() {
      let url = self.url().clone();

      let body = self
        .text()
        .with_context(|_| error::ResponseBody { url: url.clone() })?;

      return Err(error::ResponseStatus { body, status, url }.build());
    }

    Ok(self)
  }

  fn deco<T: Decode>(self) -> Result<T> {
    let url = self.url().clone();

    let bytes = self
      .bytes()
      .with_context(|_| error::ResponseBody { url: url.clone() })?;

    T::decode_from_slice(&bytes).context(error::DecodeResponse { url })
  }

  fn found(self) -> Result<bool> {
    if self.status() == StatusCode::NOT_FOUND {
      Ok(false)
    } else {
      self.check_status()?;
      Ok(true)
    }
  }
}
