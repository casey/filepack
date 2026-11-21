use super::*;

pub(crate) struct DisplayJson<'a, T> {
  style: Style,
  value: &'a T,
}

impl<T: Serialize> DisplayJson<'_, T> {
  pub(crate) fn print(value: &T) -> Result {
    println!(
      "{}",
      DisplayJson {
        style: Style::stdout(),
        value,
      },
    );

    Ok(())
  }
}

impl<T: Serialize> Display for DisplayJson<'_, T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self
      .value
      .serialize(&mut serde_json::Serializer::with_formatter(
        Writer(f),
        JsonFormatter::new(self.style),
      ))
      .map_err(|_| fmt::Error)
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
  fn color() {
    assert_eq!(
      DisplayJson {
        style: Style::new(true),
        value: &serde_json::json!({"foo": ["bar", 1, null]}),
      }
      .to_string(),
      "{\n  \u{1b}[34;1m\"foo\"\u{1b}[0m: [\n    \u{1b}[32m\"bar\"\u{1b}[0m,\n    1,\n    \
       \u{1b}[2mnull\u{1b}[0m\n  ]\n}",
    );
  }

  #[test]
  fn display() {
    assert_eq!(
      DisplayJson {
        style: Style::new(false),
        value: &serde_json::json!({"foo": ["bar", 1, null]}),
      }
      .to_string(),
      "{\n  \"foo\": [\n    \"bar\",\n    1,\n    null\n  ]\n}",
    );
  }

  #[test]
  fn error() {
    assert_eq!(
      write!(
        String::new(),
        "{}",
        DisplayJson {
          style: Style::new(false),
          value: &Decoded::<u8>::Unknown(Vec::new()),
        },
      ),
      Err(fmt::Error),
    );
  }
}
