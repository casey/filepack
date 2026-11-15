use super::*;

pub(crate) struct ErrorChain<'a> {
  pub(crate) error: Option<&'a Arc<ServerError>>,
  pub(crate) status: StatusCode,
}

impl Display for ErrorChain<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.status)?;

    if let Some(error) = self.error {
      for error in error.iter_chain() {
        write!(f, ": {error}")?;
      }
    }

    Ok(())
  }
}
