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
