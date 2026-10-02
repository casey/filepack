use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum SampleEntry {
  Avc1(Avc1),
  Mp4a(Mp4a),
  Unknown(Fourcc),
}

impl SampleEntry {
  pub(crate) fn parse(atom: &Atom) -> Result<Self, Mp4Error> {
    Ok(match atom.ty {
      Avc1::TYPE => Self::Avc1(atom.parse()?),
      Mp4a::TYPE => Self::Mp4a(atom.parse()?),
      ty => Self::Unknown(ty),
    })
  }
}
