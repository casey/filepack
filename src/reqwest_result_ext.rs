use super::*;

pub(crate) trait ReqwestResultExt {
  fn check_status(self) -> Result<reqwest::blocking::Response>;
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

      return Err(error::ResponseStatus { status, url, body }.build());
    }

    Ok(response)
  }
}
