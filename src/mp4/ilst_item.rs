use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct IlstItem {
  pub(crate) data: Vec<Data>,
  pub(crate) ty: Fourcc,
}

impl IlstItem {
  pub(crate) fn parse(atom: &Atom) -> Result<Self, Mp4Error> {
    Ok(Self {
      data: Container::parse(atom.body)
        .and_then(|container| container.many())
        .context(mp4_error::Invalid { ty: atom.ty })?,
      ty: atom.ty,
    })
  }
}
