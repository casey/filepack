use super::*;

pub(crate) struct Bech32mEncoder {
  fes: Vec<Fe32>,
  ty: Bech32mType,
}

impl Bech32mEncoder {
  pub(crate) fn bytes(&mut self, bytes: &[u8]) {
    self.fes.extend(bytes.iter().copied().bytes_to_fes());
  }

  pub(crate) fn fes(&mut self, fes: &[Fe32]) {
    self.fes.extend_from_slice(fes);
  }

  pub(crate) fn new(ty: Bech32mType) -> Self {
    Self {
      fes: vec![BECH32M_VERSION],
      ty,
    }
  }
}

impl Display for Bech32mEncoder {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for c in self
      .fes
      .iter()
      .copied()
      .with_checksum::<bech32::Bech32m>(self.ty.hrp())
      .chars()
    {
      f.write_char(c)?;
    }

    Ok(())
  }
}
