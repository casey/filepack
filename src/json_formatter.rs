use {super::*, serde_json::ser::PrettyFormatter};

pub(crate) struct JsonFormatter {
  inner: PrettyFormatter<'static>,
  key: bool,
  style: Style,
}

impl JsonFormatter {
  pub(crate) fn new(style: Style) -> Self {
    Self {
      inner: PrettyFormatter::new(),
      key: false,
      style,
    }
  }

  fn string(&self) -> owo_colors::Style {
    if self.key {
      self.style.json_key()
    } else {
      self.style.json_string()
    }
    .effective_style()
  }
}

impl serde_json::ser::Formatter for JsonFormatter {
  fn begin_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.begin_array(writer)
  }

  fn begin_array_value<W: ?Sized + Write>(
    &mut self,
    writer: &mut W,
    first: bool,
  ) -> io::Result<()> {
    self.inner.begin_array_value(writer, first)
  }

  fn begin_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.begin_object(writer)
  }

  fn begin_object_key<W: ?Sized + Write>(&mut self, writer: &mut W, first: bool) -> io::Result<()> {
    self.key = true;
    self.inner.begin_object_key(writer, first)
  }

  fn begin_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.begin_object_value(writer)
  }

  fn begin_string<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    write!(writer, "{}", self.string().prefix_formatter())?;
    self.inner.begin_string(writer)
  }

  fn end_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.end_array(writer)
  }

  fn end_array_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.end_array_value(writer)
  }

  fn end_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.end_object(writer)
  }

  fn end_object_key<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.key = false;
    self.inner.end_object_key(writer)
  }

  fn end_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.end_object_value(writer)
  }

  fn end_string<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.inner.end_string(writer)?;
    write!(writer, "{}", self.string().suffix_formatter())
  }

  fn write_null<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
    let style = self.style.json_null().effective_style();
    write!(writer, "{}", style.prefix_formatter())?;
    self.inner.write_null(writer)?;
    write!(writer, "{}", style.suffix_formatter())
  }
}
