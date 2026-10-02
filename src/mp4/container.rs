use super::*;

pub(crate) struct Container<'a> {
  atoms: Vec<Atom<'a>>,
}

impl<'a> Container<'a> {
  pub(crate) fn atoms(&self) -> &[Atom<'a>] {
    &self.atoms
  }

  pub(crate) fn many<T: Parse>(&self) -> Result<Vec<T>, Mp4Error> {
    self
      .atoms
      .iter()
      .filter(|atom| atom.ty == T::TYPE)
      .map(Atom::parse)
      .collect()
  }

  pub(crate) fn new(atoms: Vec<Atom<'a>>) -> Self {
    Self { atoms }
  }

  pub(crate) fn one<T: Parse>(&self) -> Result<T, Mp4Error> {
    self.optional()?.context(mp4_error::Missing { ty: T::TYPE })
  }

  pub(crate) fn optional<T: Parse>(&self) -> Result<Option<T>, Mp4Error> {
    let mut atoms = self.atoms.iter().filter(|atom| atom.ty == T::TYPE);

    let Some(atom) = atoms.next() else {
      return Ok(None);
    };

    ensure!(atoms.next().is_none(), mp4_error::Duplicate { ty: T::TYPE });

    Ok(Some(atom.parse()?))
  }

  pub(crate) fn parse(bytes: &'a [u8]) -> Result<Self, Mp4Error> {
    Ok(Self {
      atoms: Atoms::new(bytes).collect::<Result<Vec<Atom>, Mp4Error>>()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, PartialEq)]
  struct Foo(u8);

  impl Parse for Foo {
    const TYPE: Fourcc = Fourcc(*b"foob");

    fn parse(mut reader: Reader) -> Result<Self, Mp4Error> {
      Ok(Self(reader.u8()?))
    }
  }

  fn container(atoms: &[&[u8]]) -> Vec<u8> {
    atoms
      .iter()
      .map(|body| Mp4Builder::atom(*b"foob", body))
      .collect::<Vec<Vec<u8>>>()
      .concat()
  }

  #[test]
  fn many() {
    let bytes = container(&[&[1], &[2]]);
    assert_eq!(
      Container::parse(&bytes).unwrap().many::<Foo>().unwrap(),
      vec![Foo(1), Foo(2)],
    );
  }

  #[test]
  fn one() {
    assert_eq!(
      Container::parse(&container(&[&[1]]))
        .unwrap()
        .one::<Foo>()
        .unwrap(),
      Foo(1),
    );

    assert_eq!(
      Container::parse(&container(&[]))
        .unwrap()
        .one::<Foo>()
        .unwrap_err()
        .to_string(),
      "missing `foob` box",
    );

    assert_eq!(
      Container::parse(&container(&[&[1], &[2]]))
        .unwrap()
        .one::<Foo>()
        .unwrap_err()
        .to_string(),
      "duplicate `foob` box",
    );

    assert_eq!(
      Container::parse(&container(&[&[]]))
        .unwrap()
        .one::<Foo>()
        .unwrap_err()
        .iter_chain()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(": "),
      "invalid `foob` box: truncated",
    );
  }

  #[test]
  fn optional() {
    assert_eq!(
      Container::parse(&container(&[]))
        .unwrap()
        .optional::<Foo>()
        .unwrap(),
      None,
    );

    assert_eq!(
      Container::parse(&container(&[&[1]]))
        .unwrap()
        .optional::<Foo>()
        .unwrap(),
      Some(Foo(1)),
    );
  }
}
