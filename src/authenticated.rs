use {
  super::*,
  axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts},
  axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
  },
  subcommand::serve::WriteConfig,
};

pub(crate) struct Authenticated;

impl<S: Send + Sync> FromRequestParts<S> for Authenticated {
  type Rejection = ServerError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> ServerResult<Self> {
    let write_config = parts.extensions.get::<Arc<WriteConfig>>().cloned().unwrap();

    let (domain, key) = match write_config.as_ref() {
      WriteConfig::Admin { domain, key } => (domain, *key),
      WriteConfig::Forbidden => return Err(ServerError::WriteForbidden),
      WriteConfig::Unrestricted => return Ok(Self),
    };

    let TypedHeader(Authorization(bearer)) = parts
      .extract::<TypedHeader<Authorization<Bearer>>>()
      .await
      .map_err(|err| {
        if err.is_missing() {
          ServerError::AuthorizationMissing
        } else {
          ServerError::AuthorizationMalformed
        }
      })?;

    Claims::verify(
      key,
      domain,
      now().context(server_error::Time)?,
      bearer.token(),
    )
    .context(server_error::AuthorizationInvalid)?;

    Ok(Self)
  }
}

impl Describe for Authenticated {
  fn describe(endpoint: &mut reference::Endpoint, _registry: &mut deco::Registry) {
    endpoint.authenticated = true;
  }
}
