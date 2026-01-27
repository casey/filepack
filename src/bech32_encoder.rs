use super::*;

pub(crate) struct Bech32Encoder {
  fes: Vec<Fe32>,
  ty: Bech32Type,
}

impl Bech32Encoder {
  pub(crate) fn bytes(&mut self, bytes: &[u8]) {
    self.fes.extend(bytes.iter().copied().bytes_to_fes());
  }

  pub(crate) fn new(ty: Bech32Type) -> Self {
    Self {
      fes: vec![BECH32_VERSION],
      ty,
    }
  }
}

impl Display for Bech32Encoder {
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
