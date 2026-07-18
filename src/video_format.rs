use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VideoFormat {
  pub(crate) codecs: Vec<Codec>,
  pub(crate) ty: VideoType,
}

impl Display for VideoFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.ty)?;

    for codec in &self.codecs {
      write!(f, " {codec}")?;
    }

    Ok(())
  }
}
