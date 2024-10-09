use super::*;

pub(crate) trait IoResultExt<T> {
  fn into_option(self) -> io::Result<Option<T>>;
}

impl<T> IoResultExt<T> for io::Result<T> {
  fn into_option(self) -> io::Result<Option<T>> {
    match self {
      Err(err) => {
        if err.kind() == io::ErrorKind::NotFound {
          Ok(None)
        } else {
          Err(err)
        }
      }
      Ok(value) => Ok(Some(value)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn into_option() {
    Err::<(), io::Error>(io::Error::new(io::ErrorKind::InvalidInput, "foo"))
      .into_option()
      .unwrap_err();

    assert_eq!(
      Err::<(), io::Error>(io::Error::new(io::ErrorKind::NotFound, "foo"))
        .into_option()
        .unwrap(),
      None,
    );
  }
}
