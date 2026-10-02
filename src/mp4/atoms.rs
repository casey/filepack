use super::*;

pub(crate) struct Atoms<'a> {
  bytes: &'a [u8],
  done: bool,
  offset: usize,
}

impl<'a> Atoms<'a> {
  fn lex(&mut self) -> Result<Atom<'a>, Mp4Error> {
    let header = BoxHeader::parse(&mut Reader::new(&self.bytes[self.offset..]))?;

    let start = self.offset + usize::try_from(header.len).unwrap();

    let end = match header.size {
      None => self.bytes.len(),
      Some(size) => usize::try_from(size)
        .ok()
        .and_then(|size| self.offset.checked_add(size))
        .context(mp4_error::Truncated)?,
    };

    let body = self.bytes.get(start..end).context(mp4_error::Truncated)?;

    self.offset = end;

    Ok(Atom {
      body,
      ty: header.ty,
    })
  }

  pub(crate) fn new(bytes: &'a [u8]) -> Self {
    Self {
      bytes,
      done: false,
      offset: 0,
    }
  }
}

impl<'a> Iterator for Atoms<'a> {
  type Item = Result<Atom<'a>, Mp4Error>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.done || self.offset >= self.bytes.len() {
      return None;
    }

    let atom = self.lex();

    if atom.is_err() {
      self.done = true;
    }

    Some(atom)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lex() {
    let bytes = [
      Mp4Builder::atom(*b"foob", b"ab"),
      Mp4Builder::atom(*b"barb", b""),
      vec![0, 0, 0, 0, b'b', b'a', b'z', b'b', 1, 2, 3],
    ]
    .concat();

    assert_eq!(
      Atoms::new(&bytes)
        .collect::<Result<Vec<Atom>, Mp4Error>>()
        .unwrap(),
      vec![
        Atom {
          body: b"ab",
          ty: Fourcc(*b"foob"),
        },
        Atom {
          body: b"",
          ty: Fourcc(*b"barb"),
        },
        Atom {
          body: &[1, 2, 3],
          ty: Fourcc(*b"bazb"),
        },
      ],
    );
  }

  #[test]
  fn truncated() {
    let bytes = [
      Mp4Builder::atom(*b"foob", b"ab"),
      vec![0, 0, 0, 9, b'b', b'a', b'r', b'b'],
    ]
    .concat();

    let mut atoms = Atoms::new(&bytes);

    assert_eq!(atoms.next().unwrap().unwrap().ty, Fourcc(*b"foob"));
    assert_matches!(atoms.next().unwrap().unwrap_err(), Mp4Error::Truncated);
    assert!(atoms.next().is_none());
  }
}
