use super::*;

pub(crate) struct DisplayJson<'a, T>(pub(crate) &'a T);

impl<T: Serialize> Display for DisplayJson<'_, T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if f.alternate() {
      serde_json::to_writer_pretty(Writer(f), self.0).map_err(|_| fmt::Error)
    } else {
      serde_json::to_writer(Writer(f), self.0).map_err(|_| fmt::Error)
    }
  }
}

struct Writer<'a, 'b>(&'a mut Formatter<'b>);

impl Write for Writer<'_, '_> {
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }

  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let s = str::from_utf8(buf).map_err(io::Error::other)?;
    self.0.write_str(s).map_err(io::Error::other)?;
    Ok(buf.len())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::fmt::Write};

  #[test]
  fn display() {
    assert_eq!(DisplayJson(&["foo"]).to_string(), "[\"foo\"]");
    assert_eq!(format!("{:#}", DisplayJson(&["foo"])), "[\n  \"foo\"\n]");
  }

  #[test]
  fn error() {
    assert_eq!(
      write!(
        String::new(),
        "{}",
        DisplayJson(&Decoded::<u8>::Unknown(Vec::new())),
      ),
      Err(fmt::Error),
    );
  }
}
