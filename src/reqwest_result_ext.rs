use super::*;

pub(crate) trait ReqwestResultExt {
  fn check_status(self) -> Result<reqwest::blocking::Response>;
  fn found(self) -> Result<Option<reqwest::blocking::Response>>;
}

impl ReqwestResultExt for reqwest::Result<reqwest::blocking::Response> {
  fn check_status(self) -> Result<reqwest::blocking::Response> {
    let response = self.context(error::Request)?;

    let status = response.status();
    let url = response.url().clone();

    if !status.is_success() {
      let body = response
        .text()
        .with_context(|_| error::ResponseBody { url: url.clone() })?;

      return Err(error::ResponseStatus { body, status, url }.build());
    }

    Ok(response)
  }

  fn found(self) -> Result<Option<reqwest::blocking::Response>> {
    let response = self.context(error::Request)?;

    if response.status() == StatusCode::NOT_FOUND {
      Ok(None)
    } else {
      Ok(Some(Ok(response).check_status()?))
    }
  }
}
