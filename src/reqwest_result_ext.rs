use super::*;

pub(crate) trait ReqwestResultExt {
  fn check_status(self, url: &Url) -> Result<reqwest::blocking::Response>;
}

impl ReqwestResultExt for reqwest::Result<reqwest::blocking::Response> {
  fn check_status(self, url: &Url) -> Result<reqwest::blocking::Response> {
    let response = self.with_context(|_| error::Request { url: url.clone() })?;

    if !response.status().is_success() {
      return Err(
        error::ResponseStatus {
          status: response.status(),
          url: url.clone(),
          body: response
            .text()
            .with_context(|_| error::ResponseBody { url: url.clone() })?,
        }
        .build(),
      );
    }

    Ok(response)
  }
}
