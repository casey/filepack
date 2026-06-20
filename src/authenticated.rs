use {
  super::*,
  axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts},
  axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
  },
  subcommand::serve::AuthConfig,
};

pub(crate) struct Authenticated;

impl<S: Send + Sync> FromRequestParts<S> for Authenticated {
  type Rejection = ServerError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> ServerResult<Self> {
    let Some(auth) = parts.extensions.get::<Arc<AuthConfig>>().cloned() else {
      return Ok(Self);
    };

    let admin = auth.admin.context(server_error::UploadForbidden)?;

    let TypedHeader(Authorization(bearer)) = parts
      .extract::<TypedHeader<Authorization<Bearer>>>()
      .await
      .map_err(|err| {
        if err.is_missing() {
          server_error::AuthorizationMissing.build()
        } else {
          server_error::AuthorizationMalformed.build()
        }
      })?;

    Token::verify(
      admin,
      auth.audience.as_ref().map(|domain| domain.as_ref()),
      bearer.token(),
    )?;

    Ok(Self)
  }
}
