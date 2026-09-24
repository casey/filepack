use {
  super::*,
  axum::extract::{FromRequest, Request},
};

pub(crate) struct DecoRequest<T, const LIMIT: usize>(pub(crate) T);

impl<T: DecodeOwned, S: Send + Sync, const LIMIT: usize> FromRequest<S> for DecoRequest<T, LIMIT> {
  type Rejection = ServerError;

  async fn from_request(request: Request, _state: &S) -> ServerResult<Self> {
    let bytes = axum::body::to_bytes(request.into_body(), LIMIT)
      .await
      .context(server_error::DecoBody)?;

    Ok(Self(
      T::decode_from_slice_with_options(DecodeOptions::strict(), &bytes)
        .context(server_error::DecoDecode)?,
    ))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, tokio::runtime::Runtime};

  #[test]
  fn body_exceeding_limit_is_rejected() {
    let request = Request::builder().body(Body::from(vec![0; 5])).unwrap();

    let result = Runtime::new()
      .unwrap()
      .block_on(DecoRequest::<Vec<u8>, 4>::from_request(request, &()));

    assert_matches!(
      result.map(|DecoRequest(value)| value),
      Err(ServerError::DecoBody { .. }),
    );
  }

  #[test]
  fn unknown_field_is_rejected() {
    #[derive(Debug, Decode, PartialEq)]
    struct Foo {
      #[n(0)]
      foo: u64,
    }

    let body = with_unknown_field(BTreeMap::from([(0u64, 1u64)]));

    let request = Request::builder().body(Body::from(body)).unwrap();

    let result = Runtime::new()
      .unwrap()
      .block_on(DecoRequest::<Foo, 1024>::from_request(request, &()));

    assert_matches!(
      result.map(|DecoRequest(value)| value),
      Err(ServerError::DecoDecode {
        source: DecodeError::UnknownField { key: u64::MAX },
      }),
    );
  }
}
