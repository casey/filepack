use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VideoFormat {
  pub(crate) tracks: Vec<Track>,
  pub(crate) ty: VideoType,
}

impl Display for VideoFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.ty)?;

    for track in &self.tracks {
      write!(f, " {track}")?;
    }

    Ok(())
  }
}
